use libc;

extern "C" {
    fn hash64(data: *const libc::c_void, length: libc::size_t) -> libc::uint64_t;
}

pub fn rhash64(data: &[u8]) -> u64 {
    let ret = unsafe { hash64(data.as_ptr() as *const libc::c_void, data.len()) };
    return ret;
}

// TODO: add more tests
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_field_obj_payload() {
        let payload = b"_SOURCE_MONOTONIC_TIMESTAMP";
        let expected = 306791107295704799;
        let calc = rhash64(payload);
        assert_eq!(expected, calc);
    }
}
