use libc;

extern "C" {
    fn hash64(data: *const libc::c_void, length: libc::size_t) -> libc::uint64_t;
}

pub fn rhash64(data: &[u8]) -> u64 {
    let ret = unsafe { hash64(data.as_ptr() as *const libc::c_void, data.len()) };
    return ret;
}

// TODO: fix this to run on `cargo test`
#[cfg(test)]
mod tests {
    #[test]
    fn test_hash() {
        assert!(true);
    }
}
