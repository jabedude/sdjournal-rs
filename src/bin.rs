use journald::*;
use std::env;

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
    let mut journal = Journal::new(&args[1]).unwrap();
    
    let n_entries = journal.header.n_objects;

    let mut obj_iter = ObjectIter::new(&mut journal).unwrap();
    for _ in 0..n_entries {
        let obj = obj_iter.next().expect("object iterator error");
        if let Object::Data(d) = obj {
            println!("type: {:?} size: {}", d.object.type_, d.object.size);
            println!("Payload: {:?}", show(&d.payload));
        }
    }

    //let mut obj_iter = ObjectHeaderIter::new(&mut journal).unwrap();
    //for _ in 0..n_entries {
    //    let oh = obj_iter.next().expect("object iterator error");
    //    if oh.type_ == ObjectType::ObjectData {
    //        println!("type: {:?} size: {}", oh.type_, oh.size);
    //    }
    //}
}
