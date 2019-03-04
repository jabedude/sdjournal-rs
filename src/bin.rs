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
    
    let entry_iter = journal.entry_iter();
    for entry in entry_iter {
        println!("timestamp: {}", entry.realtime);
        println!("message: {}", entry.get_data("MESSAGE", buf).unwrap());
    }
}
