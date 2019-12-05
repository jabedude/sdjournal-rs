use chrono::prelude::DateTime;
use chrono::Utc;
use clap::{Arg, App};
use sdjournal::journal::*;
use std::fs::File;
use std::io::{Error, Write};
use std::time::{Duration, UNIX_EPOCH};

// TODO: work on entrt struct to allow for propper formatting of entries

fn main() -> Result<(), Error> {

    let matches = App::new("journalctl-rs")
                          .version("0.1")
                          .author("Joshua A. <j.abraham1776@gmail.com>")
                          .about("Journalctl clone in rust")
                          .arg(Arg::with_name("INPUT")
                               .help("Sets the journal file to use")
                               .required(true)
                               .index(1))
                          .arg(Arg::with_name("header")
                                .long("header")
                               .help("Print info in the journal header"))
                          .arg(Arg::with_name("verify")
                                .long("verify")
                               .help("Verify journal file consistency"))
                          .arg(Arg::with_name("v")
                               .short("v")
                               .multiple(true)
                               .help("Sets the level of verbosity"))
                          .get_matches();

    let mut file = File::open(matches.value_of("INPUT").expect("input missing"))?;
    let journal = Journal::new(file)?;

    if matches.is_present("header") {
        println!("{}", journal.header);
        return Ok(());
    }

    if matches.is_present("verify") {
        println!("Journal verification: {}", journal.verify());
        return Ok(());
    }

    // Iterate over all entry objects
    for ent in journal.iter_entries() {
        let d = UNIX_EPOCH + Duration::from_micros(ent.realtime);
        let datetime = DateTime::<Utc>::from(d);
        // Formats the combined date and time with the specified format string.
        print!("{} ", datetime.format("%b %d %H:%M:%S"));

        for obj in ent.items {
            let data = match obj.item {
                Object::Data(d) => d,
                _ => continue,
            };

            if data.payload.starts_with(b"_HOSTNAME=") {
                std::io::stdout().write_all(&data.payload[10..])?;
            } else if data.payload.starts_with(b"SYSLOG_IDENTIFIER") {
                std::io::stdout().write_all(&data.payload[18..])?;
            } else if data.payload.starts_with(b"MESSAGE") {
                std::io::stdout().write_all(&data.payload[7..])?;
                std::io::stdout().write_all(b"\n")?;
            }
        }
    }

    Ok(())
}
