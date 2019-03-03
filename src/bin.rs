use journald::*;
use std::env;
use std::fs::File;
use memmap::Mmap;

use std::ascii::escape_default;
use std::str;

fn show(bs: &[u8]) -> String {
    let mut visible = String::new();
    for &b in bs {
        let part: Vec<u8> = escape_default(b).collect();
        visible.push_str(str::from_utf8(&part).unwrap());
    }
    visible
}


fn main() {
    let args: Vec<String> = env::args().collect();

    let file = File::open(&args[1]).unwrap();
    let mmap = unsafe { Mmap::map(&file).expect("mmap err") };
    let buf = &*mmap;
    let journal = Journal::new(buf).unwrap();
    
    let hdr_iter = journal.header_iter();
    for oh in hdr_iter {
        if oh.type_ == ObjectType::ObjectData {
            println!("type: {:?} size: {}", oh.type_, oh.size);
        }
    }

    let obj_iter = journal.obj_iter();
    for obj in obj_iter {
        if let Object::Data(d) = obj {
            println!("type: {:?} size: {}", d.object.type_, d.object.size);
            println!("Payload: {:?}", d.payload);
        }
    }

    let entry_iter = journal.entry_iter();
    for entry in entry_iter {
        println!("timestamp: {}", entry.realtime);
    }
}
