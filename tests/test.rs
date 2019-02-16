use std::fs::File;

use journald::*;

#[test]
fn test_load_header() {
    let journal = File::open("tests/user-1000.journal").unwrap();
    let _h = load_header(&journal).unwrap();
}

#[test]
fn test_calc_next_object_offset() {
    let mut journal = Journal::new("tests/system.journal").unwrap();
    let oh = load_obj_header_at_offset(&journal.file, journal.header.field_hash_table_offset - OBJECT_HEADER_SZ).unwrap();
    assert_eq!(5584, next_obj_offset(&mut journal.file, &oh).unwrap());
}
