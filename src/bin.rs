use journald::*;
use std::env;


fn main() {
    let args: Vec<String> = env::args().collect();
    let mut journal = Journal::new(&args[1]).unwrap();
    
    let n_entries = journal.header.n_objects;

    let mut obj_iter = ObjectIter::new(&mut journal).unwrap();
    for _ in 0..n_entries {
        let obj = obj_iter.next().expect("object iterator error");
        if let Object::data(d) = obj {
            println!("type: {:?} size: {}", d.object.type_, d.object.size);
            println!("Payload: {:?}", d.payload);
        }
    }

    //let mut obj_iter = ObjectHeaderIter::new(&mut journal).unwrap();
    //for _ in 0..n_entries {
    //    let oh = obj_iter.next().expect("object iterator error");
    //    if oh.type_ == ObjectType::OBJECT_DATA {
    //        println!("type: {:?} size: {}", oh.type_, oh.size);
    //    }
    //}
}
