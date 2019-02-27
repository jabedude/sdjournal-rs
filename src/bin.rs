use journald::*;
use std::env;
use std::fs::File;
use std::cell::Cell;
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

    let mut file = File::open(&args[1]).unwrap();
    let mmap = unsafe { Mmap::map(&file).expect("mmap err") };
    let buf = &*mmap;
    let c = Cell::new(buf);
    let mut journal = Journal::new(buf).unwrap();
    
    let obj_iter = ObjectIter::new(&mut journal).unwrap();
    for obj in obj_iter {
        if let Object::Entry(e) = obj {
            println!("entry object time: {}", e.realtime);
            for eo in e.items {
                let o = get_obj_at_offset(c.get(), eo.object_offset).unwrap();
                if let Object::Data(d) = o {
                    println!("object type {}", show(&d.payload));
                }
            }
        }
    }
}
