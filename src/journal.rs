use byteorder::{LittleEndian, ReadBytesExt};
use std::fmt;
use std::io::Cursor;
use std::io::{Read, Result};

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
pub struct ObjectHeader {
    pub type_: ObjectType,
    pub flags: u8,
    pub reserved: [u8; 6],
    pub size: u64,
}

impl SizedObject for ObjectHeader {
    fn size(&self) -> u64 {
        self.size
    }
}

/// Data objects have the actual data in the payload field
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
}

/// Represents one log entry
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

pub struct EntryArrayObject {
    pub object: ObjectHeader,
    pub next_entry_array_offset: u64,
    // TODO: think about creating an offset type?
    pub items: Vec<ObjectOffset>,
}

pub struct HashItem {
    pub hash_head_offset: u64,
    pub tail_hash_offset: u64,
}

pub struct HashTableObject {
    pub object: ObjectHeader,
    pub items: Vec<HashItem>,
}

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
    pub fn new(file: &[u8]) -> Result<JournalHeader> {
        let mut file = Cursor::new(file);
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
        let file_id = file.read_u128::<LittleEndian>()?;
        let machine_id = file.read_u128::<LittleEndian>()?;
        let boot_id = file.read_u128::<LittleEndian>()?;
        let seqnum_id = file.read_u128::<LittleEndian>()?;
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
            "File ID: {:x}\nMachine ID: {:x}\nBoot ID: {}\nSequential Number ID: {}\
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
