#![feature(test)]
extern crate test;

#[cfg(test)]
mod tests {
use journald::*;
use test::Bencher;

    #[test]
    fn test_object_iter_user() {

        let mut journal = Journal::new("tests/user-1000.journal").unwrap();

        let mut obj_iter = ObjectIter::new(&mut journal).unwrap();
        for obj in obj_iter {
            if let Object::data(d) = obj {
                println!("type: {:?} size: {}", d.object.type_, d.object.size);
                println!("Payload: {:?}", d.payload);
            }
        }
    }

    #[bench]
    fn bench_object_iter_user(b: &mut Bencher) {
        b.iter(|| test_object_iter_user());
    }
}
