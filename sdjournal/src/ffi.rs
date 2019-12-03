use std::ffi;
use std::ptr;
use std::slice;
use std::sync::atomic;

use libc::c_char;
use libc::c_int;
use libc::c_void;
use libc::size_t;
use libc::ssize_t;

use crate::*;


#[no_mangle]
pub fn sd_journal_open(ret: *mut *mut c_void, flags: c_int) -> c_int {
    unimplemented!();
}

#[no_mangle]
pub fn sd_journal_close(j: &mut sd_journal) {
    unimplmented!();
}

#[no_mangle]
pub extern fn print_hello_from_rust() {
    println!("Hello from Rust");
}
