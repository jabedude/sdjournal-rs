#[cfg(test)]
mod tests {
use journald::*;
use std::fs::File;
use std::cell::Cell;
use memmap::Mmap;

    #[test]
    fn test_compression_false() {
        let mut file = File::open("tests/user-1000.journal").unwrap();
        let mmap = unsafe { Mmap::map(&file).expect("mmap err") };
        let buf = &*mmap;
        let c = Cell::new(buf);
        let mut journal = Journal::new(buf).unwrap();
        let obj_iter = journal.header_iter();
        for oh in obj_iter {
            assert!(!oh.is_compressed());
        }
    }

    #[test]
    fn test_hash_object() {
    use journald::hash::rhash64;

        let mut file = File::open("tests/user-1000.journal").unwrap();
        let mmap = unsafe { Mmap::map(&file).expect("mmap err") };
        let buf = &*mmap;
        let c = Cell::new(buf);
        let mut journal = Journal::new(buf).unwrap();
        let expected = journal.header.n_objects;
        let obj_iter = journal.obj_iter();
        for obj in obj_iter {
            if let Object::Data(o) = obj {
                let h = rhash64(&o.payload);
                assert_eq!(h, o.hash);
            }
        }
    }

    #[test]
    fn test_object_iter_user() {

        let mut file = File::open("tests/user-1000.journal").unwrap();
        let mmap = unsafe { Mmap::map(&file).expect("mmap err") };
        let buf = &*mmap;
        let c = Cell::new(buf);
        let mut journal = Journal::new(buf).unwrap();
        let expected = journal.header.n_objects;
        let mut counter = 0;
        let obj_iter = journal.obj_iter();
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
        let ent_iter = journal.entry_iter();
        for ent in ent_iter {
            counter += 1;
        }
        assert_eq!(counter, expected);
    }

    #[test]
    fn test_multi_iter_user() {

        let mut file = File::open("tests/user-1000.journal").unwrap();
        let mmap = unsafe { Mmap::map(&file).expect("mmap err") };
        let buf = &*mmap;
        let c = Cell::new(buf);
        let mut journal = Journal::new(buf).unwrap();
        let hdr_iter = journal.header_iter();
        for oh in hdr_iter {
            if oh.type_ == ObjectType::ObjectData {
                println!("type: {:?} size: {}", oh.type_, oh.size);
            }
        }

        let obj_iter = journal.obj_iter();
        let entry_iter = journal.entry_iter();
        for entry in entry_iter {
            println!("timestamp: {}", entry.realtime);
        }
    }

    #[test]
    fn test_object_header_iter_user() {
        let mut file = File::open("tests/user-1000.journal").unwrap();
        let mmap = unsafe { Mmap::map(&file).expect("mmap err") };
        let buf = &*mmap;
        let mut journal = Journal::new(buf).unwrap();
        let expected = journal.header.n_objects;
        let obj_iter = journal.header_iter();
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
        let obj_iter = journal.header_iter();
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
        let obj_iter = journal.obj_iter();
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
        let ent_iter = journal.entry_iter();
        for ent in ent_iter {
            counter += 1;
        }
        assert_eq!(counter, expected);
    }
}
