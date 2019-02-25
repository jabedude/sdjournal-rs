use journald::*;
use std::env;
use std::fs::File;
use std::io::Cursor;
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
    let mut cur = Cursor::new(buf);
    let mut journal = Journal::new(&mut cur).unwrap();
    
    let obj_iter = ObjectIter::new(&mut journal).unwrap();
    for obj in obj_iter {
        if let Object::Data(d) = obj {
            println!("type: {:?} size: {}", d.object.type_, d.object.size);
            println!("Payload: {:?}", show(&d.payload));
        }
    }

    //let mut obj_iter = ObjectHeaderIter::new(&mut journal).unwrap();
    //for in obj_iter {
    //    let oh = obj_iter.next().expect("object iterator error");
    //    if oh.type_ == ObjectType::ObjectData {
    //        println!("type: {:?} size: {}", oh.type_, oh.size);
    //    }
    //}
}
