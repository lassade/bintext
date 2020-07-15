/// Invalid nibble
const I: u8 = 255;

/// Hex nibble decoding table
const HEX_NIBBLE_DECODE: [u8; 256] = [
    I,   I,   I,   I,   I,   I,   I,   I,   I,   I,  I,  I,  I,  I,  I,  I,
    I,   I,   I,   I,   I,   I,   I,   I,   I,   I,  I,  I,  I,  I,  I,  I,
    I,   I,   I,   I,   I,   I,   I,   I,   I,   I,  I,  I,  I,  I,  I,  I,
  0x0, 0x1, 0x2, 0x3, 0x4, 0x5, 0x6, 0x7, 0x8, 0x9,  I,  I,  I,  I,  I,  I,
    I, 0xa, 0xb, 0xc, 0xd, 0xe, 0xf,   I,   I,   I,  I,  I,  I,  I,  I,  I,
    I,   I,   I,   I,   I,   I,   I,   I,   I,   I,  I,  I,  I,  I,  I,  I,
    I, 0xa, 0xb, 0xc, 0xd, 0xe, 0xf,   I,   I,   I,  I,  I,  I,  I,  I,  I,
    I,   I,   I,   I,   I,   I,   I,   I,   I,   I,  I,  I,  I,  I,  I,  I,
    I,   I,   I,   I,   I,   I,   I,   I,   I,   I,  I,  I,  I,  I,  I,  I,
    I,   I,   I,   I,   I,   I,   I,   I,   I,   I,  I,  I,  I,  I,  I,  I,
    I,   I,   I,   I,   I,   I,   I,   I,   I,   I,  I,  I,  I,  I,  I,  I,
    I,   I,   I,   I,   I,   I,   I,   I,   I,   I,  I,  I,  I,  I,  I,  I,
    I,   I,   I,   I,   I,   I,   I,   I,   I,   I,  I,  I,  I,  I,  I,  I,
    I,   I,   I,   I,   I,   I,   I,   I,   I,   I,  I,  I,  I,  I,  I,  I,
    I,   I,   I,   I,   I,   I,   I,   I,   I,   I,  I,  I,  I,  I,  I,  I,
    I,   I,   I,   I,   I,   I,   I,   I,   I,   I,  I,  I,  I,  I,  I,  I
];

/// Allocates `Vec<u8>` of a given length with uninialized data
fn alloc(length: usize) -> Vec<u8> {
    let mut v = Vec::with_capacity(length);
    unsafe { v.set_len(length); }
    v
}

#[derive(Debug)]
pub enum DecodeError {
    OddLength,
    InvalidCharAt(usize),
}

#[inline(always)]
fn de(input: &str) -> Result<Vec<u8>, DecodeError> {
    use DecodeError::*;

    let l = input.len();
    if l & 1 != 0 { Err(OddLength)? }

    let mut i = 0;
    let mut v = alloc(l >> 1);
    let p = input.as_bytes();
    for b in v.iter_mut() {
        unsafe {
            let msn = *HEX_NIBBLE_DECODE.get_unchecked(*p.get_unchecked(i) as usize);
            if msn > 0xf { Err(InvalidCharAt(i))? }

            let lsn = *HEX_NIBBLE_DECODE.get_unchecked(*p.get_unchecked(i | 1) as usize);
            if lsn > 0xf { Err(InvalidCharAt(i | 1))? }

            *b = (msn << 4) | lsn;
            i += 2;
        }
    }

    Ok(v)
}

/// Fast hex string decode. No error description is provided
pub fn decode_no(input: &str) -> Result<Vec<u8>, ()> {
    de(input).map_err(|_| ())
}

/// Decodes an hex string with all error messages, useful when dealing with
/// recoverable code logic or when a error message is required to facilitate
/// user action.
pub fn decode(input: &str) -> Result<Vec<u8>, DecodeError> {
    de(input)
}

const HEX_ENCODE: [u8; 512] = 
    *b"000102030405060708090a0b0c0d0e0f\
       101112131415161718191a1b1c1d1e1f\
       202122232425262728292a2b2c2d2e2f\
       303132333435363738393a3b3c3d3e3f\
       404142434445464748494a4b4c4d4e4f\
       505152535455565758595a5b5c5d5e5f\
       606162636465666768696a6b6c6d6e6f\
       707172737475767778797a7b7c7d7e7f\
       808182838485868788898a8b8c8d8e8f\
       909192939495969798999a9b9c9d9e9f\
       a0a1a2a3a4a5a6a7a8a9aaabacadaeaf\
       b0b1b2b3b4b5b6b7b8b9babbbcbdbebf\
       c0c1c2c3c4c5c6c7c8c9cacbcccdcecf\
       d0d1d2d3d4d5d6d7d8d9dadbdcdddedf\
       e0e1e2e3e4e5e6e7e8e9eaebecedeeef\
       f0f1f2f3f4f5f6f7f8f9fafbfcfdfeff";

pub fn encode(input: &[u8]) -> String {
    let mut i = 0usize;
    let mut v = alloc(input.len() << 1);
    unsafe { 
        for b in input {
            let j = (*b as usize) << 1;
            *v.get_unchecked_mut(i) = *HEX_ENCODE.get_unchecked(j);
            *v.get_unchecked_mut(i | 1) = *HEX_ENCODE.get_unchecked(j | 1);
            i += 2;
        }
        String::from_utf8_unchecked(v)
    }
}