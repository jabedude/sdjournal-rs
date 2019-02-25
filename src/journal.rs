use std::fs::File;

pub const OBJECT_HEADER_SZ: u64 = 16;
pub const DATA_OBJECT_HEADER_SZ: u64 = 48;
pub const FIELD_OBJECT_HEADER_SZ: u64 = 24;

/// This trait guarantees an object that implements it can return it's
/// own size.
pub trait SizedObject {
    fn size(&self) -> u64;
}

pub enum Object {
    object(ObjectHeader),
    data(DataObject),
    field(FieldObject),
    entry(EntryObject),
    hash_table(HashTableObject),
    entry_array(EntryArrayObject),
    tag(TagObject),
}

impl SizedObject for Object {
    fn size(&self) -> u64 {
        match self {
            Object::object(o) => return o.size,
            Object::data(d) => return d.object.size,
            Object::field(f) => return f.object.size,
            Object::entry(e) => return e.object.size,
            Object::hash_table(ht) => return ht.object.size,
            Object::entry_array(ea) => return ea.object.size,
            Object::tag(t) => return t.object.size,
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum ObjectType {
    OBJECT_UNUSED = 0,
    OBJECT_DATA = 1,
    OBJECT_FIELD = 2,
    OBJECT_ENTRY = 3,
    OBJECT_DATA_HASH_TABLE = 4,
    OBJECT_FIELD_HASH_TABLE = 5,
    OBJECT_ENTRY_ARRAY = 6,
    OBJECT_TAG = 7,
    _OBJECT_TYPE_MAX
}

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

pub struct FieldObject {
    pub object: ObjectHeader,
    pub hash: u64,
    pub next_hash_offset: u64,
    pub head_data_offset: u64,
    pub payload: Vec<u8>,
}

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
    pub tag: [u8; 256/8], /* SHA-256 HMAC */
}

pub union SdId128 {
    pub bytes: [u8; 16],
    pub qwords: [u64; 2],
}

#[repr(C, packed)]
pub struct JournalHeader {
    pub signature: [u8; 8],
    pub compatible_flags: u32,
    pub incompatible_flags: u32,
    pub state: u8,
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
    pub field_hash_table_size : u64,
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

pub struct Journal {
    pub file: File,
    pub header: JournalHeader,
}
