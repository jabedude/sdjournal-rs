#[macro_use]
extern crate criterion;

use criterion::Criterion;
use journald::*;
use std::fs::File;
use std::io::Cursor;
use memmap::Mmap;

fn test_object_iter_user(cur: Cursor<&[u8]>) {
    let mut journal = Journal::new(&mut cur).unwrap();
    let mut obj_iter = ObjectIter::new(&mut journal).unwrap();
    for obj in obj_iter {
	if let Object::Data(d) = obj {
            let _e = 0;
	}
    }
}

fn criterion_benchmark(c: &mut Criterion) {
    let mut file = File::open("tests/user-1000.journal").unwrap();
    let mmap = unsafe { Mmap::map(&file).expect("mmap err") };
    let buf = &*mmap;
    let mut cur = Cursor::new(buf);

    c.bench_function("test_object_iter_user", |b| b.iter(|| test_object_iter_user(cur)));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
