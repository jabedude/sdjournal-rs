pub fn rhash64(data: &[u8]) -> u64 {
    let (a, b) = hashlittle2(data, 0, 0);
    ((a as u64) << 32u64) | (b as u64)
}

/// hash_size returns the number of elements that n bits addressing can cover
///
/// # arguments
/// * `n` the size of the hash in number of bits
#[inline]
pub fn hash_size(n: u8) -> u32 {
    1 << n
}

/// hash_mask generates a binary mask for a hash of size n
///
/// # arguments
/// * `n` the size of the hash in number of bits
#[inline]
pub fn hash_mask(n: u8) -> u32 {
    hash_size(n) - 1
}

#[inline]
fn rot(x: u32, k: u8) -> u32 {
    x << k | x >> (32 - k)
}

#[inline]
fn mix(a: &mut u32, b: &mut u32, c: &mut u32) {
    *a = a.wrapping_sub(*c);
    *a ^= rot(*c, 4);
    *c = c.wrapping_add(*b);
    *b = b.wrapping_sub(*a);
    *b ^= rot(*a, 6);
    *a = a.wrapping_add(*c);
    *c = c.wrapping_sub(*b);
    *c ^= rot(*b, 8);
    *b = b.wrapping_add(*a);
    *a = a.wrapping_sub(*c);
    *a ^= rot(*c, 16);
    *c = c.wrapping_add(*b);
    *b = b.wrapping_sub(*a);
    *b ^= rot(*a, 19);
    *a = a.wrapping_add(*c);
    *c = c.wrapping_sub(*b);
    *c ^= rot(*b, 4);
    *b = b.wrapping_add(*a);
}

#[inline]
fn final_(a: &mut u32, b: &mut u32, c: &mut u32) {
    *c ^= *b;
    *c = c.wrapping_sub(rot(*b, 14));
    *a ^= *c;
    *a = a.wrapping_sub(rot(*c, 11));
    *b ^= *a;
    *b = b.wrapping_sub(rot(*a, 25));
    *c ^= *b;
    *c = c.wrapping_sub(rot(*b, 16));
    *a ^= *c;
    *a = a.wrapping_sub(rot(*c, 4));
    *b ^= *a;
    *b = b.wrapping_sub(rot(*a, 14));
    *c ^= *b;
    *c = c.wrapping_sub(rot(*b, 24));
}

/// Hashes an array of u32's
///
/// # Arguments
/// * `k` - the key, an array of u32 values
/// * `init_val` - the previous hash or an arbitrary value
#[inline]
pub fn hashword(k: &[u32], init_val: u32) -> u32 {
    let mut a: u32;
    let mut b: u32;
    let mut c: u32;
    let mut length = k.len() as u32;
    let mut k = k;

    /* Set up the internal state */
    a = 0xdeadbeef + (length << 2) + init_val;
    b = a;
    c = a;

    // handle most of the key
    while length > 3 {
        a = a.wrapping_add(k[0]);
        b = b.wrapping_add(k[1]);
        c = c.wrapping_add(k[2]);
        mix(&mut a, &mut b, &mut c);
        length -= 3;
        k = &k[3..];
    }

    // handle the last 3 u32's
    match length {
        3 => {
            c = c.wrapping_add(k[2]);
            b = b.wrapping_add(k[1]);
            a = a.wrapping_add(k[0]);
        }
        2 => {
            b = b.wrapping_add(k[1]);
            a = a.wrapping_add(k[0]);
        }
        1 => a = a.wrapping_add(k[0]),
        _ => (),
    }

    final_(&mut a, &mut b, &mut c);
    c
}

/// Hashword2 hashes a slice of u32 values taking as input 2 seeds
/// and outputs 2 independent hashes
///
/// # Arguments
/// * `k` - the key, a slice of u32 values
/// * `pc` - primary seed
/// * `pb` - secondary seed
#[inline]
pub fn hashword2(k: &[u32], pc: u32, pb: u32) -> (u32, u32) {
    let mut a: u32;
    let mut b: u32;
    let mut c: u32;
    let mut length = k.len() as u32;
    let mut k = k;

    // Set up the internal state
    a = 0xdeadbeefu32.wrapping_add(length << 2).wrapping_add(pc);
    b = a;
    c = a;
    c = c.wrapping_add(pb);

    // handle most of the key
    while length > 3 {
        a = a.wrapping_add(k[0]);
        b = b.wrapping_add(k[1]);
        c = c.wrapping_add(k[2]);
        mix(&mut a, &mut b, &mut c);
        length -= 3;
        k = &k[3..];
    }

    // handle the last 3 uint32_t's
    match length {
        3 => {
            c = c.wrapping_add(k[2]);
            b = b.wrapping_add(k[1]);
            a = a.wrapping_add(k[0]);
        }
        2 => {
            b = b.wrapping_add(k[1]);
            a = a.wrapping_add(k[0]);
        }
        1 => a = a.wrapping_add(k[0]),
        _ => (),
    }

    final_(&mut a, &mut b, &mut c);

    // report the result
    (c, b)
}

#[inline]
pub fn hashlittle(key: &[u8], init_val: u32) -> u32 {
    let mut a: u32;
    let mut b: u32;
    let mut c: u32;
    let mut length = key.len() as u32;
    let is_little_endian = cfg!(target_endian = "little");

    // Set up the internal state
    a = 0xdeadbeefu32.wrapping_add(length).wrapping_add(init_val);
    b = a;
    c = a;

    let alignment = key.as_ptr() as u32;

    if is_little_endian && ((alignment & 0x3) == 0) {
        let mut k: &[u32] = unsafe { std::mem::transmute(key) }; // read 32 bit chunks

        // all but last block: aligned reads and affect 32 bits of (a,b,c)
        while length > 12 {
            a = a.wrapping_add(k[0]);
            b = b.wrapping_add(k[1]);
            c = c.wrapping_add(k[2]);
            mix(&mut a, &mut b, &mut c);
            length -= 12;
            k = &k[3..];
        }

        // handle the last (probably partial) block

        /*
         * "k[2]&0xffffff" actually reads beyond the end of the string, but
         * then masks off the part it's not allowed to read.  Because the
         * string is aligned, the masked-off tail is in the same word as the
         * rest of the string.  Every machine with memory protection I've seen
         * does it on word boundaries, so is OK with this.  But VALGRIND will
         * still catch it and complain.  The masking trick does make the hash
         * noticably faster for short strings (like English words).
         */

        match length {
            12 => {
                c = c.wrapping_add(k[2]);
                b = b.wrapping_add(k[1]);
                a = a.wrapping_add(k[0]);
            }

            11 => {
                c = c.wrapping_add(k[2] & 0xffffff);
                b = b.wrapping_add(k[1]);
                a = a.wrapping_add(k[0]);
            }

            10 => {
                c = c.wrapping_add(k[2] & 0xffff);
                b = b.wrapping_add(k[1]);
                a = a.wrapping_add(k[0]);
            }

            9 => {
                c = c.wrapping_add(k[2] & 0xff);
                b = b.wrapping_add(k[1]);
                a = a.wrapping_add(k[0]);
            }

            8 => {
                b = b.wrapping_add(k[1]);
                a = a.wrapping_add(k[0]);
            }

            7 => {
                b = b.wrapping_add(k[1] & 0xffffff);
                a = a.wrapping_add(k[0]);
            }

            6 => {
                b = b.wrapping_add(k[1] & 0xffff);
                a = a.wrapping_add(k[0]);
            }

            5 => {
                b = b.wrapping_add(k[1] & 0xff);
                a = a.wrapping_add(k[0]);
            }

            4 => {
                a = a.wrapping_add(k[0]);
            }

            3 => a = a.wrapping_add(k[0] & 0xffffff),

            2 => a = a.wrapping_add(k[0] & 0xffff),

            1 => a = a.wrapping_add(k[0] & 0xff),

            _ => return c,
        }
    } else if is_little_endian && ((alignment & 0x1) == 0) {
        let mut k: &[u16] = unsafe { std::mem::transmute(key) };

        // all but last block: aligned reads and different mixing
        while length > 12 {
            a = a.wrapping_add((k[0] as u32) + ((k[1] as u32) << 16));
            b = b.wrapping_add((k[2] as u32) + ((k[3] as u32) << 16));
            c = c.wrapping_add((k[4] as u32) + ((k[5] as u32) << 16));
            mix(&mut a, &mut b, &mut c);
            length -= 12;
            k = &k[6..];
        }

        // handle the last (probably partial) block
        let k8: &[u8] = unsafe { std::mem::transmute(k) };
        match length {
            12 => {
                c = c.wrapping_add((k[4] as u32) + ((k[5] as u32) << 16));
                b = b.wrapping_add((k[2] as u32) + ((k[3] as u32) << 16));
                a = a.wrapping_add((k[0] as u32) + ((k[1] as u32) << 16));
            }

            11 => {
                c = c.wrapping_add((k8[10] as u32) << 16);
                b = b.wrapping_add((k[2] as u32) + ((k[3] as u32) << 16));
                a = a.wrapping_add((k[0] as u32) + ((k[1] as u32) << 16));
            }

            10 => {
                c = c.wrapping_add(k[4] as u32);
                b = b.wrapping_add((k[2] as u32) + ((k[3] as u32) << 16));
                a = a.wrapping_add((k[0] as u32) + ((k[1] as u32) << 16));
            }

            9 => {
                c = c.wrapping_add(k8[8] as u32);
                b = b.wrapping_add((k[2] as u32) + ((k[3] as u32) << 16));
                a = a.wrapping_add((k[0] as u32) + ((k[1] as u32) << 16));
            }

            8 => {
                b = b.wrapping_add((k[2] as u32) + ((k[3] as u32) << 16));
                a = a.wrapping_add((k[0] as u32) + ((k[1] as u32) << 16));
            }

            7 => {
                b = b.wrapping_add((k8[6] as u32) << 16);
                b = b.wrapping_add(k[2] as u32);
                a = a.wrapping_add((k[0] as u32) + ((k[1] as u32) << 16));
            }

            6 => {
                b = b.wrapping_add(k[2] as u32);
                a = a.wrapping_add((k[0] as u32) + ((k[1] as u32) << 16));
            }

            5 => {
                b = b.wrapping_add(k8[4] as u32);
                a = a.wrapping_add((k[0] as u32) + ((k[1] as u32) << 16));
            }

            4 => {
                a = a.wrapping_add((k[0] as u32) + ((k[1] as u32) << 16));
            }

            3 => {
                a = a.wrapping_add((k8[2] as u32) << 16);
                a = a.wrapping_add(k[0] as u32);
            }

            2 => a = a.wrapping_add(k[0] as u32),

            1 => a = a.wrapping_add(k8[0] as u32),

            _ => return c,
        }
    } else {
        // need to read the key one byte at a time
        let mut k: &[u8] = unsafe { std::mem::transmute(key) };

        // all but the last block: affect some 32 bits of (a,b,c)
        while length > 12 {
            a = a.wrapping_add(k[0] as u32);
            a = a.wrapping_add((k[1] as u32) << 8);
            a = a.wrapping_add((k[2] as u32) << 16);
            a = a.wrapping_add((k[3] as u32) << 24);
            b = b.wrapping_add(k[4] as u32);
            b = b.wrapping_add((k[5] as u32) << 8);
            b = b.wrapping_add((k[6] as u32) << 16);
            b = b.wrapping_add((k[7] as u32) << 24);
            c = c.wrapping_add(k[8] as u32);
            c = c.wrapping_add((k[9] as u32) << 8);
            c = c.wrapping_add((k[10] as u32) << 16);
            c = c.wrapping_add((k[11] as u32) << 24);
            mix(&mut a, &mut b, &mut c);
            length -= 12;
            k = &k[12..];
        }

        // last block: affect all 32 bits of (c)
        match length {
            12 => {
                c = c.wrapping_add((k[11] as u32) << 24);
                c = c.wrapping_add((k[10] as u32) << 16);
                c = c.wrapping_add((k[9] as u32) << 8);
                c = c.wrapping_add(k[8] as u32);
                b = b.wrapping_add((k[7] as u32) << 24);
                b = b.wrapping_add((k[6] as u32) << 16);
                b = b.wrapping_add((k[5] as u32) << 8);
                b = b.wrapping_add(k[4] as u32);
                a = a.wrapping_add((k[3] as u32) << 24);
                a = a.wrapping_add((k[2] as u32) << 16);
                a = a.wrapping_add((k[1] as u32) << 8);
                a = a.wrapping_add(k[0] as u32);
            }

            11 => {
                c = c.wrapping_add((k[10] as u32) << 16);
                c = c.wrapping_add((k[9] as u32) << 8);
                c = c.wrapping_add(k[8] as u32);
                b = b.wrapping_add((k[7] as u32) << 24);
                b = b.wrapping_add((k[6] as u32) << 16);
                b = b.wrapping_add((k[5] as u32) << 8);
                b = b.wrapping_add(k[4] as u32);
                a = a.wrapping_add((k[3] as u32) << 24);
                a = a.wrapping_add((k[2] as u32) << 16);
                a = a.wrapping_add((k[1] as u32) << 8);
                a = a.wrapping_add(k[0] as u32);
            }

            10 => {
                c = c.wrapping_add((k[9] as u32) << 8);
                c = c.wrapping_add(k[8] as u32);
                b = b.wrapping_add((k[7] as u32) << 24);
                b = b.wrapping_add((k[6] as u32) << 16);
                b = b.wrapping_add((k[5] as u32) << 8);
                b = b.wrapping_add(k[4] as u32);
                a = a.wrapping_add((k[3] as u32) << 24);
                a = a.wrapping_add((k[2] as u32) << 16);
                a = a.wrapping_add((k[1] as u32) << 8);
                a = a.wrapping_add(k[0] as u32);
            }

            9 => {
                c = c.wrapping_add(k[8] as u32);
                b = b.wrapping_add((k[7] as u32) << 24);
                b = b.wrapping_add((k[6] as u32) << 16);
                b = b.wrapping_add((k[5] as u32) << 8);
                b = b.wrapping_add(k[4] as u32);
                a = a.wrapping_add((k[3] as u32) << 24);
                a = a.wrapping_add((k[2] as u32) << 16);
                a = a.wrapping_add((k[1] as u32) << 8);
                a = a.wrapping_add(k[0] as u32);
            }

            8 => {
                b = b.wrapping_add((k[7] as u32) << 24);
                b = b.wrapping_add((k[6] as u32) << 16);
                b = b.wrapping_add((k[5] as u32) << 8);
                b = b.wrapping_add(k[4] as u32);
                a = a.wrapping_add((k[3] as u32) << 24);
                a = a.wrapping_add((k[2] as u32) << 16);
                a = a.wrapping_add((k[1] as u32) << 8);
                a = a.wrapping_add(k[0] as u32);
            }

            7 => {
                b = b.wrapping_add((k[6] as u32) << 16);
                b = b.wrapping_add((k[5] as u32) << 8);
                b = b.wrapping_add(k[4] as u32);
                a = a.wrapping_add((k[3] as u32) << 24);
                a = a.wrapping_add((k[2] as u32) << 16);
                a = a.wrapping_add((k[1] as u32) << 8);
                a = a.wrapping_add(k[0] as u32);
            }

            6 => {
                b = b.wrapping_add((k[5] as u32) << 8);
                b = b.wrapping_add(k[4] as u32);
                a = a.wrapping_add((k[3] as u32) << 24);
                a = a.wrapping_add((k[2] as u32) << 16);
                a = a.wrapping_add((k[1] as u32) << 8);
                a = a.wrapping_add(k[0] as u32);
            }

            5 => {
                b = b.wrapping_add(k[4] as u32);
                a = a.wrapping_add((k[3] as u32) << 24);
                a = a.wrapping_add((k[2] as u32) << 16);
                a = a.wrapping_add((k[1] as u32) << 8);
                a = a.wrapping_add(k[0] as u32);
            }

            4 => {
                a = a.wrapping_add((k[3] as u32) << 24);
                a = a.wrapping_add((k[2] as u32) << 16);
                a = a.wrapping_add((k[1] as u32) << 8);
                a = a.wrapping_add(k[0] as u32);
            }

            3 => {
                a = a.wrapping_add((k[2] as u32) << 16);
                a = a.wrapping_add((k[1] as u32) << 8);
                a = a.wrapping_add(k[0] as u32);
            }

            2 => {
                a = a.wrapping_add((k[1] as u32) << 8);
                a = a.wrapping_add(k[0] as u32);
            }

            1 => a = a.wrapping_add(k[0] as u32),

            _ => return c,
        }
    }

    final_(&mut a, &mut b, &mut c);
    c
}

/// hashlittle2 return 2 32-bit hash values
///
/// This is identical to hashlittle(), except it returns two 32-bit hash
/// values instead of just one.  This is good enough for hash table
/// lookup with 2^^64 buckets, or if you want a second hash if you're not
/// happy with the first, or if you want a probably-unique 64-bit ID for
/// the key.  *pc is better mixed than *pb, so use *pc first.  If you want
/// a 64-bit value do something like "*pc + (((uint64_t)*pb)<<32)".
#[inline]
pub fn hashlittle2(key: &[u8], pc: u32, pb: u32) -> (u32, u32) {
    let mut a: u32;
    let mut b: u32;
    let mut c: u32; /* internal state */
    let is_little_endian = cfg!(target_endian = "little");
    let mut length = key.len() as u32;

    // Set up the internal state
    a = 0xdeadbeefu32.wrapping_add(length).wrapping_add(pc);
    b = a;
    c = a;
    c = c.wrapping_add(pb);

    let alignment = key.as_ptr() as u32;

    if is_little_endian && ((alignment & 0x3) == 0) {
        let mut k: &[u32] = unsafe { std::mem::transmute(key) }; // read 32-bit chunks

        // all but last block: aligned reads and affect 32 bits of (a,b,c)
        while length > 12 {
            a = a.wrapping_add(k[0]);
            b = b.wrapping_add(k[1]);
            c = c.wrapping_add(k[2]);
            mix(&mut a, &mut b, &mut c);
            length -= 12;
            k = &k[3..];
        }

        // handle the last (probably partial) block
        //
        // "k[2]&0xffffff" actually reads beyond the end of the string, but
        // then masks off the part it's not allowed to read.  Because the
        // string is aligned, the masked-off tail is in the same word as the
        // rest of the string.  Every machine with memory protection I've seen
        // does it on word boundaries, so is OK with this.  But VALGRIND will
        // still catch it and complain.  The masking trick does make the hash
        // noticably faster for short strings (like English words).

        match length {
            12 => {
                c = c.wrapping_add(k[2]);
                b = b.wrapping_add(k[1]);
                a = a.wrapping_add(k[0]);
            }

            11 => {
                c = c.wrapping_add(k[2] & 0xffffff);
                b = b.wrapping_add(k[1]);
                a = a.wrapping_add(k[0]);
            }

            10 => {
                c = c.wrapping_add(k[2] & 0xffff);
                b = b.wrapping_add(k[1]);
                a = a.wrapping_add(k[0]);
            }

            9 => {
                c = c.wrapping_add(k[2] & 0xff);
                b = b.wrapping_add(k[1]);
                a = a.wrapping_add(k[0]);
            }

            8 => {
                b = b.wrapping_add(k[1]);
                a = a.wrapping_add(k[0]);
            }

            7 => {
                b = b.wrapping_add(k[1] & 0xffffff);
                a = a.wrapping_add(k[0]);
            }

            6 => {
                b = b.wrapping_add(k[1] & 0xffff);
                a = a.wrapping_add(k[0]);
            }

            5 => {
                b = b.wrapping_add(k[1] & 0xff);
                a = a.wrapping_add(k[0]);
            }

            4 => a = a.wrapping_add(k[0]),

            3 => a = a.wrapping_add(k[0] & 0xffffff),

            2 => a = a.wrapping_add(k[0] & 0xffff),

            1 => a = a.wrapping_add(k[0] & 0xff),

            _ => return (c, b), // zero length strings require no mixing
        }
    } else if is_little_endian && ((alignment & 0x1) == 0) {
        let mut k: &[u16] = unsafe { std::mem::transmute(key) }; // read 16-bit chunks

        // all but last block: aligned reads and different mixing
        while length > 12 {
            a = a.wrapping_add(k[0] as u32 + ((k[1] as u32) << 16));
            b = b.wrapping_add(k[2] as u32 + ((k[3] as u32) << 16));
            c = c.wrapping_add(k[4] as u32 + ((k[5] as u32) << 16));
            mix(&mut a, &mut b, &mut c);
            length -= 12;
            k = &k[6..];
        }

        // handle the last (probably partial) block
        let k8: &[u8] = unsafe { std::mem::transmute(k) };
        match length {
            12 => {
                c = c.wrapping_add(k[4] as u32 + ((k[5] as u32) << 16));
                b = b.wrapping_add(k[2] as u32 + ((k[3] as u32) << 16));
                a = a.wrapping_add(k[0] as u32 + ((k[1] as u32) << 16));
            }

            11 => {
                c = c.wrapping_add((k8[10] as u32) << 16);
                c = c.wrapping_add(k[4] as u32);
                b = b.wrapping_add(k[2] as u32 + ((k[3] as u32) << 16));
                a = a.wrapping_add(k[0] as u32 + ((k[1] as u32) << 16));
            }

            10 => {
                c = c.wrapping_add(k[4] as u32);
                b = b.wrapping_add(k[2] as u32 + ((k[3] as u32) << 16));
                a = a.wrapping_add(k[0] as u32 + ((k[1] as u32) << 16));
            }

            9 => {
                c = c.wrapping_add(k8[8] as u32);
                b = b.wrapping_add(k[2] as u32 + ((k[3] as u32) << 16));
                a = a.wrapping_add(k[0] as u32 + ((k[1] as u32) << 16));
            }

            8 => {
                b = b.wrapping_add(k[2] as u32 + ((k[3] as u32) << 16));
                a = a.wrapping_add(k[0] as u32 + ((k[1] as u32) << 16));
            }

            7 => {
                b = b.wrapping_add((k8[6] as u32) << 16);
                b = b.wrapping_add(k[2] as u32);
                a = a.wrapping_add(k[0] as u32 + ((k[1] as u32) << 16));
            }

            6 => {
                b = b.wrapping_add(k[2] as u32);
                a = a.wrapping_add(k[0] as u32 + ((k[1] as u32) << 16));
            }

            5 => {
                b = b.wrapping_add(k8[4] as u32);
                a = a.wrapping_add(k[0] as u32 + ((k[1] as u32) << 16));
            }

            4 => {
                a = a.wrapping_add(k[0] as u32 + ((k[1] as u32) << 16));
            }

            3 => {
                a = a.wrapping_add((k8[2] as u32) << 16);
                a = a.wrapping_add(k[0] as u32);
            }

            2 => a = a.wrapping_add(k[0] as u32),

            1 => a = a.wrapping_add(k8[0] as u32),

            _ => return (c, b),
        }
    } else {
        // need to read the key one byte at a time
        let mut k: &[u8] = unsafe { std::mem::transmute(key) };

        // all but the last block: affect some 32 bits of (a,b,c)
        while length > 12 {
            a = a.wrapping_add(k[0] as u32);
            a = a.wrapping_add((k[1] as u32) << 8);
            a = a.wrapping_add((k[2] as u32) << 16);
            a = a.wrapping_add((k[3] as u32) << 24);
            b = b.wrapping_add(k[4] as u32);
            b = b.wrapping_add((k[5] as u32) << 8);
            b = b.wrapping_add((k[6] as u32) << 16);
            b = b.wrapping_add((k[7] as u32) << 24);
            c = c.wrapping_add(k[8] as u32);
            c = c.wrapping_add((k[9] as u32) << 8);
            c = c.wrapping_add((k[10] as u32) << 16);
            c = c.wrapping_add((k[11] as u32) << 24);
            mix(&mut a, &mut b, &mut c);
            length -= 12;
            k = &k[12..];
        }

        // last block: affect all 32 bits of (c)
        match length {
            12 => {
                c = c.wrapping_add((k[11] as u32) << 24);
                c = c.wrapping_add((k[10] as u32) << 16);
                c = c.wrapping_add((k[9] as u32) << 8);
                c = c.wrapping_add(k[8] as u32);
                b = b.wrapping_add((k[7] as u32) << 24);
                b = b.wrapping_add((k[6] as u32) << 16);
                b = b.wrapping_add((k[5] as u32) << 8);
                b = b.wrapping_add(k[4] as u32);
                a = a.wrapping_add((k[3] as u32) << 24);
                a = a.wrapping_add((k[2] as u32) << 16);
                a = a.wrapping_add((k[1] as u32) << 8);
                a = a.wrapping_add(k[0] as u32);
            }

            11 => {
                c = c.wrapping_add((k[10] as u32) << 16);
                c = c.wrapping_add((k[9] as u32) << 8);
                c = c.wrapping_add(k[8] as u32);
                b = b.wrapping_add((k[7] as u32) << 24);
                b = b.wrapping_add((k[6] as u32) << 16);
                b = b.wrapping_add((k[5] as u32) << 8);
                b = b.wrapping_add(k[4] as u32);
                a = a.wrapping_add((k[3] as u32) << 24);
                a = a.wrapping_add((k[2] as u32) << 16);
                a = a.wrapping_add((k[1] as u32) << 8);
                a = a.wrapping_add(k[0] as u32);
            }

            10 => {
                c = c.wrapping_add((k[9] as u32) << 8);
                c = c.wrapping_add(k[8] as u32);
                b = b.wrapping_add((k[7] as u32) << 24);
                b = b.wrapping_add((k[6] as u32) << 16);
                b = b.wrapping_add((k[5] as u32) << 8);
                b = b.wrapping_add(k[4] as u32);
                a = a.wrapping_add((k[3] as u32) << 24);
                a = a.wrapping_add((k[2] as u32) << 16);
                a = a.wrapping_add((k[1] as u32) << 8);
                a = a.wrapping_add(k[0] as u32);
            }

            9 => {
                c = c.wrapping_add(k[8] as u32);
                b = b.wrapping_add((k[7] as u32) << 24);
                b = b.wrapping_add((k[6] as u32) << 16);
                b = b.wrapping_add((k[5] as u32) << 8);
                b = b.wrapping_add(k[4] as u32);
                a = a.wrapping_add((k[3] as u32) << 24);
                a = a.wrapping_add((k[2] as u32) << 16);
                a = a.wrapping_add((k[1] as u32) << 8);
                a = a.wrapping_add(k[0] as u32);
            }

            8 => {
                b = b.wrapping_add((k[7] as u32) << 24);
                b = b.wrapping_add((k[6] as u32) << 16);
                b = b.wrapping_add((k[5] as u32) << 8);
                b = b.wrapping_add(k[4] as u32);
                a = a.wrapping_add((k[3] as u32) << 24);
                a = a.wrapping_add((k[2] as u32) << 16);
                a = a.wrapping_add((k[1] as u32) << 8);
                a = a.wrapping_add(k[0] as u32);
            }

            7 => {
                b = b.wrapping_add((k[6] as u32) << 16);
                b = b.wrapping_add((k[5] as u32) << 8);
                b = b.wrapping_add(k[4] as u32);
                a = a.wrapping_add((k[3] as u32) << 24);
                a = a.wrapping_add((k[2] as u32) << 16);
                a = a.wrapping_add((k[1] as u32) << 8);
                a = a.wrapping_add(k[0] as u32);
            }

            6 => {
                b = b.wrapping_add((k[5] as u32) << 8);
                b = b.wrapping_add(k[4] as u32);
                a = a.wrapping_add((k[3] as u32) << 24);
                a = a.wrapping_add((k[2] as u32) << 16);
                a = a.wrapping_add((k[1] as u32) << 8);
                a = a.wrapping_add(k[0] as u32);
            }

            5 => {
                b = b.wrapping_add(k[4] as u32);
                a = a.wrapping_add((k[3] as u32) << 24);
                a = a.wrapping_add((k[2] as u32) << 16);
                a = a.wrapping_add((k[1] as u32) << 8);
                a = a.wrapping_add(k[0] as u32);
            }

            4 => {
                a = a.wrapping_add((k[3] as u32) << 24);
                a = a.wrapping_add((k[2] as u32) << 16);
                a = a.wrapping_add((k[1] as u32) << 8);
                a = a.wrapping_add(k[0] as u32);
            }

            3 => {
                a = a.wrapping_add((k[2] as u32) << 16);
                a = a.wrapping_add((k[1] as u32) << 8);
                a = a.wrapping_add(k[0] as u32);
            }

            2 => {
                a = a.wrapping_add((k[1] as u32) << 8);
                a = a.wrapping_add(k[0] as u32);
            }

            1 => a = a.wrapping_add(k[0] as u32),
            _ => return (c, b), // zero length strings require no mixing
        }
    }

    final_(&mut a, &mut b, &mut c);
    (c, b)
}

#[cfg(test)]
mod tests {
    use super::{rhash64, hashlittle, hashlittle2};

    #[test]
    fn test_hash_field_obj_payload() {
        let payload = b"_SOURCE_MONOTONIC_TIMESTAMP";
        let expected = 306791107295704799;
        let calc = rhash64(payload);
        assert_eq!(expected, calc);
    }

    #[test]
    fn test_hashlittle() {
        let mut c: u32;
        c = 0;
        c = hashlittle(b"", c);
        assert_eq!(c, 0xdeadbeef);

        c = 0xdeadbeef;
        c = hashlittle(b"", c);
        assert_eq!(c, 0xbd5b7dde);

        c = 0;
        c = hashlittle(b"Four score and seven years ago", c);
        assert_eq!(c, 0x17770551);

        c = 1;
        c = hashlittle(b"Four score and seven years ago", c);
        assert_eq!(c, 0xcd628161);
    }

    #[test]
    fn test_hashlittle2() {
        let (c, b) = hashlittle2(b"", 0, 0);
        assert_eq!(c, 0xdeadbeef);
        assert_eq!(b, 0xdeadbeef);

        let (c, b) = hashlittle2(b"", 0, 0xdeadbeef);
        assert_eq!(c, 0xbd5b7dde);
        assert_eq!(b, 0xdeadbeef);

        let (c, b) = hashlittle2(b"", 0xdeadbeef, 0xdeadbeef);
        assert_eq!(c, 0x9c093ccd);
        assert_eq!(b, 0xbd5b7dde);

        let (c, b) = hashlittle2(b"Four score and seven years ago", 0, 0);
        assert_eq!(c, 0x17770551);
        assert_eq!(b, 0xce7226e6);

        let (c, b) = hashlittle2(b"Four score and seven years ago", 0, 1);
        assert_eq!(c, 0xe3607cae);
        assert_eq!(b, 0xbd371de4);

        let (c, b) = hashlittle2(b"Four score and seven years ago", 1, 0);
        assert_eq!(c, 0xcd628161);
        assert_eq!(b, 0x6cbea4b3);
    }

    #[test]
    fn test_hashing_repeatedly() {
        let buf: [u8; 1] = [!0];
        let mut h: u32 = 0;

        h = hashlittle(&buf[0..0], h);
        assert_eq!(h, 0xdeadbeef);

        h = hashlittle(&buf[0..0], h);
        assert_eq!(h, 0xbd5b7dde);

        h = hashlittle(&buf[0..0], h);
        assert_eq!(h, 0x9c093ccd);

        h = hashlittle(&buf[0..0], h);
        assert_eq!(h, 0x7ab6fbbc);

        h = hashlittle(&buf[0..0], h);
        assert_eq!(h, 0x5964baab);

        h = hashlittle(&buf[0..0], h);
        assert_eq!(h, 0x3812799a);

        h = hashlittle(&buf[0..0], h);
        assert_eq!(h, 0x16c03889);

        h = hashlittle(&buf[0..0], h);
        assert_eq!(h, 0xf56df778);
    }
}
