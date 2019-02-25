#![feature(untagged_unions)]
use std::fs::File;
use std::mem;
use byteorder::{BigEndian, LittleEndian, ReadBytesExt};
use std::io::{Read, Result, Error, ErrorKind, Seek, SeekFrom};

pub mod journal;
pub use crate::journal::*;

fn is_valid64(u: u64) -> bool {
    u & 7 == 0
}

fn align64(u: u64) -> u64 {
    (u + 7u64) & !7u64
}

fn next_obj_offset<T: SizedObject>(mut file: &File, obj: &T) -> Option<u64> {
    let curr = file.seek(SeekFrom::Current(0)).unwrap();
    let offset = align64(curr + obj.size());
    Some(offset)
}

fn next_obj_header_offset<T: SizedObject>(mut file: &File, obj: &T) -> Option<u64> {
    let curr = file.seek(SeekFrom::Current(0)).unwrap();
    let offset = align64(curr + obj.size() - OBJECT_HEADER_SZ);
    Some(offset)
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

    if !is_valid64(offset) {
        return Err(Error::new(ErrorKind::Other, "Invalid offset"));
    }


    file.seek(SeekFrom::Start(offset))?;
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

    Ok(ObjectHeader{
        type_: type_,
        flags: flags,
        reserved: reserved,
        size: size,
    })
}

pub fn load_obj_at_offset(mut file: &File, offset: u64) -> Result<Object> {

    if !is_valid64(offset) {
        return Err(Error::new(ErrorKind::Other, "Invalid offset"));
    }

    file.seek(SeekFrom::Start(offset))?;
    let type_ = file.read_u8()?;
    match type_ {
        0 => return Err(Error::new(ErrorKind::Other, "Unused Object")),
        1 => {
            println!("data object");
            let flags = file.read_u8()?;
            let mut reserved = [0u8; 6];
            file.read_exact(&mut reserved)?;
            let size = file.read_u64::<LittleEndian>()?;
            let obj_header = ObjectHeader {
                type_: ObjectType::OBJECT_DATA,
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
            let mut payload: Vec<u8> = vec![0u8; (size - 48) as usize];
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
            return Ok(Object::data(data_object));
        },
        2 => {
            println!("field object");
            let flags = file.read_u8()?;
            let mut reserved = [0u8; 6];
            file.read_exact(&mut reserved)?;
            let size = file.read_u64::<LittleEndian>()?;
            let object_header = ObjectHeader{
                type_: ObjectType::OBJECT_FIELD,
                flags: flags,
                reserved: reserved,
                size: size,
            };
            let hash = file.read_u64::<LittleEndian>()?;
            let next_hash_offset = file.read_u64::<LittleEndian>()?;
            let head_data_offset = file.read_u64::<LittleEndian>()?;
            let head_data_offset = file.read_u64::<LittleEndian>()?;
            let mut payload: Vec<u8> = vec![0u8; (size - 24) as usize];
            file.read_exact(&mut payload)?;

            let field_object = FieldObject {
                object: object_header,
                hash: hash,
                next_hash_offset: head_data_offset,
                head_data_offset: head_data_offset,
                payload: payload,
            };
            return Ok(Object::field(field_object));
        },
        3 => {
            println!("entry object");
            let flags = file.read_u8()?;
            let mut reserved = [0u8; 6];
            file.read_exact(&mut reserved)?;
            let size = file.read_u64::<LittleEndian>()?;
            let object_header = ObjectHeader{
                type_: ObjectType::OBJECT_ENTRY,
                flags: flags,
                reserved: reserved,
                size: size,
            };
            let seqnum = file.read_u64::<LittleEndian>()?;
            let realtime = file.read_u64::<LittleEndian>()?;
            let monotonic = file.read_u64::<LittleEndian>()?;
            let boot_id = file.read_u128::<LittleEndian>()?;
            let xor_hash = file.read_u64::<LittleEndian>()?;
            let mut items: Vec<EntryItem> = Vec::new();
            for _ in 0..((size - 48) / 16) {
                let object_offset = file.read_u64::<LittleEndian>()?;
                let hash = file.read_u64::<LittleEndian>()?;
                let item = EntryItem {
                    object_offset: object_offset,
                    hash: hash,
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
            return Ok(Object::entry(entry_object));
        },
        4 => {
            println!("data hash table object");
            let flags = file.read_u8()?;
            let mut reserved = [0u8; 6];
            file.read_exact(&mut reserved)?;
            let size = file.read_u64::<LittleEndian>()?;
            return Ok(Object::object(ObjectHeader{
                type_: ObjectType::OBJECT_DATA_HASH_TABLE,
                flags: flags,
                reserved: reserved,
                size: size,
            }));
        },
        5 => {
            println!("field hash table object");
            let flags = file.read_u8()?;
            let mut reserved = [0u8; 6];
            file.read_exact(&mut reserved)?;
            let size = file.read_u64::<LittleEndian>()?;
            let object_header = ObjectHeader{
                type_: ObjectType::OBJECT_FIELD_HASH_TABLE,
                flags: flags,
                reserved: reserved,
                size: size,
            };
            let mut items: Vec<HashItem> = Vec::new();
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
            return Ok(Object::hash_table(hash_table));
        },
        6 => {
            println!("entry array object");
            let flags = file.read_u8()?;
            let mut reserved = [0u8; 6];
            file.read_exact(&mut reserved)?;
            let size = file.read_u64::<LittleEndian>()?;
            return Ok(Object::object(ObjectHeader{
                type_: ObjectType::OBJECT_ENTRY_ARRAY,
                flags: flags,
                reserved: reserved,
                size: size,
            }));
        },
        7 => {
            println!("tag object");
            let flags = file.read_u8()?;
            let mut reserved = [0u8; 6];
            file.read_exact(&mut reserved)?;
            let size = file.read_u64::<LittleEndian>()?;
            return Ok(Object::object(ObjectHeader{
                type_: ObjectType::OBJECT_TAG,
                flags: flags,
                reserved: reserved,
                size: size,
            }));
        },
        _ => return Err(Error::new(ErrorKind::Other, "Unused MAX Object")),
    }
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
        journal.file.seek(SeekFrom::Start(journal.header.field_hash_table_offset - OBJECT_HEADER_SZ))?;
        let offset = journal.header.field_hash_table_offset - OBJECT_HEADER_SZ;

        Ok(ObjectHeaderIter {
            journal: journal,
            current_offset: offset,
            next_offset: offset,
        })
    }
}

pub struct ObjectHeaderIter<'a> {
    journal: &'a Journal,
    pub current_offset: u64,
    next_offset: u64,
}

impl<'a> Iterator for ObjectHeaderIter<'a> {
    type Item = ObjectHeader;

    fn next(&mut self) -> Option<ObjectHeader> {
        println!("next offset: {}", self.next_offset);
        let header = load_obj_header_at_offset(&self.journal.file, self.next_offset);
        self.current_offset = self.next_offset;
        match header {
            Ok(h) => {
                self.next_offset = next_obj_header_offset(&self.journal.file, &h)?;
                return Some(h);
            },
            Err(_) => return None,
        }
    }
}

impl<'a> ObjectIter<'a> {
    pub fn new(mut journal: &'a mut Journal) -> Result<ObjectIter> {
        journal.file.seek(SeekFrom::Start(journal.header.field_hash_table_offset - OBJECT_HEADER_SZ))?;
        let offset = journal.header.field_hash_table_offset - OBJECT_HEADER_SZ;

        Ok(ObjectIter {
            journal: journal,
            current_offset: offset,
            next_offset: offset,
        })
    }
}

pub struct ObjectIter<'a> {
    journal: &'a mut Journal,
    pub current_offset: u64,
    next_offset: u64,
}

impl<'a> Iterator for ObjectIter<'a> {
    type Item = Object;

    fn next(&mut self) -> Option<Object> {
        println!("next offset: {}", self.next_offset);
        let object = load_obj_at_offset(&self.journal.file, self.next_offset);
        self.current_offset = self.next_offset;
        self.journal.file.seek(SeekFrom::Start(self.current_offset)).unwrap();
        match object {
            Ok(o) => {
                self.next_offset = next_obj_offset(&self.journal.file, &o)?;
                return Some(o);
            },
            Err(e) => {
                return None;
            }
        }
    }
}
