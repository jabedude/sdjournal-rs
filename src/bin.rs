use journald::*;
use std::env;
use std::fs::File;
use memmap::Mmap;
use chrono::prelude::DateTime;
use chrono::{Utc};
use std::time::{SystemTime, UNIX_EPOCH, Duration};

use std::ascii::escape_default;
use std::str;
use std::io::Write;

// TODO: work on entrt struct to allow for propper formatting of entries

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

    // TODO: going to need to handle command line flags...
    if args.len() != 2 {
        println!("Usage: {} <journal file>", args[0]);
        return;
    }

    let file = File::open(&args[1]).unwrap();
    let mmap = unsafe { Mmap::map(&file).expect("mmap err") };
    let buf = &*mmap;
    let journal = Journal::new(buf).unwrap();
    
    //Iterate over all entry objects
    let ent_iter = journal.iter_entries();
    for ent in ent_iter {
        let d = UNIX_EPOCH + Duration::from_micros(ent.realtime);
        let datetime = DateTime::<Utc>::from(d);
        // Formats the combined date and time with the specified format string.
        print!("{} ", datetime.format("%b %d %H:%M:%S"));
        for obj in ent.items {
            //println!("obj: {}", obj.object_offset);
            let data = match get_obj_at_offset(buf, obj.object_offset).unwrap() {
                Object::Data(d) => d,
                _ => continue,
            };

            if data.payload.starts_with(b"SYSLOG_IDENTIFIER") {
                std::io::stdout().write_all(&data.payload[18..]);
            } else if data.payload.starts_with(b"MESSAGE") {
                std::io::stdout().write_all(&data.payload[7..]);
                std::io::stdout().write_all(b"\n");
            }

        }
    }
}
