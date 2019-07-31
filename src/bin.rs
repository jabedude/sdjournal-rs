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
    
    let ent_iter = journal.entry_iter();
    for ent in ent_iter {
        println!("ent: {}", ent.realtime);
        for obj in ent.items {
            //println!("obj: {}", obj.object_offset);
            let data = match get_obj_at_offset(buf, obj.object_offset).unwrap() {
                Object::Data(d) => d,
                _ => continue,
            };
            println!("obj: {}", str::from_utf8(&data.payload).unwrap());
        }
    }
}
