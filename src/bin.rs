use journald::*;
use std::env;
use std::mem;
use std::io::{Read, Seek, SeekFrom};


fn main() {
    let args: Vec<String> = env::args().collect();
    let mut journal = Journal::new(&args[1]).unwrap();
    
    let n_entries = journal.header.n_objects;
    let mut obj_iter = ObjectHeaderIter::new(&mut journal).unwrap();
    for _ in 0..n_entries {
        let oh = obj_iter.next().unwrap();
        if oh.type_ == ObjectType::OBJECT_DATA {
            println!("type: {:?} size: {}", oh.type_, oh.size);
        }
    }

}
