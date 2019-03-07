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
    let entry_iter = journal.entry_iter();
    for entry in entry_iter {
        entry.get_data("MESSAGE", cur);
    }
}

fn test_object_iter_user(cur: &[u8]) {
    let journal = Journal::new(cur).unwrap();
    let mut obj_iter = journal.obj_iter();
    for obj in obj_iter {
        let _e = 0;
    }
}

fn test_entry_iter_user(cur: &[u8]) {
    let mut journal = Journal::new(cur).unwrap();
    let entry_iter = journal.entry_iter();
    for entry in entry_iter {
        let _e = 0;
    }
}

fn test_obj_header_iter_user(cur: &[u8]) {
    let mut journal = Journal::new(cur).unwrap();
    let objheader_iter = journal.header_iter();
    for oh in objheader_iter {
        let _e = 0;
    }
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("test_object_iter_user", |b| b.iter(|| test_object_iter_user(&BUF)));
    c.bench_function("test_entry_iter_user", |b| b.iter(|| test_entry_iter_user(&BUF)));
    c.bench_function("test_obj_header_iter_user", |b| b.iter(|| test_obj_header_iter_user(&BUF)));
    c.bench_function("test_retrieve_data", |b| b.iter(|| test_retrieve_data(&BUF)));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
