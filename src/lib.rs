#![feature(untagged_unions)]
use byteorder::{LittleEndian, ReadBytesExt};
use std::convert::TryInto;
use std::io::Cursor;
use std::io::{Error, ErrorKind, Read, Result, Seek, SeekFrom};
use std::str;

pub mod journal;
pub mod hash;
pub mod traits;
pub use crate::journal::*;
use crate::traits::SizedObject;

pub struct Journal<'a> {
    pub file: &'a [u8],
    pub header: JournalHeader,
}

#[inline(always)]
fn is_valid64(u: u64) -> bool {
    u & 7 == 0
}

#[inline(always)]
fn align64(u: u64) -> u64 {
    (u + 7u64) & !7u64
}

impl ObjectHeader {
    // possible FIXME: is this accurate?
    pub fn is_compressed(&self) -> bool {
        self.flags & OBJECT_COMPRESSED_MASK != 0
    }
}

pub fn get_obj_at_offset(file: &[u8], offset: u64) -> Result<Object> {
    let mut file = Cursor::new(file);

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

impl<'a> Journal<'a> {
    // TODO: add verify() method
    pub fn new(mut path: &'a [u8]) -> Result<Journal<'a>> {
        let header = JournalHeader::new(&mut path)?;

        Ok(Journal {
            file: path,
            header: header,
        })
    }

    /// Iterate over all header objects in journal
    pub fn iter_headers<'b>(&'b self) -> ObjectHeaderIter<'b> {
        let start = self.header.field_hash_table_offset - OBJECT_HEADER_SZ;
        ObjectHeaderIter::new(self.file, start)
    }

    pub fn obj_iter<'b>(&'b self) -> ObjectIter<'b> {
        let start = self.header.field_hash_table_offset - OBJECT_HEADER_SZ;
        ObjectIter::new(self.file, start)
    }

    /// Iterate over all entry objects in the journal
    pub fn iter_entries<'b>(&'b self) -> EntryIter<'b> {
        let start = self.header.field_hash_table_offset - OBJECT_HEADER_SZ;
        let n_objects = self.header.n_objects;
        EntryIter::new(self.file, start, n_objects)
    }

    pub fn ea_iter<'b>(&'b self) -> EntryArrayIter<'b> {
        let start = self.header.entry_array_offset;
        EntryArrayIter::new(self.file, start)
    }
}

impl EntryObject {
    pub fn get_data(&self, key: &str, buf: &[u8]) -> Option<String> {
        for item in self.items.iter() {
            let obj = get_obj_at_offset(buf, item.object_offset).unwrap();
            if let Object::Data(o) = obj {
                if key.as_bytes() == &o.payload[..key.len()] {
                    return Some(str::from_utf8(&o.payload[key.len()..]).unwrap().to_owned());
                }
            }
        }
        None
    }
}

pub struct ObjectHeaderIter<'a> {
    buf: Cursor<&'a [u8]>,
    next_offset: u64,
}

impl<'a> ObjectHeaderIter<'a> {
    fn new(buf: &'a [u8], start: u64) -> ObjectHeaderIter<'a> {
        let mut buf = Cursor::new(buf);
        buf.seek(SeekFrom::Start(start)).unwrap();

        ObjectHeaderIter {
            buf: buf,
            next_offset: start,
        }
    }

    fn next_obj_header_offset<T: SizedObject>(&mut self, obj: &T) -> Option<u64> {
        let curr = self.buf.seek(SeekFrom::Current(0)).unwrap();
        let offset = align64(curr + obj.size() - OBJECT_HEADER_SZ);
        Some(offset)
    }

    fn load_obj_header_at_offset(&mut self, offset: u64) -> Result<ObjectHeader> {
        if !is_valid64(offset) {
            return Err(Error::new(ErrorKind::Other, "Invalid offset"));
        }

        self.buf.seek(SeekFrom::Start(offset))?;
        let type_ = self.buf.read_u8()?;
        let type_ = match type_ {
            0 => {
                return Err(Error::new(ErrorKind::Other, "Invalid offset"));
            }
            1 => ObjectType::ObjectData,
            2 => ObjectType::ObjectField,
            3 => ObjectType::ObjectEntry,
            4 => ObjectType::ObjectDataHashTable,
            5 => ObjectType::ObjectFieldHashTable,
            6 => ObjectType::ObjectEntryArray,
            7 => ObjectType::ObjectTag,
            _ => ObjectType::ObjectTypeMax,
        };

        let flags = self.buf.read_u8()?;
        let mut reserved = [0u8; 6];
        self.buf.read_exact(&mut reserved)?;
        let size = self.buf.read_u64::<LittleEndian>()?;

        Ok(ObjectHeader {
            type_: type_,
            flags: flags,
            reserved: reserved,
            size: size,
        })
    }
}

impl<'a> Iterator for ObjectHeaderIter<'a> {
    type Item = ObjectHeader;

    fn next(&mut self) -> Option<ObjectHeader> {
        let header = self.load_obj_header_at_offset(self.next_offset);
        match header {
            Ok(h) => {
                self.next_offset = self.next_obj_header_offset(&h)?;
                return Some(h);
            }
            Err(_) => return None,
        }
    }
}

impl<'a> ObjectIter<'a> {
    fn new(buf: &'a [u8], start: u64) -> ObjectIter<'a> {
        let mut buf = Cursor::new(buf);
        buf.seek(SeekFrom::Start(start)).unwrap();

        ObjectIter {
            buf: buf,
            current_offset: start,
            next_offset: start,
        }
    }

    fn next_obj_offset<T: SizedObject>(&mut self, obj: &T) -> Option<u64> {
        let offset = align64(self.buf.position() + obj.size());
        Some(offset)
    }

    fn load_obj_at_offset(&mut self, offset: u64) -> Result<Object> {
        if !is_valid64(offset) {
            return Err(Error::new(ErrorKind::Other, "Invalid offset"));
        }

        self.buf.seek(SeekFrom::Start(offset))?;
        let type_ = self.buf.read_u8()?;
        match type_ {
            0 => return Err(Error::new(ErrorKind::Other, "Unused Object")),
            1 => {
                let flags = self.buf.read_u8()?;
                let mut reserved = [0u8; 6];
                self.buf.read_exact(&mut reserved)?;
                let size = self.buf.read_u64::<LittleEndian>()?;
                let obj_header = ObjectHeader {
                    type_: ObjectType::ObjectData,
                    flags: flags,
                    reserved: reserved,
                    size: size,
                };
                let hash = self.buf.read_u64::<LittleEndian>()?;
                let next_hash_offset = self.buf.read_u64::<LittleEndian>()?;
                let next_field_offset = self.buf.read_u64::<LittleEndian>()?;
                let entry_offset = self.buf.read_u64::<LittleEndian>()?;
                let entry_array_offset = self.buf.read_u64::<LittleEndian>()?;
                let n_entries = self.buf.read_u64::<LittleEndian>()?;
                let mut payload: Vec<u8> = vec![0u8; (size - 64) as usize];
                self.buf.read_exact(&mut payload)?;

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
                let flags = self.buf.read_u8()?;
                let mut reserved = [0u8; 6];
                self.buf.read_exact(&mut reserved)?;
                let size = self.buf.read_u64::<LittleEndian>()?;
                let object_header = ObjectHeader {
                    type_: ObjectType::ObjectField,
                    flags: flags,
                    reserved: reserved,
                    size: size,
                };
                let hash = self.buf.read_u64::<LittleEndian>()?;
                let next_hash_offset = self.buf.read_u64::<LittleEndian>()?;
                let head_data_offset = self.buf.read_u64::<LittleEndian>()?;
                let mut payload: Vec<u8> = vec![0u8; (size - 40) as usize];
                self.buf.read_exact(&mut payload)?;

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
                let flags = self.buf.read_u8()?;
                let mut reserved = [0u8; 6];
                self.buf.read_exact(&mut reserved)?;
                let size = self.buf.read_u64::<LittleEndian>()?;
                let object_header = ObjectHeader {
                    type_: ObjectType::ObjectEntry,
                    flags: flags,
                    reserved: reserved,
                    size: size,
                };
                let seqnum = self.buf.read_u64::<LittleEndian>()?;
                let realtime = self.buf.read_u64::<LittleEndian>()?;
                let monotonic = self.buf.read_u64::<LittleEndian>()?;
                let boot_id = self.buf.read_u128::<LittleEndian>()?;
                let xor_hash = self.buf.read_u64::<LittleEndian>()?;
                let mut items: Vec<EntryItem> =
                    Vec::with_capacity(((size - 48) / 16).try_into().unwrap());
                for _ in 1..((size - 48) / 16) {
                    let object_offset = self.buf.read_u64::<LittleEndian>()?;
                    let hash = self.buf.read_u64::<LittleEndian>()?;
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
            }
            4 => {
                let flags = self.buf.read_u8()?;
                let mut reserved = [0u8; 6];
                self.buf.read_exact(&mut reserved)?;
                let size = self.buf.read_u64::<LittleEndian>()?;
                let object_header = ObjectHeader {
                    type_: ObjectType::ObjectDataHashTable,
                    flags: flags,
                    reserved: reserved,
                    size: size,
                };
                let mut items: Vec<HashItem> =
                    Vec::with_capacity(((size - 48) / 16).try_into().unwrap());
                for _ in 0..((size - 48) / 16) {
                    let hash_head_offset = self.buf.read_u64::<LittleEndian>()?;
                    let tail_hash_offset = self.buf.read_u64::<LittleEndian>()?;
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
                let flags = self.buf.read_u8()?;
                let mut reserved = [0u8; 6];
                self.buf.read_exact(&mut reserved)?;
                let size = self.buf.read_u64::<LittleEndian>()?;
                let object_header = ObjectHeader {
                    type_: ObjectType::ObjectFieldHashTable,
                    flags: flags,
                    reserved: reserved,
                    size: size,
                };
                let mut items: Vec<HashItem> =
                    Vec::with_capacity(((size - 48) / 16).try_into().unwrap());
                for _ in 0..((size - 48) / 16) {
                    let hash_head_offset = self.buf.read_u64::<LittleEndian>()?;
                    let tail_hash_offset = self.buf.read_u64::<LittleEndian>()?;
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
                let flags = self.buf.read_u8()?;
                let mut reserved = [0u8; 6];
                self.buf.read_exact(&mut reserved)?;
                let size = self.buf.read_u64::<LittleEndian>()?;
                let object_header = ObjectHeader {
                    type_: ObjectType::ObjectEntryArray,
                    flags: flags,
                    reserved: reserved,
                    size: size,
                };
                let next_entry_array_offset = self.buf.read_u64::<LittleEndian>()?;
                let mut items: Vec<u64> = Vec::with_capacity(((size - 20) / 8).try_into().unwrap());
                for _ in 0..((size - 20) / 8) {
                    let item = self.buf.read_u64::<LittleEndian>()?;
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
                let flags = self.buf.read_u8()?;
                let mut reserved = [0u8; 6];
                self.buf.read_exact(&mut reserved)?;
                let size = self.buf.read_u64::<LittleEndian>()?;
                let object_header = ObjectHeader {
                    type_: ObjectType::ObjectTag,
                    flags: flags,
                    reserved: reserved,
                    size: size,
                };
                let seqnum = self.buf.read_u64::<LittleEndian>()?;
                let epoch = self.buf.read_u64::<LittleEndian>()?;
                let mut tag = [0u8; 256 / 8];
                self.buf.read_exact(&mut tag)?;
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
}

pub struct ObjectIter<'a> {
    buf: Cursor<&'a [u8]>,
    pub current_offset: u64,
    next_offset: u64,
}

impl<'a> Iterator for ObjectIter<'a> {
    type Item = Object;

    fn next(&mut self) -> Option<Object> {
        let object = get_obj_at_offset(self.buf.get_ref(), self.next_offset);
        self.current_offset = self.next_offset;
        self.buf.seek(SeekFrom::Start(self.current_offset)).unwrap();
        match object {
            Ok(o) => {
                self.next_offset = self.next_obj_offset(&o)?;
                return Some(o);
            }
            Err(_) => {
                return None;
            }
        }
    }
}

pub struct EntryArrayIter<'a> {
    buf: Cursor<&'a [u8]>,
    current_offset: u64,
}

impl<'a> EntryArrayIter<'a> {
    fn new(buf: &'a [u8], start: u64) -> EntryArrayIter<'a> {
        let mut buf = Cursor::new(buf);
        buf.seek(SeekFrom::Start(start)).unwrap();

        EntryArrayIter {
            buf: buf,
            current_offset: start,
        }
    }
}

impl<'a> Iterator for EntryArrayIter<'a> {
    type Item = EntryArrayObject;

    fn next(&mut self) -> Option<EntryArrayObject> {
        if self.current_offset == 0 {
            return None;
        }
        let entry_array = get_obj_at_offset(self.buf.get_ref(), self.current_offset);
        match entry_array {
            Ok(h) => {
                if let Object::EntryArray(ea) = h {
                    self.current_offset = ea.next_entry_array_offset;
                    return Some(ea);
                } else {
                    return None;
                }
            }
            Err(_) => return None,
        }
    }
}

pub struct EntryIter<'a> {
    n_objects: u64,
    inner: ObjectIter<'a>,
}

impl<'a> EntryIter<'a> {
    fn new(buf: &'a [u8], start: u64, n_objects: u64) -> EntryIter<'a> {
        let inner = ObjectIter::new(buf, start);
        EntryIter {
            n_objects: n_objects,
            inner: inner,
        }
    }
}

impl<'a> Iterator for EntryIter<'a> {
    type Item = EntryObject;

    fn next(&mut self) -> Option<EntryObject> {
        for _ in 0..self.n_objects {
            let obj = self.inner.next()?;
            if let Object::Entry(e) = obj {
                return Some(e);
            }
        }
        None
    }
}
