use libc;

extern {
    fn hash64(data: * const libc::c_void, length: libc::size_t) -> libc::uint64_t;
}

pub fn rhash64(data: &[u8]) -> u64 {
    let ret = unsafe {
        hash64(data.as_ptr() as * const libc::c_void, data.len())
    };
    return ret;
}



#[cfg(test)]
mod tests {
    #[test]
    fn test_hash() {
        assert!(true);
    }
}
