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

fn test_object_iter_user(cur: &[u8]) {
    let mut journal = Journal::new(cur).unwrap();
    let mut obj_iter = ObjectIter::new(&mut journal).unwrap();
    for obj in obj_iter {
	if let Object::Data(d) = obj {
            let _e = 0;
	}
    }
}

fn criterion_benchmark(c: &mut Criterion) {

    c.bench_function("test_object_iter_user", |b| b.iter(|| test_object_iter_user(&BUF)));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
