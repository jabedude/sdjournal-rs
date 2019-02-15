#[repr(C, packed)]
pub struct ObjectHeader {
    type_: u8,
    flags: u8,
    reserved: [u8; 6],
    size: u64,
    payload: Vec<u8>,
}

#[repr(C, packed)]
pub struct DataObject {
    object: ObjectHeader,
    hash: u64,
    next_hash_offset: u64,
    next_field_offset: u64,
    entry_offset: u64,
    entry_array_offset: u64,
    n_entries: u64,
    payload: Vec<u8>,
}

#[repr(C, packed)]
pub struct FieldObject {
    object: ObjectHeader,
    hash: u64,
    next_hash_offset: u64,
    head_data_offset: u64,
    payload: Vec<u8>,
}

#[repr(C, packed)]
pub struct EntryItem {
    object_offset: u64,
    hash: u64,
}

#[repr(C, packed)]
pub struct EntryObject {
    object: ObjectHeader,
    seqnum: u64,
    realtime: u64,
    monotonic: u64,
    boot_id: sd_id128,
    xor_hash: u64,
    items: Vec<EntryItem>,
}

#[repr(C, packed)]
pub struct HashItem {
    hash_head_offset: u64,
    tail_hash_offset: u64,
}

#[repr(C, packed)]
pub struct HashTableObject {
    object: ObjectHeader,
    items: Vec<HashItem>,
}

pub union sd_id128 {
    pub bytes: [u8; 16],
    pub qwords: [u64; 2],
}

pub struct JournalHeader {
        pub signature: [u8; 8],
        pub compatible_flags: u32,
        pub incompatible_flags: u32,
        pub state: u8,
        pub reserved: [u8; 7],
        pub file_id: sd_id128,
        pub machine_id: sd_id128,
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
