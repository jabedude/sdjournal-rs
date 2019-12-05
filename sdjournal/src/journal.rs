use byteorder::{BigEndian, LittleEndian, ReadBytesExt};
use std::fmt;
use std::str;
use std::io::{Error, ErrorKind, Read, Result, Seek, SeekFrom};
use std::convert::TryInto;

use crate::iter::*;
use crate::traits::{SizedObject, HashableObject};
use crate::hash::rhash64;

// TODO: compression support
// TODO: Result/Error type
// TODO: work on entrt struct to allow for propper formatting of entries

/// Code relating to the jounral structure goes here.

pub const OBJECT_HEADER_SZ: u64 = 16;
pub const DATA_OBJECT_HEADER_SZ: u64 = 48;
pub const FIELD_OBJECT_HEADER_SZ: u64 = 24;

pub const OBJECT_COMPRESSED_XZ: u8 = 1 << 0;
pub const OBJECT_COMPRESSED_LZ4: u8 = 1 << 1;
pub const OBJECT_COMPRESSED_MASK: u8 = OBJECT_COMPRESSED_XZ | OBJECT_COMPRESSED_LZ4;

pub const TAG_LENGTH: usize = (256 / 8);

pub type ObjectOffset = u64;

#[inline(always)]
pub(crate) fn is_valid64(u: u64) -> bool {
    u & 7 == 0
}

#[inline(always)]
pub(crate) fn align64(u: u64) -> u64 {
    (u + 7u64) & !7u64
}

pub struct Journal<'a, T>
where
    &'a T: Read + Seek,
{
    pub file: &'a T,
    pub header: JournalHeader,
}

impl<'a, T> Journal<'a, T>
where
    &'a T: Read + Seek,
{
    pub fn new(bytes: &'a T) -> Result<Journal<'a, T>> {
        let header = JournalHeader::new(bytes)?;

        Ok(Journal {
            file: bytes,
            header: header,
        })
    }

    pub fn obj_iter(&self) -> ObjectIter<'a, T> {
        let start = self.header.field_hash_table_offset - OBJECT_HEADER_SZ;
        ObjectIter::new(self.file, start)
    }

    /// Iterate over all header objects in journal
    pub fn iter_headers(&self) -> ObjectHeaderIter<'a, T> {
        let start = self.header.field_hash_table_offset - OBJECT_HEADER_SZ;
        ObjectHeaderIter::new(self.file, start)
    }

    /// Iterate over all entry objects in the journal
    pub fn iter_entries(&self) -> EntryIter<'a, T> {
        let start = self.header.entry_array_offset;
        let n_objects = self.header.n_objects;
        EntryIter::new(self.file, start, n_objects)
    }

    pub fn ea_iter(&self) -> EntryArrayIter<'a, T> {
        let start = self.header.entry_array_offset;
        EntryArrayIter::new(self.file, start)
    }

    // TODO: add more tests in verify
    pub fn verify(&self) -> bool {
        for obj in self.obj_iter() {
            if let Object::Data(d) = obj {
                let stored_hash = d.hash;
                let calc_hash = d.hash();
                if stored_hash != calc_hash {
                    return false;
                }
            }
        }
        true
    }
}

pub fn get_obj_at_offset<T: Read + Seek>(file: &mut T, offset: u64) -> Result<Object> {
    //let mut file = Cursor::new(file);

    if !is_valid64(offset) {
        return Err(Error::new(ErrorKind::Other, "Invalid offset"));
    }

    file.seek(SeekFrom::Start(offset))?;
    let type_ = file.read_u8()?;
    match type_ {
        0 => return Err(Error::new(ErrorKind::Other, "Unused Object")),
        1 => {
            let flags = file.read_u8()?;
            let mut reserved = [0u8; 6];
            file.read_exact(&mut reserved)?;
            let size = file.read_u64::<LittleEndian>()?;
            let obj_header = ObjectHeader {
                type_: ObjectType::ObjectData,
                flags: flags,
                reserved: reserved,
                size: size,
            };
            let hash = file.read_u64::<LittleEndian>()?;
            let next_hash_offset = file.read_u64::<LittleEndian>()?;
            let next_field_offset = file.read_u64::<LittleEndian>()?;
            let entry_offset = file.read_u64::<LittleEndian>()?;
            let entry_array_offset = file.read_u64::<LittleEndian>()?;
            let n_entries = file.read_u64::<LittleEndian>()?;
            let mut payload: Vec<u8> = vec![0u8; (size - 64) as usize];
            file.read_exact(&mut payload)?;

            let data_object = DataObject {
                object: obj_header,
                hash: hash,
                next_hash_offset: next_hash_offset,
                next_field_offset: next_field_offset,
                entry_offset: entry_offset,
                entry_array_offset: entry_array_offset,
                n_entries: n_entries,
                payload: payload,
            };
            return Ok(Object::Data(data_object));
        }
        2 => {
            let flags = file.read_u8()?;
            let mut reserved = [0u8; 6];
            file.read_exact(&mut reserved)?;
            let size = file.read_u64::<LittleEndian>()?;
            let object_header = ObjectHeader {
                type_: ObjectType::ObjectField,
                flags: flags,
                reserved: reserved,
                size: size,
            };
            let hash = file.read_u64::<LittleEndian>()?;
            let next_hash_offset = file.read_u64::<LittleEndian>()?;
            let head_data_offset = file.read_u64::<LittleEndian>()?;
            let mut payload: Vec<u8> = vec![0u8; (size - 40) as usize];
            file.read_exact(&mut payload)?;

            let field_object = FieldObject {
                object: object_header,
                hash: hash,
                next_hash_offset: next_hash_offset,
                head_data_offset: head_data_offset,
                payload: payload,
            };
            return Ok(Object::Field(field_object));
        }
        3 => {
            let flags = file.read_u8()?;
            let mut reserved = [0u8; 6];
            file.read_exact(&mut reserved)?;
            let size = file.read_u64::<LittleEndian>()?;
            let object_header = ObjectHeader {
                type_: ObjectType::ObjectEntry,
                flags: flags,
                reserved: reserved,
                size: size,
            };
            let seqnum = file.read_u64::<LittleEndian>()?;
            let realtime = file.read_u64::<LittleEndian>()?;
            let monotonic = file.read_u64::<LittleEndian>()?;
            let boot_id = file.read_u128::<LittleEndian>()?;
            let xor_hash = file.read_u64::<LittleEndian>()?;
            let mut items: Vec<EntryItem> =
                Vec::with_capacity(((size - 48) / 16).try_into().unwrap());
            for _ in 1..((size - 48) / 16) {
                let object_offset = file.read_u64::<LittleEndian>()?;
                let hash = file.read_u64::<LittleEndian>()?;
                let saved_offset = file.seek(SeekFrom::Current(0))?;
                let item_obj = get_obj_at_offset(file, object_offset)?;
                file.seek(SeekFrom::Start(saved_offset))?;
                let item = EntryItem {
                    object_offset: object_offset,
                    hash: hash,
                    item: item_obj,
                };
                items.push(item);
            }
            let entry_object = EntryObject {
                object: object_header,
                seqnum: seqnum,
                realtime: realtime,
                monotonic: monotonic,
                boot_id: boot_id,
                xor_hash: xor_hash,
                items: items,
            };
            return Ok(Object::Entry(entry_object));
        }
        4 => {
            let flags = file.read_u8()?;
            let mut reserved = [0u8; 6];
            file.read_exact(&mut reserved)?;
            let size = file.read_u64::<LittleEndian>()?;
            let object_header = ObjectHeader {
                type_: ObjectType::ObjectDataHashTable,
                flags: flags,
                reserved: reserved,
                size: size,
            };
            let mut items: Vec<HashItem> =
                Vec::with_capacity(((size - 48) / 16).try_into().unwrap());
            for _ in 0..((size - 48) / 16) {
                let hash_head_offset = file.read_u64::<LittleEndian>()?;
                let tail_hash_offset = file.read_u64::<LittleEndian>()?;
                let item = HashItem {
                    hash_head_offset: hash_head_offset,
                    tail_hash_offset: tail_hash_offset,
                };
                items.push(item);
            }
            let hash_table = HashTableObject {
                object: object_header,
                items: items,
            };
            return Ok(Object::HashTable(hash_table));
        }
        5 => {
            let flags = file.read_u8()?;
            let mut reserved = [0u8; 6];
            file.read_exact(&mut reserved)?;
            let size = file.read_u64::<LittleEndian>()?;
            let object_header = ObjectHeader {
                type_: ObjectType::ObjectFieldHashTable,
                flags: flags,
                reserved: reserved,
                size: size,
            };
            let mut items: Vec<HashItem> =
                Vec::with_capacity(((size - 48) / 16).try_into().unwrap());
            for _ in 0..((size - 48) / 16) {
                let hash_head_offset = file.read_u64::<LittleEndian>()?;
                let tail_hash_offset = file.read_u64::<LittleEndian>()?;
                let item = HashItem {
                    hash_head_offset: hash_head_offset,
                    tail_hash_offset: tail_hash_offset,
                };
                items.push(item);
            }
            let hash_table = HashTableObject {
                object: object_header,
                items: items,
            };
            return Ok(Object::HashTable(hash_table));
        }
        6 => {
            let flags = file.read_u8()?;
            let mut reserved = [0u8; 6];
            file.read_exact(&mut reserved)?;
            let size = file.read_u64::<LittleEndian>()?;
            let object_header = ObjectHeader {
                type_: ObjectType::ObjectEntryArray,
                flags: flags,
                reserved: reserved,
                size: size,
            };
            let next_entry_array_offset = file.read_u64::<LittleEndian>()?;
            let mut items: Vec<u64> = Vec::with_capacity(((size - 20) / 8).try_into().unwrap());
            for _ in 0..((size - 20) / 8) {
                let item = file.read_u64::<LittleEndian>()?;
                if item == 0u64 {
                    continue;
                }
                items.push(item);
            }
            let entry_array_object = EntryArrayObject {
                object: object_header,
                next_entry_array_offset: next_entry_array_offset,
                items: items,
            };
            return Ok(Object::EntryArray(entry_array_object));
        }
        7 => {
            let flags = file.read_u8()?;
            let mut reserved = [0u8; 6];
            file.read_exact(&mut reserved)?;
            let size = file.read_u64::<LittleEndian>()?;
            let object_header = ObjectHeader {
                type_: ObjectType::ObjectTag,
                flags: flags,
                reserved: reserved,
                size: size,
            };
            let seqnum = file.read_u64::<LittleEndian>()?;
            let epoch = file.read_u64::<LittleEndian>()?;
            let mut tag = [0u8; 256 / 8];
            file.read_exact(&mut tag)?;
            let tag_object = TagObject {
                object: object_header,
                seqnum: seqnum,
                epoch: epoch,
                tag: tag, /* SHA-256 HMAC */
            };
            return Ok(Object::Tag(tag_object));
        }
        _ => return Err(Error::new(ErrorKind::Other, "Unused MAX Object")),
    }
}

#[derive(Debug, PartialEq)]
pub enum JournalState {
    Offline,
    Online,
    Archived,
    StateMax,
}

impl fmt::Display for JournalState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            JournalState::Offline => write!(f, "OFFLINE"),
            JournalState::Online => write!(f, "ONLINE"),
            JournalState::Archived => write!(f, "ARCHIVED"),
            JournalState::StateMax => write!(f, "MAX"),
        }
    }
}

/// Represents all the possible types of objects in a journal file.
#[derive(Debug, PartialEq)]
pub enum Object {
    /// Holds data in the payload field
    Data(DataObject),
    /// Holds the field name data, such as "_SYSTEMD_UNIT"
    Field(FieldObject),
    /// Represents a log entry
    Entry(EntryObject),
    /// A hash table with offsets to data and field objects
    HashTable(HashTableObject),
    /// A hash table with offsets to data and field objects
    EntryArray(EntryArrayObject),
    /// An object used to seal the journal from modification
    Tag(TagObject),
}

impl fmt::Display for Object {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Object::Data(_) => write!(f, "Data object"),
            Object::Field(_) => write!(f, "Field object"),
            Object::Entry(_) => write!(f, "Entry object"),
            Object::HashTable(_) => write!(f, "HashTable object"),
            Object::EntryArray(_) => write!(f, "Entry array object"),
            Object::Tag(_) => write!(f, "Tag object"),
        }
    }
}

impl SizedObject for Object {
    fn size(&self) -> u64 {
        match self {
            Object::Data(d) => return d.object.size,
            Object::Field(f) => return f.object.size,
            Object::Entry(e) => return e.object.size,
            Object::HashTable(ht) => return ht.object.size,
            Object::EntryArray(ea) => return ea.object.size,
            Object::Tag(t) => return t.object.size,
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum ObjectType {
    ObjectUnused = 0,
    ObjectData = 1,
    ObjectField = 2,
    ObjectEntry = 3,
    ObjectDataHashTable = 4,
    ObjectFieldHashTable = 5,
    ObjectEntryArray = 6,
    ObjectTag = 7,
    ObjectTypeMax,
}

/// The common object header for any object
#[derive(Debug, PartialEq)]
pub struct ObjectHeader {
    pub type_: ObjectType,
    pub flags: u8,
    pub reserved: [u8; 6],
    pub size: u64,
}

impl ObjectHeader {
    // possible FIXME: is this accurate?
    pub fn is_compressed(&self) -> bool {
        self.flags & OBJECT_COMPRESSED_MASK != 0
    }
}

impl SizedObject for ObjectHeader {
    fn size(&self) -> u64 {
        self.size
    }
}

/// Data objects have the actual data in the payload field
#[derive(Debug, PartialEq)]
pub struct DataObject {
    /// The object header
    pub object: ObjectHeader,
    /// A hash of the payload
    pub hash: u64,
    /// Used in cases of hash collisions
    pub next_hash_offset: u64,
    /// Links data objects with the same field
    pub next_field_offset: u64,
    /// An offset to the first entry object referencing this data
    pub entry_offset: u64,
    /// An offset to an array object with offsets to other entries that point to this data
    pub entry_array_offset: u64,
    /// Count of entry objects that point to this object
    pub n_entries: u64,
    /// The field and data. Possibly compressed if object header indicates.
    pub payload: Vec<u8>,
}

impl DataObject {
    /// Returns true if data object payload was added by by the journal and 
    /// cannot be altered by client code
    pub fn payload_is_trusted(&self) -> bool {
        0x5f == self.payload[0]
    }
}

impl HashableObject for DataObject {
    fn hash(&self) -> u64 {
        rhash64(&self.payload)
    }
}

#[derive(Debug, PartialEq)]
pub struct FieldObject {
    pub object: ObjectHeader,
    pub hash: u64,
    pub next_hash_offset: u64,
    pub head_data_offset: u64,
    pub payload: Vec<u8>,
}

impl HashableObject for FieldObject {
    fn hash(&self) -> u64 {
        rhash64(&self.payload)
    }
}

#[derive(Debug, PartialEq)]
pub struct EntryItem {
    pub object_offset: u64,
    pub hash: u64,
    pub item: Object,
}

/// Represents one log entry
#[derive(Debug, PartialEq)]
pub struct EntryObject {
    pub object: ObjectHeader,
    /// Sequence number of the entry
    pub seqnum: u64,
    /// Realtime timestamp
    pub realtime: u64,
    /// Timestamp for the boot
    pub monotonic: u64,
    /// Boot id the monotonic timestamp refers to
    pub boot_id: u128,
    /// Binary XOR of the hashes of the payload of all DATA objects in the entry
    pub xor_hash: u64,
    pub items: Vec<EntryItem>,
}

impl EntryObject {
    pub fn get_data<T: Read + Seek>(&self, key: &str, buf: &mut T) -> Option<String> {
        for item in self.items.iter() {
            let obj = get_obj_at_offset(buf, item.object_offset).ok()?;
            if let Object::Data(o) = obj {
                if key.as_bytes() == &o.payload[..key.len()] {
                    return Some(str::from_utf8(&o.payload[key.len()..]).unwrap().to_owned());
                }
            }
        }
        None
    }
}

impl HashableObject for EntryObject {
    fn hash(&self) -> u64 {
        // TODO: use for_each here?
        let mut xor_hash: u64 = 0;
        for item in &self.items {
            xor_hash = xor_hash ^ item.hash;
        }
        xor_hash
    }
}

#[derive(Debug, PartialEq)]
pub struct EntryArrayObject {
    pub object: ObjectHeader,
    pub next_entry_array_offset: u64,
    // TODO: think about creating an offset type?
    pub items: Vec<ObjectOffset>,
}

#[derive(Debug, PartialEq)]
pub struct HashItem {
    pub hash_head_offset: u64,
    pub tail_hash_offset: u64,
}

#[derive(Debug, PartialEq)]
pub struct HashTableObject {
    pub object: ObjectHeader,
    pub items: Vec<HashItem>,
}

#[derive(Debug, PartialEq)]
pub struct TagObject {
    pub object: ObjectHeader,
    pub seqnum: u64,
    pub epoch: u64,
    pub tag: [u8; TAG_LENGTH], /* SHA-256 HMAC */
}

pub union SdId128 {
    pub bytes: [u8; 16],
    pub qwords: [u64; 2],
}

pub struct JournalHeader {
    pub signature: [u8; 8],
    pub compatible_flags: u32,
    pub incompatible_flags: u32,
    pub state: JournalState,
    pub reserved: [u8; 7],
    pub file_id: u128,
    pub machine_id: u128,
    pub boot_id: u128,
    pub seqnum_id: u128,
    pub header_size: u64,
    pub arena_size: u64,
    pub data_hash_table_offset: u64,
    pub data_hash_table_size: u64,
    pub field_hash_table_offset: u64,
    pub field_hash_table_size: u64,
    pub tail_object_offset: u64,
    pub n_objects: u64,
    pub n_entries: u64,
    pub tail_entry_seqnum: u64,
    pub head_entry_seqnum: u64,
    pub entry_array_offset: u64,
    pub head_entry_realtime: u64,
    pub tail_entry_realtime: u64,
    pub tail_entry_monotonic: u64,
    /* Added in 187 */
    pub n_data: u64,
    pub n_fields: u64,
    /* Added in 189 */
    pub n_tags: u64,
    pub n_entry_arrays: u64,
}

impl JournalHeader {
    pub fn new<T: Read + Seek>(mut file: T) -> Result<JournalHeader> {
        //let mut file = Cursor::new(file);
        let mut signature = [0u8; 8];
        file.read_exact(&mut signature)?;
        let compatible_flags = file.read_u32::<LittleEndian>()?;
        let incompatible_flags = file.read_u32::<LittleEndian>()?;
        let state = file.read_u8()?;
        let state = match state {
            0 => JournalState::Offline,
            1 => JournalState::Online,
            2 => JournalState::Archived,
            _ => JournalState::StateMax,
        };
        let mut reserved = [0u8; 7];
        file.read_exact(&mut reserved)?;
        let file_id = file.read_u128::<BigEndian>()?;
        let machine_id = file.read_u128::<BigEndian>()?;
        let boot_id = file.read_u128::<BigEndian>()?;
        let seqnum_id = file.read_u128::<BigEndian>()?;
        let header_size = file.read_u64::<LittleEndian>()?;
        let arena_size = file.read_u64::<LittleEndian>()?;
        let data_hash_table_offset = file.read_u64::<LittleEndian>()?;
        let data_hash_table_size = file.read_u64::<LittleEndian>()?;
        let field_hash_table_offset = file.read_u64::<LittleEndian>()?;
        let field_hash_table_size = file.read_u64::<LittleEndian>()?;
        let tail_object_offset = file.read_u64::<LittleEndian>()?;
        let n_objects = file.read_u64::<LittleEndian>()?;
        let n_entries = file.read_u64::<LittleEndian>()?;
        let tail_entry_seqnum = file.read_u64::<LittleEndian>()?;
        let head_entry_seqnum = file.read_u64::<LittleEndian>()?;
        let entry_array_offset = file.read_u64::<LittleEndian>()?;
        let head_entry_realtime = file.read_u64::<LittleEndian>()?;
        let tail_entry_realtime = file.read_u64::<LittleEndian>()?;
        let tail_entry_monotonic = file.read_u64::<LittleEndian>()?;
        let n_data = file.read_u64::<LittleEndian>()?;
        let n_fields = file.read_u64::<LittleEndian>()?;
        let n_tags = file.read_u64::<LittleEndian>()?;
        let n_entry_arrays = file.read_u64::<LittleEndian>()?;

        Ok(JournalHeader {
            signature: signature,
            compatible_flags: compatible_flags,
            incompatible_flags: incompatible_flags,
            state: state,
            reserved: reserved,
            file_id: file_id,
            machine_id: machine_id,
            boot_id: boot_id,
            seqnum_id: seqnum_id,
            header_size: header_size,
            arena_size: arena_size,
            data_hash_table_offset: data_hash_table_offset,
            data_hash_table_size: data_hash_table_size,
            field_hash_table_offset: field_hash_table_offset,
            field_hash_table_size: field_hash_table_size,
            tail_object_offset: tail_object_offset,
            n_objects: n_objects,
            n_entries: n_entries,
            tail_entry_seqnum: tail_entry_seqnum,
            head_entry_seqnum: head_entry_seqnum,
            entry_array_offset: entry_array_offset,
            head_entry_realtime: head_entry_realtime,
            tail_entry_realtime: tail_entry_realtime,
            tail_entry_monotonic: tail_entry_monotonic,
            /* Added in 187 */
            n_data: n_data,
            n_fields: n_fields,
            /* Added in 189 */
            n_tags: n_tags,
            n_entry_arrays: n_entry_arrays,
        })
    }
}

impl fmt::Display for JournalHeader {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let out = format!(
            "File ID: {:x}\nMachine ID: {:x}\nBoot ID: {:x}\nSequential Number ID: {:x}\n
            State: {}\nCompatible Flags: {}\nIncompatible Flags: {}\nHeader size: {}\n\
            Arena size: {}\nData Hash Table Size: {}\nField Hash Table Size: {}\n\
            Head Sequential Number: {}\nTail Sequential Number: {}\nHead Realtime Timestamp: {}\n\
            Tail Realtime Timestamp: {}\nTail Monotonic Timestamp: {}\nObjects: {}\nEntry Objects: {}\n\
            Data Objects: {}\nField Objects: {}\nTag Objects: {}\nEntry Array Objects: {}",
            self.file_id, self.machine_id, self.boot_id, self.seqnum_id, self.state, self.compatible_flags,
            self.incompatible_flags, self.header_size, self.arena_size, self.data_hash_table_size,
            self.field_hash_table_size, self.head_entry_seqnum, self.tail_entry_seqnum, self.head_entry_realtime,
            self.tail_entry_realtime, self.tail_entry_monotonic, self.n_objects, self.n_entries, self.n_fields,
            self.n_data, self.n_tags, self.n_entry_arrays
        );
        write!(f, "{}", out)
    }
}
