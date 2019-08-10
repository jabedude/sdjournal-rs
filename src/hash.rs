use libc;

extern "C" {
    fn hash64(data: *const libc::c_void, length: libc::size_t) -> libc::uint64_t;
}

pub fn rhash64(data: &[u8]) -> u64 {
    let ret = unsafe { hash64(data.as_ptr() as *const libc::c_void, data.len()) };
    return ret;
}

// TODO: add real tests
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash() {
        assert_eq!(0, 0);
    }
}
