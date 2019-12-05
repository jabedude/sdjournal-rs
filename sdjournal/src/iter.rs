use byteorder::{LittleEndian, ReadBytesExt};
use std::convert::TryInto;
use std::io::{Error, ErrorKind, Read, Result, Seek, SeekFrom};
use std::collections::VecDeque;

pub use crate::journal::*;

use crate::traits::{SizedObject};

pub struct ObjectHeaderIter<'a, T>
where
    &'a T: Read + Seek,
{
    buf: &'a T,
    next_offset: u64,
}

impl<'a, T> ObjectHeaderIter<'a, T>
where
    &'a T: Read + Seek,
{
    pub (crate) fn new(mut buf: &'a T, start: u64) -> ObjectHeaderIter<'a, T> {
        //let mut buf = Cursor::new(buf);
        buf.seek(SeekFrom::Start(start)).unwrap();

        ObjectHeaderIter {
            buf: buf,
            next_offset: start,
        }
    }

    pub fn next_obj_header_offset<R: SizedObject>(&mut self, obj: &R) -> Option<u64> {
        let curr = self.buf.seek(SeekFrom::Current(0)).unwrap();
        let offset = align64(curr + obj.size() - OBJECT_HEADER_SZ);
        Some(offset)
    }

    pub fn load_obj_header_at_offset(&mut self, offset: u64) -> Result<ObjectHeader> {
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

impl<'a, T> Iterator for ObjectHeaderIter<'a, T>
where
    &'a T: Read + Seek,
{
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

pub struct ObjectIter<'a, T>
where
    &'a T: Read + Seek,
{
    buf: &'a T,
    pub current_offset: u64,
    next_offset: u64,
}

impl<'a, T> ObjectIter<'a, T>
where
    &'a T: Read + Seek,
{
    pub (crate) fn new(mut buf: &'a T, start: u64) -> ObjectIter<'a, T> {
        buf.seek(SeekFrom::Start(start)).unwrap();

        ObjectIter {
            buf: buf,
            current_offset: start,
            next_offset: start,
        }
    }

    fn next_obj_offset<R: SizedObject>(&mut self, obj: &R) -> Option<u64> {
        let pos = self.buf.seek(SeekFrom::Current(0)).ok()?;
        let offset = align64(pos + obj.size());
        Some(offset)
    }
}

impl<'a, T> Iterator for ObjectIter<'a, T>
where
    &'a T: Read + Seek,
{
    type Item = Object;

    fn next(&mut self) -> Option<Object> {
        let object = get_obj_at_offset(&mut self.buf, self.next_offset);
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

pub struct EntryArrayIter<'a, T>
where
    &'a T: Read + Seek,
{
    buf: &'a T,
    current_offset: u64,
}

impl<'a, T> EntryArrayIter<'a, T>
where
    &'a T: Read + Seek,
{
    pub(crate) fn new(mut buf: &'a T, start: u64) -> EntryArrayIter<'a, T> {
        buf.seek(SeekFrom::Start(start)).unwrap();

        EntryArrayIter {
            buf: buf,
            current_offset: start,
        }
    }
}

impl<'a, T> Iterator for EntryArrayIter<'a, T>
where
    &'a T: Read + Seek,
{
    type Item = EntryArrayObject;

    fn next(&mut self) -> Option<EntryArrayObject> {
        if self.current_offset == 0 {
            return None;
        }
        let entry_array = get_obj_at_offset(&mut self.buf, self.current_offset);
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

pub struct EntryIter<'a, T>
where
    &'a T: Read + Seek,
{
    buf: &'a T,
    offsets: VecDeque<u64>,
}

impl<'a, T> EntryIter<'a, T>
where
    &'a T: Read + Seek,
{
    pub(crate) fn new(buf: &'a T, start: u64, n_objects: u64) -> EntryIter<'a, T> {
        let ea_iter = EntryArrayIter::new(buf, start);

        let mut offsets: VecDeque<u64> = VecDeque::with_capacity(n_objects.try_into().unwrap());
        // TODO: see if pushing entire vector will boost perf
        for entry_array in ea_iter {
            for offset in entry_array.items {
                offsets.push_back(offset);
            }
        }

        EntryIter {
            buf: buf,
            offsets: offsets,
        }
    }
}

impl<'a, T> Iterator for EntryIter<'a, T>
where
    &'a T: Read + Seek,
{
    type Item = EntryObject;

    fn next(&mut self) -> Option<EntryObject> {
        let offset = self.offsets.pop_front()?;
        let entry = get_obj_at_offset(&mut self.buf, offset).unwrap();
        if let Object::Entry(e) = entry {
            return Some(e);
        } else {
            return None;
        }
    }
}
