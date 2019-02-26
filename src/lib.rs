#![feature(untagged_unions)]
use std::fs::File;
use std::io::Cursor;
use byteorder::{LittleEndian, ReadBytesExt};
use std::io::{Read, Result, Error, ErrorKind, Seek, SeekFrom};

pub mod journal;
pub use crate::journal::*;

fn is_valid64(u: u64) -> bool {
    u & 7 == 0
}

fn align64(u: u64) -> u64 {
    (u + 7u64) & !7u64
}

fn next_obj_offset<T: SizedObject>(mut file: &mut Cursor<&[u8]>, obj: &T) -> Option<u64> {
    let curr = file.seek(SeekFrom::Current(0)).unwrap();
    let offset = align64(curr + obj.size());
    Some(offset)
}

fn next_obj_header_offset<T: SizedObject>(mut file: &mut Cursor<&[u8]>, obj: &T) -> Option<u64> {
    let curr = file.seek(SeekFrom::Current(0)).unwrap();
    let offset = align64(curr + obj.size() - OBJECT_HEADER_SZ);
    Some(offset)
}

pub fn load_header(mut file: &mut Cursor<&[u8]>) -> Result<JournalHeader> {
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

pub fn load_obj_header_at_offset(file: &mut Cursor<&[u8]>, offset: u64) -> Result<ObjectHeader> {

    if !is_valid64(offset) {
        return Err(Error::new(ErrorKind::Other, "Invalid offset"));
    }


    file.seek(SeekFrom::Start(offset))?;
    let type_ = file.read_u8()?;
    let type_ = match type_ {
        0 => ObjectType::ObjectUnused,
        1 => ObjectType::ObjectData,
        2 => ObjectType::ObjectField,
        3 => ObjectType::ObjectEntry,
        4 => ObjectType::ObjectDataHashTable,
        5 => ObjectType::ObjectFieldHashTable,
        6 => ObjectType::ObjectEntryArray,
        7 => ObjectType::ObjectTag,
        _ => ObjectType::ObjectTypeMax
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

pub fn load_obj_at_offset(file: &mut Cursor<&[u8]>, offset: u64) -> Result<Object> {

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
            return Ok(Object::Data(data_object));
        },
        2 => {
            let flags = file.read_u8()?;
            let mut reserved = [0u8; 6];
            file.read_exact(&mut reserved)?;
            let size = file.read_u64::<LittleEndian>()?;
            let object_header = ObjectHeader{
                type_: ObjectType::ObjectField,
                flags: flags,
                reserved: reserved,
                size: size,
            };
            let hash = file.read_u64::<LittleEndian>()?;
            let next_hash_offset = file.read_u64::<LittleEndian>()?;
            let head_data_offset = file.read_u64::<LittleEndian>()?;
            let mut payload: Vec<u8> = vec![0u8; (size - 24) as usize];
            file.read_exact(&mut payload)?;

            let field_object = FieldObject {
                object: object_header,
                hash: hash,
                next_hash_offset: next_hash_offset,
                head_data_offset: head_data_offset,
                payload: payload,
            };
            return Ok(Object::Field(field_object));
        },
        3 => {
            let flags = file.read_u8()?;
            let mut reserved = [0u8; 6];
            file.read_exact(&mut reserved)?;
            let size = file.read_u64::<LittleEndian>()?;
            let object_header = ObjectHeader{
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
            let mut items: Vec<EntryItem> = Vec::new();
            for _ in 1..((size - 48) / 16) {
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
            return Ok(Object::Entry(entry_object));
        },
        4 => {
            let flags = file.read_u8()?;
            let mut reserved = [0u8; 6];
            file.read_exact(&mut reserved)?;
            let size = file.read_u64::<LittleEndian>()?;
            let object_header = ObjectHeader{
                type_: ObjectType::ObjectDataHashTable,
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
            return Ok(Object::HashTable(hash_table));
        },
        5 => {
            let flags = file.read_u8()?;
            let mut reserved = [0u8; 6];
            file.read_exact(&mut reserved)?;
            let size = file.read_u64::<LittleEndian>()?;
            let object_header = ObjectHeader{
                type_: ObjectType::ObjectFieldHashTable,
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
            return Ok(Object::HashTable(hash_table));
        },
        6 => {
            let flags = file.read_u8()?;
            let mut reserved = [0u8; 6];
            file.read_exact(&mut reserved)?;
            let size = file.read_u64::<LittleEndian>()?;
            let object_header = ObjectHeader{
                type_: ObjectType::ObjectEntryArray,
                flags: flags,
                reserved: reserved,
                size: size,
            };
            let next_entry_array_offset = file.read_u64::<LittleEndian>()?;
            let mut items: Vec<u64> = Vec::new();
            for _ in 0..((size - 48) / 8) {
                let item = file.read_u64::<LittleEndian>()?;
                items.push(item);
            }
            let entry_array_object = EntryArrayObject {
                object: object_header,
                next_entry_array_offset: next_entry_array_offset,
                items: items,
            };
            return Ok(Object::EntryArray(entry_array_object));
        },
        7 => {
            let flags = file.read_u8()?;
            let mut reserved = [0u8; 6];
            file.read_exact(&mut reserved)?;
            let size = file.read_u64::<LittleEndian>()?;
            let object_header = ObjectHeader{
                type_: ObjectType::ObjectTag,
                flags: flags,
                reserved: reserved,
                size: size,
            };
            let seqnum = file.read_u64::<LittleEndian>()?;
            let epoch = file.read_u64::<LittleEndian>()?;
            let mut tag = [0u8; 256/8];
            file.read_exact(&mut tag)?;
            let tag_object = TagObject {
                object: object_header,
                seqnum: seqnum,
                epoch: epoch,
                tag: tag, /* SHA-256 HMAC */
            };
            return Ok(Object::Tag(tag_object));
        },
        _ => return Err(Error::new(ErrorKind::Other, "Unused MAX Object")),
    }
}

pub fn get_obj_at_offset(file: &[u8], offset: u64) -> Result<Object> {

    if !is_valid64(offset) {
        return Err(Error::new(ErrorKind::Other, "Invalid offset"));
    }
    let mut file = Cursor::new(file);

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
            return Ok(Object::Data(data_object));
        },
        2 => {
            let flags = file.read_u8()?;
            let mut reserved = [0u8; 6];
            file.read_exact(&mut reserved)?;
            let size = file.read_u64::<LittleEndian>()?;
            let object_header = ObjectHeader{
                type_: ObjectType::ObjectField,
                flags: flags,
                reserved: reserved,
                size: size,
            };
            let hash = file.read_u64::<LittleEndian>()?;
            let next_hash_offset = file.read_u64::<LittleEndian>()?;
            let head_data_offset = file.read_u64::<LittleEndian>()?;
            let mut payload: Vec<u8> = vec![0u8; (size - 24) as usize];
            file.read_exact(&mut payload)?;

            let field_object = FieldObject {
                object: object_header,
                hash: hash,
                next_hash_offset: next_hash_offset,
                head_data_offset: head_data_offset,
                payload: payload,
            };
            return Ok(Object::Field(field_object));
        },
        3 => {
            let flags = file.read_u8()?;
            let mut reserved = [0u8; 6];
            file.read_exact(&mut reserved)?;
            let size = file.read_u64::<LittleEndian>()?;
            let object_header = ObjectHeader{
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
            let mut items: Vec<EntryItem> = Vec::new();
            for _ in 1..((size - 48) / 16) {
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
            return Ok(Object::Entry(entry_object));
        },
        4 => {
            let flags = file.read_u8()?;
            let mut reserved = [0u8; 6];
            file.read_exact(&mut reserved)?;
            let size = file.read_u64::<LittleEndian>()?;
            let object_header = ObjectHeader{
                type_: ObjectType::ObjectDataHashTable,
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
            return Ok(Object::HashTable(hash_table));
        },
        5 => {
            let flags = file.read_u8()?;
            let mut reserved = [0u8; 6];
            file.read_exact(&mut reserved)?;
            let size = file.read_u64::<LittleEndian>()?;
            let object_header = ObjectHeader{
                type_: ObjectType::ObjectFieldHashTable,
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
            return Ok(Object::HashTable(hash_table));
        },
        6 => {
            let flags = file.read_u8()?;
            let mut reserved = [0u8; 6];
            file.read_exact(&mut reserved)?;
            let size = file.read_u64::<LittleEndian>()?;
            let object_header = ObjectHeader{
                type_: ObjectType::ObjectEntryArray,
                flags: flags,
                reserved: reserved,
                size: size,
            };
            let next_entry_array_offset = file.read_u64::<LittleEndian>()?;
            let mut items: Vec<u64> = Vec::new();
            for _ in 0..((size - 48) / 8) {
                let item = file.read_u64::<LittleEndian>()?;
                items.push(item);
            }
            let entry_array_object = EntryArrayObject {
                object: object_header,
                next_entry_array_offset: next_entry_array_offset,
                items: items,
            };
            return Ok(Object::EntryArray(entry_array_object));
        },
        7 => {
            let flags = file.read_u8()?;
            let mut reserved = [0u8; 6];
            file.read_exact(&mut reserved)?;
            let size = file.read_u64::<LittleEndian>()?;
            let object_header = ObjectHeader{
                type_: ObjectType::ObjectTag,
                flags: flags,
                reserved: reserved,
                size: size,
            };
            let seqnum = file.read_u64::<LittleEndian>()?;
            let epoch = file.read_u64::<LittleEndian>()?;
            let mut tag = [0u8; 256/8];
            file.read_exact(&mut tag)?;
            let tag_object = TagObject {
                object: object_header,
                seqnum: seqnum,
                epoch: epoch,
                tag: tag, /* SHA-256 HMAC */
            };
            return Ok(Object::Tag(tag_object));
        },
        _ => return Err(Error::new(ErrorKind::Other, "Unused MAX Object")),
    }
}

impl<'a> Journal<'a> {
    pub fn new(mut path: Cursor<&'a [u8]>) -> Result<Journal<'a>> {
        //TODO: mmap file
        let header = load_header(&mut path)?;

        Ok(Journal{
            file: path,
            header: header,
        })
    }

    pub fn load_obj_at_offset(&mut self, offset: u64) -> Result<Object> {

        if !is_valid64(offset) {
            return Err(Error::new(ErrorKind::Other, "Invalid offset"));
        }

        self.file.seek(SeekFrom::Start(offset))?;
        let type_ = self.file.read_u8()?;
        match type_ {
            0 => return Err(Error::new(ErrorKind::Other, "Unused Object")),
            1 => {
                let flags = self.file.read_u8()?;
                let mut reserved = [0u8; 6];
                self.file.read_exact(&mut reserved)?;
                let size = self.file.read_u64::<LittleEndian>()?;
                let obj_header = ObjectHeader {
                    type_: ObjectType::ObjectData,
                    flags: flags,
                    reserved: reserved,
                    size: size,
                };
                let hash = self.file.read_u64::<LittleEndian>()?;
                let next_hash_offset = self.file.read_u64::<LittleEndian>()?;
                let next_field_offset = self.file.read_u64::<LittleEndian>()?;
                let entry_offset = self.file.read_u64::<LittleEndian>()?;
                let entry_array_offset = self.file.read_u64::<LittleEndian>()?;
                let n_entries = self.file.read_u64::<LittleEndian>()?;
                let mut payload: Vec<u8> = vec![0u8; (size - 48) as usize];
                self.file.read_exact(&mut payload)?;
                
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
            },
            2 => {
                let flags = self.file.read_u8()?;
                let mut reserved = [0u8; 6];
                self.file.read_exact(&mut reserved)?;
                let size = self.file.read_u64::<LittleEndian>()?;
                let object_header = ObjectHeader{
                    type_: ObjectType::ObjectField,
                    flags: flags,
                    reserved: reserved,
                    size: size,
                };
                let hash = self.file.read_u64::<LittleEndian>()?;
                let next_hash_offset = self.file.read_u64::<LittleEndian>()?;
                let head_data_offset = self.file.read_u64::<LittleEndian>()?;
                let mut payload: Vec<u8> = vec![0u8; (size - 24) as usize];
                self.file.read_exact(&mut payload)?;

                let field_object = FieldObject {
                    object: object_header,
                    hash: hash,
                    next_hash_offset: next_hash_offset,
                    head_data_offset: head_data_offset,
                    payload: payload,
                };
                return Ok(Object::Field(field_object));
            },
            3 => {
                let flags = self.file.read_u8()?;
                let mut reserved = [0u8; 6];
                self.file.read_exact(&mut reserved)?;
                let size = self.file.read_u64::<LittleEndian>()?;
                let object_header = ObjectHeader{
                    type_: ObjectType::ObjectEntry,
                    flags: flags,
                    reserved: reserved,
                    size: size,
                };
                let seqnum = self.file.read_u64::<LittleEndian>()?;
                let realtime = self.file.read_u64::<LittleEndian>()?;
                let monotonic = self.file.read_u64::<LittleEndian>()?;
                let boot_id = self.file.read_u128::<LittleEndian>()?;
                let xor_hash = self.file.read_u64::<LittleEndian>()?;
                let mut items: Vec<EntryItem> = Vec::new();
                for _ in 0..((size - 48) / 16) {
                    let object_offset = self.file.read_u64::<LittleEndian>()?;
                    let hash = self.file.read_u64::<LittleEndian>()?;
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
                return Ok(Object::Entry(entry_object));
            },
            4 => {
                let flags = self.file.read_u8()?;
                let mut reserved = [0u8; 6];
                self.file.read_exact(&mut reserved)?;
                let size = self.file.read_u64::<LittleEndian>()?;
                let object_header = ObjectHeader{
                    type_: ObjectType::ObjectDataHashTable,
                    flags: flags,
                    reserved: reserved,
                    size: size,
                };
                let mut items: Vec<HashItem> = Vec::new();
                for _ in 0..((size - 48) / 16) {
                    let hash_head_offset = self.file.read_u64::<LittleEndian>()?;
                    let tail_hash_offset = self.file.read_u64::<LittleEndian>()?;
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
            },
            5 => {
                let flags = self.file.read_u8()?;
                let mut reserved = [0u8; 6];
                self.file.read_exact(&mut reserved)?;
                let size = self.file.read_u64::<LittleEndian>()?;
                let object_header = ObjectHeader{
                    type_: ObjectType::ObjectFieldHashTable,
                    flags: flags,
                    reserved: reserved,
                    size: size,
                };
                let mut items: Vec<HashItem> = Vec::new();
                for _ in 0..((size - 48) / 16) {
                    let hash_head_offset = self.file.read_u64::<LittleEndian>()?;
                    let tail_hash_offset = self.file.read_u64::<LittleEndian>()?;
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
            },
            6 => {
                let flags = self.file.read_u8()?;
                let mut reserved = [0u8; 6];
                self.file.read_exact(&mut reserved)?;
                let size = self.file.read_u64::<LittleEndian>()?;
                let object_header = ObjectHeader{
                    type_: ObjectType::ObjectEntryArray,
                    flags: flags,
                    reserved: reserved,
                    size: size,
                };
                let next_entry_array_offset = self.file.read_u64::<LittleEndian>()?;
                let mut items: Vec<u64> = Vec::new();
                for _ in 0..((size - 48) / 8) {
                    let item = self.file.read_u64::<LittleEndian>()?;
                    items.push(item);
                }
                let entry_array_object = EntryArrayObject {
                    object: object_header,
                    next_entry_array_offset: next_entry_array_offset,
                    items: items,
                };
                return Ok(Object::EntryArray(entry_array_object));
            },
            7 => {
                let flags = self.file.read_u8()?;
                let mut reserved = [0u8; 6];
                self.file.read_exact(&mut reserved)?;
                let size = self.file.read_u64::<LittleEndian>()?;
                let object_header = ObjectHeader{
                    type_: ObjectType::ObjectTag,
                    flags: flags,
                    reserved: reserved,
                    size: size,
                };
                let seqnum = self.file.read_u64::<LittleEndian>()?;
                let epoch = self.file.read_u64::<LittleEndian>()?;
                let mut tag = [0u8; 256/8];
                self.file.read_exact(&mut tag)?;
                let tag_object = TagObject {
                    object: object_header,
                    seqnum: seqnum,
                    epoch: epoch,
                    tag: tag, /* SHA-256 HMAC */
                };
                return Ok(Object::Tag(tag_object));
            },
            _ => return Err(Error::new(ErrorKind::Other, "Unused MAX Object")),
        }
    }
}

impl<'a> ObjectHeaderIter<'a> {
    pub fn new(journal: &'a mut Journal<'a>) -> Result<ObjectHeaderIter<'a>> {
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
    journal: &'a mut Journal<'a>,
    pub current_offset: u64,
    next_offset: u64,
}

impl<'a> Iterator for ObjectHeaderIter<'a> {
    type Item = ObjectHeader;

    fn next(&mut self) -> Option<ObjectHeader> {
        let header = load_obj_header_at_offset(&mut self.journal.file, self.next_offset);
        self.current_offset = self.next_offset;
        match header {
            Ok(h) => {
                self.next_offset = next_obj_header_offset(&mut self.journal.file, &h)?;
                return Some(h);
            },
            Err(_) => return None,
        }
    }
}

impl<'a> ObjectIter<'a> {
    pub fn new(journal: &'a mut Journal<'a>) -> Result<ObjectIter<'a>> {
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
    journal: &'a mut Journal<'a>,
    pub current_offset: u64,
    next_offset: u64,
}

impl<'a> Iterator for ObjectIter<'a> {
    type Item = Object;

    fn next(&mut self) -> Option<Object> {
        let object = load_obj_at_offset(&mut self.journal.file, self.next_offset);
        self.current_offset = self.next_offset;
        self.journal.file.seek(SeekFrom::Start(self.current_offset)).unwrap();
        match object {
            Ok(o) => {
                self.next_offset = next_obj_offset(&mut self.journal.file, &o)?;
                return Some(o);
            },
            Err(_) => {
                return None;
            }
        }
    }
}
