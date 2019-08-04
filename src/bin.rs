use journald::*;
use std::env;
use std::fs::File;
use memmap::Mmap;
use chrono::prelude::DateTime;
use chrono::{Utc};
use std::time::{UNIX_EPOCH, Duration};
use std::io::Write;
use std::io::{Error, ErrorKind};

// TODO: work on entrt struct to allow for propper formatting of entries


fn main() -> Result<(), Error> {
    let args: Vec<String> = env::args().collect();

    // TODO: going to need to handle command line flags...
    if args.len() != 2 {
        println!("Usage: {} <journal file>", args[0]);
        return Err(Error::new(ErrorKind::InvalidInput, "Needs at least one argument"));
    }

    let file = File::open(&args[1])?;
    let mmap = unsafe { Mmap::map(&file).expect("mmap err") };
    let buf = &*mmap;
    let journal = Journal::new(buf)?;
    
    //Iterate over all entry objects
    for ent in journal.iter_entries() {
        let d = UNIX_EPOCH + Duration::from_micros(ent.realtime);
        let datetime = DateTime::<Utc>::from(d);
        // Formats the combined date and time with the specified format string.
        print!("{} ", datetime.format("%b %d %H:%M:%S"));
        for obj in ent.items {
            //println!("obj: {}", obj.object_offset);
            let data = match get_obj_at_offset(buf, obj.object_offset)? {
                Object::Data(d) => d,
                _ => continue,
            };

            if data.payload.starts_with(b"SYSLOG_IDENTIFIER") {
                std::io::stdout().write_all(&data.payload[18..])?;
            } else if data.payload.starts_with(b"MESSAGE") {
                std::io::stdout().write_all(&data.payload[7..])?;
                std::io::stdout().write_all(b"\n")?;
            }

        }
    }

    Ok(())
}
