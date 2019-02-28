#![feature(test)]
extern crate test;

#[cfg(test)]
mod tests {
use journald::*;
use std::fs::File;
use std::cell::Cell;
use memmap::Mmap;

    #[test]
    fn test_object_iter_user() {

        let mut file = File::open("tests/user-1000.journal").unwrap();
        let mmap = unsafe { Mmap::map(&file).expect("mmap err") };
        let buf = &*mmap;
        let c = Cell::new(buf);
        let mut journal = Journal::new(buf).unwrap();
        let expected = journal.header.n_objects;
        let mut counter = 0;
        let mut obj_iter = ObjectIter::new(&mut journal).unwrap();
        for obj in obj_iter {
            counter += 1;
            if let Object::Data(d) = obj {
                println!("type: {:?} size: {}", d.object.type_, d.object.size);
                println!("Payload: {:?}", d.payload);
            }
        }
        assert_eq!(counter, expected);
    }

    #[test]
    fn test_entry_iter_user() {

        let mut file = File::open("tests/user-1000.journal").unwrap();
        let mmap = unsafe { Mmap::map(&file).expect("mmap err") };
        let buf = &*mmap;
        let c = Cell::new(buf);
        let mut journal = Journal::new(buf).unwrap();
        let expected = journal.header.n_entries;
        let mut counter = 0;
        let mut ent_iter = EntryIter::new(&mut journal).unwrap();
        for ent in ent_iter {
            counter += 1;
        }
        assert_eq!(counter, expected);
    }

    #[test]
    fn test_object_header_iter_user() {
        let mut file = File::open("tests/user-1000.journal").unwrap();
        let mmap = unsafe { Mmap::map(&file).expect("mmap err") };
        let buf = &*mmap;
        let mut journal = Journal::new(buf).unwrap();
        let expected = journal.header.n_objects;
        let mut obj_iter = ObjectHeaderIter::new(&mut journal).unwrap();
        let mut counter = 0;
        for oh in obj_iter {
            if oh.type_ == ObjectType::ObjectData {
                println!("type: {:?} size: {}", oh.type_, oh.size);
            }
            counter += 1;
        }
        assert_eq!(counter, expected);
    }

    #[test]
    fn test_object_header_iter_system() {
        let mut file = File::open("tests/system.journal").unwrap();
        let mmap = unsafe { Mmap::map(&file).expect("mmap err") };
        let buf = &*mmap;
        let mut journal = Journal::new(buf).unwrap();
        let expected = journal.header.n_objects;
        let mut obj_iter = ObjectHeaderIter::new(&mut journal).unwrap();
        let mut counter = 0;
        for oh in obj_iter {
            if oh.type_ == ObjectType::ObjectData {
                println!("type: {:?} size: {}", oh.type_, oh.size);
            }
            counter += 1;
        }
        assert_eq!(counter, expected);
    }

    #[test]
    fn test_object_iter_system() {

        let mut file = File::open("tests/system.journal").unwrap();
        let mmap = unsafe { Mmap::map(&file).expect("mmap err") };
        let buf = &*mmap;
        let c = Cell::new(buf);
        let mut journal = Journal::new(buf).unwrap();
        let expected = journal.header.n_objects;
        let mut counter = 0;
        let mut obj_iter = ObjectIter::new(&mut journal).unwrap();
        for obj in obj_iter {
            counter += 1;
        }
        assert_eq!(counter, expected);
    }

    #[test]
    fn test_entry_iter_system() {

        let mut file = File::open("tests/system.journal").unwrap();
        let mmap = unsafe { Mmap::map(&file).expect("mmap err") };
        let buf = &*mmap;
        let c = Cell::new(buf);
        let mut journal = Journal::new(buf).unwrap();
        let expected = journal.header.n_entries;
        let mut counter = 0;
        let mut ent_iter = EntryIter::new(&mut journal).unwrap();
        for ent in ent_iter {
            counter += 1;
        }
        assert_eq!(counter, expected);
    }
}
