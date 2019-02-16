use journald::*;
use std::env;
use std::mem;
use std::io::{Read, Seek, SeekFrom};


fn main() {
    let args: Vec<String> = env::args().collect();
    let mut journal = Journal::new(&args[1]).unwrap();

    //let mut obj_iter = ObjectHeaderIter::new(&mut journal).unwrap();
    //for _ in 0..5 {
    //    let oh = obj_iter.next().unwrap();
    //    println!("type: {:?}", oh.type_);
    //}
}
