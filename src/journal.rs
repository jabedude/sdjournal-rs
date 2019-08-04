use std::fmt;

// TODO: compression support
// TODO: work on entrt struct to allow for propper formatting of entries

pub const OBJECT_HEADER_SZ: u64 = 16;
pub const DATA_OBJECT_HEADER_SZ: u64 = 48;
pub const FIELD_OBJECT_HEADER_SZ: u64 = 24;

pub const OBJECT_COMPRESSED_XZ: u8 = 1 << 0;
pub const OBJECT_COMPRESSED_LZ4: u8 = 1 << 1;
pub const OBJECT_COMPRESSED_MASK: u8 = OBJECT_COMPRESSED_XZ | OBJECT_COMPRESSED_LZ4;

pub const TAG_LENGTH: usize = (256 / 8);

#[derive(PartialEq)]
pub enum JournalState {
    Offline,
    Online,
    Archived,
    StateMax,
}

/// This trait guarantees an object that implements it can return it's
/// own size.
pub trait SizedObject {
    fn size(&self) -> u64;
}

/// Represents all the possible types of objects in a journal file.
pub enum Object {
    /// Holds the common object header for any object
    Object(ObjectHeader),
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
            Object::Object(_) => write!(f, "Object header"),
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
            Object::Object(o) => return o.size,
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

pub struct DataObject {
    pub object: ObjectHeader,
    pub hash: u64,
    pub next_hash_offset: u64,
    pub next_field_offset: u64,
    pub entry_offset: u64,
    pub entry_array_offset: u64,
    pub n_entries: u64,
    pub payload: Vec<u8>,
}

impl DataObject {
    pub fn payload_is_trusted(&self) -> bool {
        0x5f == self.payload[0]
    }
}

pub struct FieldObject {
    pub object: ObjectHeader,
    pub hash: u64,
    pub next_hash_offset: u64,
    pub head_data_offset: u64,
    pub payload: Vec<u8>,
}

#[derive(Debug, PartialEq)]
pub struct EntryItem {
    pub object_offset: u64,
    pub hash: u64,
}

pub struct EntryObject {
    pub object: ObjectHeader,
    pub seqnum: u64,
    pub realtime: u64,
    pub monotonic: u64,
    pub boot_id: u128,
    pub xor_hash: u64,
    pub items: Vec<EntryItem>,
}

pub struct EntryArrayObject {
    pub object: ObjectHeader,
    pub next_entry_array_offset: u64,
    pub items: Vec<u64>,
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
    pub tag: [u8; 256 / 8], /* SHA-256 HMAC */
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

pub struct Journal<'a> {
    pub file: &'a [u8],
    pub header: JournalHeader,
}
