#[cfg(test)]
mod tests {
    use sdjournal::*;
    use memmap::Mmap;
    use std::fs::File;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_journal_state_offline() {
        let file = File::open("tests/user-1000.journal").unwrap();
        let mmap = unsafe { Mmap::map(&file).expect("mmap err") };
        let buf = &*mmap;
        let journal = Journal::from_bytes(buf).unwrap();
        assert!(journal.header.state == JournalState::Offline);
    }

    #[test]
    fn test_header_parsing_user() {
        let file = File::open("tests/user-1000.journal").unwrap();
        let mmap = unsafe { Mmap::map(&file).expect("mmap err") };
        let buf = &*mmap;
        let journal = Journal::from_bytes(buf).unwrap();
        assert_eq!(journal.header.header_size, 240);
        assert_eq!(journal.header.state, JournalState::Offline);
        assert_eq!(journal.header.arena_size, 8388368);
        assert_eq!(journal.header.data_hash_table_size, 233016);
        assert_eq!(journal.header.field_hash_table_size, 333);
        assert_eq!(journal.header.n_objects, 2123);
        assert_eq!(journal.header.n_entries, 645);
        assert_eq!(journal.header.n_entry_arrays, 595);
    }

    #[test]
    fn test_header_parsing_system() {
        let file = File::open("tests/system.journal").unwrap();
        let mmap = unsafe { Mmap::map(&file).expect("mmap err") };
        let buf = &*mmap;
        let journal = Journal::from_bytes(buf).unwrap();
        assert_eq!(journal.header.file_id, 0xf5c61067f7f64d32963ef8770ad232e6);
        assert_eq!(journal.header.machine_id, 0xc48110828c69477cb6be61e3601f532f);
        assert_eq!(journal.header.header_size, 240);
        assert_eq!(journal.header.arena_size, 41942800);
        assert_eq!(journal.header.state, JournalState::Offline);
        assert_eq!(journal.header.data_hash_table_size, 233016);
        assert_eq!(journal.header.field_hash_table_size, 333);
        assert_eq!(journal.header.n_objects, 52245);
        assert_eq!(journal.header.n_entries, 40600);
        assert_eq!(journal.header.n_entry_arrays, 10739);
    }

    #[test]
    fn test_compression_false() {
        let file = File::open("tests/user-1000.journal").unwrap();
        let mmap = unsafe { Mmap::map(&file).expect("mmap err") };
        let buf = &*mmap;
        let journal = Journal::from_bytes(buf).unwrap();
        let obj_iter = journal.iter_headers();
        for oh in obj_iter {
            assert!(!oh.is_compressed());
        }
    }

    #[test]
    fn test_hash_object() {
        use sdjournal::hash::rhash64;

        let file = File::open("tests/user-1000.journal").unwrap();
        let mmap = unsafe { Mmap::map(&file).expect("mmap err") };
        let buf = &*mmap;
        let journal = Journal::from_bytes(buf).unwrap();
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
        let file = File::open("tests/user-1000.journal").unwrap();
        let mmap = unsafe { Mmap::map(&file).expect("mmap err") };
        let buf = &*mmap;
        let journal = Journal::from_bytes(buf).unwrap();
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
    fn test_iter_entries_user() {
        let file = File::open("tests/user-1000.journal").unwrap();
        let mmap = unsafe { Mmap::map(&file).expect("mmap err") };
        let buf = &*mmap;
        let journal = Journal::from_bytes(buf).unwrap();
        let expected = journal.header.n_entries;
        let mut counter = 0;
        let ent_iter = journal.iter_entries();
        for _ in ent_iter {
            counter += 1;
        }
        assert_eq!(counter, expected);
    }

    #[test]
    fn test_multi_iter_user() {
        let file = File::open("tests/user-1000.journal").unwrap();
        let mmap = unsafe { Mmap::map(&file).expect("mmap err") };
        let buf = &*mmap;
        let journal = Journal::from_bytes(buf).unwrap();
        let hdr_iter = journal.iter_headers();
        for oh in hdr_iter {
            if oh.type_ == ObjectType::ObjectData {
                println!("type: {:?} size: {}", oh.type_, oh.size);
            }
        }

        let _obj_iter = journal.obj_iter();
        let iter_entries = journal.iter_entries();
        for entry in iter_entries {
            println!("timestamp: {}", entry.realtime);
        }
    }

    #[test]
    fn test_object_iter_headers_user() {
        let file = File::open("tests/user-1000.journal").unwrap();
        let mmap = unsafe { Mmap::map(&file).expect("mmap err") };
        let buf = &*mmap;
        let journal = Journal::from_bytes(buf).unwrap();
        let expected = journal.header.n_objects;
        let obj_iter = journal.iter_headers();
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
    fn test_object_iter_headers_system() {
        let file = File::open("tests/system.journal").unwrap();
        let mmap = unsafe { Mmap::map(&file).expect("mmap err") };
        let buf = &*mmap;
        let journal = Journal::from_bytes(buf).unwrap();
        let expected = journal.header.n_objects;
        let obj_iter = journal.iter_headers();
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
        let file = File::open("tests/system.journal").unwrap();
        let mmap = unsafe { Mmap::map(&file).expect("mmap err") };
        let buf = &*mmap;
        let journal = Journal::from_bytes(buf).unwrap();
        let expected = journal.header.n_objects;
        let mut counter = 0;
        let obj_iter = journal.obj_iter();
        for _ in obj_iter {
            counter += 1;
        }
        assert_eq!(counter, expected);
    }

    #[test]
    fn test_iter_entries_system() {
        let file = File::open("tests/system.journal").unwrap();
        let mmap = unsafe { Mmap::map(&file).expect("mmap err") };
        let buf = &*mmap;
        let journal = Journal::from_bytes(buf).unwrap();
        let expected = journal.header.n_entries;
        let mut counter = 0;
        let ent_iter = journal.iter_entries();
        for _ in ent_iter {
            counter += 1;
        }
        assert_eq!(counter, expected);
    }

    #[test]
    fn test_entry_array_iter_user() {
        let file = File::open("tests/user-1000.journal").unwrap();
        let mmap = unsafe { Mmap::map(&file).expect("mmap err") };
        let buf = &*mmap;
        let journal = Journal::from_bytes(buf).unwrap();
        let expected = journal.header.n_entries;
        let mut counter = 0;
        let ent_iter = journal.ea_iter();
        for ea in ent_iter {
            for _ in ea.items {
                counter += 1;
            }
        }
        assert_eq!(counter, expected);
    }

    #[test]
    fn test_data_object_is_trusted() {
        let file = File::open("tests/user-1000.journal").unwrap();
        let mmap = unsafe { Mmap::map(&file).expect("mmap err") };
        let buf = &*mmap;
        let journal = Journal::from_bytes(buf).unwrap();

        let ent_iter = journal.iter_entries();
        for ent in ent_iter {
            for obj in ent.items {
                let data = match get_obj_at_offset(buf, obj.object_offset).unwrap() {
                    Object::Data(d) => d,
                    _ => continue,
                };
                let string = std::str::from_utf8(&data.payload).unwrap();
                if string.starts_with('_') {
                    assert!(data.payload_is_trusted());
                }
            }
        }
    }

    #[test]
    fn test_hash_data_objects_user() {
        use sdjournal::traits::HashableObject;

        let file = File::open("tests/user-1000.journal").unwrap();
        let mmap = unsafe { Mmap::map(&file).expect("mmap err") };
        let buf = &*mmap;
        let journal = Journal::from_bytes(buf).unwrap();

        for obj in journal.obj_iter() {
            if let Object::Data(d) = obj {
                let stored_hash = d.hash;
                let calc_hash = d.hash();
                assert_eq!(stored_hash, calc_hash);
            }
        }
    }

    #[test]
    fn test_hash_field_objects_user() {
        use sdjournal::traits::HashableObject;

        let file = File::open("tests/user-1000.journal").unwrap();
        let mmap = unsafe { Mmap::map(&file).expect("mmap err") };
        let buf = &*mmap;
        let journal = Journal::from_bytes(buf).unwrap();

        for obj in journal.obj_iter() {
            if let Object::Field(f) = obj {
                let stored_hash = f.hash;
                let calc_hash = f.hash();
                assert_eq!(stored_hash, calc_hash);
            }
        }
    }

    #[test]
    fn test_hash_field_objects_system() {
        use sdjournal::traits::HashableObject;

        let file = File::open("tests/system.journal").unwrap();
        let mmap = unsafe { Mmap::map(&file).expect("mmap err") };
        let buf = &*mmap;
        let journal = Journal::from_bytes(buf).unwrap();

        for obj in journal.obj_iter() {
            if let Object::Field(f) = obj {
                println!("payload: {:?}", &f.payload);
                println!("obj header size: {}, payload vec size: {}, payload len should be: {}",
                                                f.object.size, f.payload.len(), f.object.size - 35);
                let stored_hash = f.hash;
                let calc_hash = f.hash();
                assert_eq!(stored_hash, calc_hash);
            }
        }
    }

    #[test]
    fn test_hash_entry_objects_user() {
        use sdjournal::traits::HashableObject;

        let file = File::open("tests/user-1000.journal").unwrap();
        let mmap = unsafe { Mmap::map(&file).expect("mmap err") };
        let buf = &*mmap;
        let journal = Journal::from_bytes(buf).unwrap();

        for entry in journal.iter_entries() {
            let stored_hash = entry.xor_hash;
            let calc_hash = entry.hash();
            assert_eq!(stored_hash, calc_hash);
        }
    }

    #[test]
    fn test_journal_verify_user() {
        let file = File::open("tests/user-1000.journal").unwrap();
        let mmap = unsafe { Mmap::map(&file).expect("mmap err") };
        let buf = &*mmap;
        let journal = Journal::from_bytes(buf).unwrap();

        assert_eq!(journal.verify(false), true);
    }

    #[test]
    fn test_journal_verify_system() {
        let file = File::open("tests/system.journal").unwrap();
        let mmap = unsafe { Mmap::map(&file).expect("mmap err") };
        let buf = &*mmap;
        let journal = Journal::from_bytes(buf).unwrap();

        assert_eq!(journal.verify(false), true);
    }
}
