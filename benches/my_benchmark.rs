#[macro_use]
extern crate criterion;
#[macro_use]
extern crate lazy_static;

use criterion::Criterion;
use journald::*;
use std::fs::File;
use std::cell::Cell;
use memmap::Mmap;

lazy_static! {
    static ref FILE: File = File::open("tests/user-1000.journal").unwrap();
    static ref MMAP: Mmap = unsafe { Mmap::map(&FILE).expect("mmap err") };
    static ref BUF: &'static [u8] = &*MMAP;
}

fn test_retrieve_data(cur: &[u8]) {
    let journal = Journal::new(cur).unwrap();
    let iter_entries = journal.iter_entries();
    for entry in iter_entries {
        entry.get_data("MESSAGE", cur);
    }
}

fn test_object_iter_user(cur: &[u8]) {
    let journal = Journal::new(cur).unwrap();
    let mut obj_iter = journal.obj_iter();
    for _obj in obj_iter {
        let _e = 0;
    }
}

fn test_iter_entries_user(cur: &[u8]) {
    let journal = Journal::new(cur).unwrap();
    let iter_entries = journal.iter_entries();
    for _entry in iter_entries {
        let _e = 0;
    }
}

fn test_iter_entries_new_api_user(cur: &[u8]) {
    let journal = Journal::new(cur).unwrap();
    let ea_iter = journal.ea_iter();
    for ea in ea_iter {
        for entry in ea.items {
            let _e = get_obj_at_offset(cur, entry).unwrap();
        }
    }
}

fn test_obj_header_iter_user(cur: &[u8]) {
    let journal = Journal::new(cur).unwrap();
    let objheader_iter = journal.header_iter();
    for _oh in objheader_iter {
        let _e = 0;
    }
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("test_object_iter_user", |b| b.iter(|| test_object_iter_user(&BUF)));
    c.bench_function("test_iter_entries_user", |b| b.iter(|| test_iter_entries_user(&BUF)));
    c.bench_function("test_obj_header_iter_user", |b| b.iter(|| test_obj_header_iter_user(&BUF)));
    c.bench_function("test_retrieve_data", |b| b.iter(|| test_retrieve_data(&BUF)));
    c.bench_function("test_iter_entries_new_api_user", |b| b.iter(|| test_iter_entries_new_api_user(&BUF)));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
