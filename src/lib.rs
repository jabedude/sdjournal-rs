use std::fs::File;
use std::mem;
use byteorder::{BigEndian, LittleEndian, ReadBytesExt};
use std::io::{Read, Result, Seek, SeekFrom};

pub mod journal;
pub use crate::journal::*;

pub fn next_obj_offset(mut file: &File, obj_header: &ObjectHeader) -> Option<u64> {
    let curr = file.seek(SeekFrom::Current(0));
    match curr {
        Ok(c) => Some(c + obj_header.size - OBJECT_HEADER_SZ),
        Err(_) => None,
    }
}

pub fn load_header(mut file: &File) -> Result<JournalHeader> {
    let mut signature = [0u8; 8];
    file.read_exact(&mut signature)?;
    let compatible_flags = file.read_u32::<LittleEndian>()?;
    let incompatible_flags = file.read_u32::<LittleEndian>()?;
    let state = file.read_u8()?;
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
        field_hash_table_size : field_hash_table_size,
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

pub fn load_obj_header_at_offset(mut file: &File, offset: u64) -> Result<ObjectHeader> {
    println!("offset: {}", offset);
    file.seek(SeekFrom::Start(offset));
    let type_ = file.read_u8()?;
    let type_ = match type_ {
        0 => ObjectType::OBJECT_UNUSED,
        1 => ObjectType::OBJECT_DATA,
        2 => ObjectType::OBJECT_FIELD,
        3 => ObjectType::OBJECT_ENTRY,
        4 => ObjectType::OBJECT_DATA_HASH_TABLE,
        5 => ObjectType::OBJECT_FIELD_HASH_TABLE,
        6 => ObjectType::OBJECT_ENTRY_ARRAY,
        7 => ObjectType::OBJECT_TAG,
        _ => ObjectType::_OBJECT_TYPE_MAX
    };


    let flags = file.read_u8()?;
    let mut reserved = [0u8; 6];
    file.read_exact(&mut reserved)?;
    let size = file.read_u64::<LittleEndian>()?;
    let mut payload: Vec<u8> = vec![0u8; 8];
    //file.read_exact(&mut payload)?;

    Ok(ObjectHeader{
        type_: type_,
        flags: flags,
        reserved: reserved,
        size: size,
        payload: payload,
    })
}

pub fn load_object_header(mut file: &File) -> Result<ObjectHeader> {
    let type_ = file.read_u8()?;
    let type_ = match type_ {
        0 => ObjectType::OBJECT_UNUSED,
        1 => ObjectType::OBJECT_DATA,
        2 => ObjectType::OBJECT_FIELD,
        3 => ObjectType::OBJECT_ENTRY,
        4 => ObjectType::OBJECT_DATA_HASH_TABLE,
        5 => ObjectType::OBJECT_FIELD_HASH_TABLE,
        6 => ObjectType::OBJECT_ENTRY_ARRAY,
        7 => ObjectType::OBJECT_TAG,
        _ => ObjectType::_OBJECT_TYPE_MAX
    };


    let flags = file.read_u8()?;
    let mut reserved = [0u8; 6];
    file.read_exact(&mut reserved)?;
    let size = file.read_u64::<LittleEndian>()?;
    let struct_size = mem::size_of::<ObjectHeader>() as u64;
    println!("s {} size {} to read ", struct_size, size);
    let mut payload: Vec<u8> = vec![0u8; size as usize];
    file.read_exact(&mut payload)?;

    Ok(ObjectHeader{
        type_: type_,
        flags: flags,
        reserved: reserved,
        size: size,
        payload: payload,
    })
}

impl Journal {
    pub fn new(path: &str) -> Result<Journal> {
        let mut file = File::open(path)?;
        let header = load_header(&mut file)?;

        Ok(Journal{
            file: file,
            header: header,
        })
    }
}

impl<'a> ObjectHeaderIter<'a> {
    pub fn new(mut journal: &'a mut Journal) -> Result<ObjectHeaderIter> {
        journal.file.seek(SeekFrom::Start(journal.header.field_hash_table_offset));
        let offset = journal.header.field_hash_table_offset;

        Ok(ObjectHeaderIter {
            journal: journal,
            next_offset: offset,
        })
    }
}

pub struct ObjectHeaderIter<'a> {
    journal: &'a Journal,
    next_offset: u64,
}

impl<'a> Iterator for ObjectHeaderIter<'a> {
    type Item = ObjectHeader;

    fn next(&mut self) -> Option<ObjectHeader> {
        let header = load_obj_header_at_offset(&self.journal.file, self.next_offset);
        match header {
            Ok(h) => {
                self.next_offset = next_obj_offset(&self.journal.file, &h)?;
                return Some(h);
            },
            Err(_) => return None,
        }
    }
}
