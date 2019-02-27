#[macro_use]
extern crate criterion;

use criterion::Criterion;
use journald::*;

fn test_object_iter_user(mut journal: &mut Journal) {
    let mut obj_iter = ObjectIter::new(&mut journal).unwrap();
    for obj in obj_iter {
	if let Object::Data(d) = obj {
            let _e = 0;
	}
    }
}

fn criterion_benchmark(c: &mut Criterion) {
    let mut journal = Journal::new("tests/user-1000.journal").unwrap();
    c.bench_function("test_object_iter_user", move |b| b.iter(|| test_object_iter_user(&mut journal)));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
