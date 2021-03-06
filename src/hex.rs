//! Hex encoding and decoding

use std::error::Error;
use std::fmt;

mod avx2;
mod fallback;
mod sse2;

mod support;
mod tests;

/// Invalid nibble
const I: u8 = 255;

/// Hex nibble decoding table
#[rustfmt::skip]
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

// Inverted to make error handle works
const N: u8 = 0;
const HEX_DECODE_64LUT_X30_1: i64 = i64::from_le_bytes([!0x8, !0x9, N, N, N, N, N, N]); // [0-9]
const HEX_DECODE_64LUT_X30_0: i64 =
    i64::from_le_bytes([!0x0, !0x1, !0x2, !0x3, !0x4, !0x5, !0x6, !0x7]);
const HEX_DECODE_64LUT_AZ: i64 = i64::from_le_bytes([N, !0xa, !0xb, !0xc, !0xd, !0xe, !0xf, N]); // [a-z] [A-Z]

#[rustfmt::skip]
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

// (L) least (M) more significant mibble masks
const MN_MASK: i32 = 0xF0F0F0F0u32 as i32;
const LN_MASK: i32 = 0x0F0F0F0F;
const HEX_ENCODE_64LUT_1: i64 = i64::from_be_bytes(*b"fedcba98");
const HEX_ENCODE_64LUT_0: i64 = i64::from_be_bytes(*b"76543210");

/// Allocates `Vec<u8>` of a given length with uninitialized data
#[inline(always)]
fn alloc(length: usize) -> Vec<u8> {
    let mut v = Vec::with_capacity(length);
    unsafe {
        v.set_len(length);
    }
    v
}

#[derive(Debug)]
pub enum DecodeError {
    OddLength,
    InvalidCharAt(usize),
    /// Offset was less than alignment (it needs to be at least equal or greater)
    BadOffset,
}

impl fmt::Display for DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use DecodeError::*;
        match self {
            OddLength => write!(f, "odd length, hexadecimal strings should be always even"),
            InvalidCharAt(pos) => write!(f, "invalid hexadecimal char at {}", pos),
            BadOffset => write!(
                f,
                "not enough offset was given, it needs to be equal or greater than alignment"
            ),
        }
    }
}

impl Error for DecodeError {}

/// Fast hex string decode. No error description is provided
#[no_mangle]
pub fn decode_noerr(input: &str) -> Result<Vec<u8>, ()> {
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    if is_x86_feature_detected!("avx2") {
        return unsafe { avx2::decode(input).map_err(|_| ()) };
    } else if is_x86_feature_detected!("ssse3") {
        return unsafe { sse2::decode(input).map_err(|_| ()) };
    }

    fallback::decode(input).map_err(|_| ())
}

/// Decodes an hex string with all error messages, useful when dealing with
/// recoverable code logic or when a error message is required to facilitate
/// user action.
#[no_mangle]
pub fn decode(input: &str) -> Result<Vec<u8>, DecodeError> {
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    if is_x86_feature_detected!("avx2") {
        return unsafe { avx2::decode(input) };
    } else if is_x86_feature_detected!("ssse3") {
        return unsafe { sse2::decode(input) };
    }

    fallback::decode(input)
}

/// Decodes a hex str starting from `offset` with a given `align`ment.
///
/// The input str will no longer be a valid utf8 string, a byte slice
/// will be returned upon success matching the alignment requirements
///
/// **NOTE** `offset` must be greater or equal to `align`
///
/// ```rust
/// // Padding of 8 (suppose it was read form a file)
/// let mut hex = "--------a1f7d5e8d14f0f76".to_string();
///
/// unsafe {
///     // Decode with padding of 8 and alignment of 8
///     let slice = bintext::hex::decode_aligned(&mut hex, 8, 8).unwrap();
///     // Data is aligned so you can safely do this:
///     let slice: &[u64] = std::slice::from_raw_parts(
///        slice.as_ptr() as *const _,
///        slice.len() / std::mem::size_of::<u64>()
///     );
/// }
/// ```
#[no_mangle]
pub unsafe fn decode_aligned(
    input: &mut str,
    offset: usize,
    align: usize,
) -> Result<&mut [u8], DecodeError> {
    use DecodeError::*;

    // Safe only when if offset is greater or equal than the alignment requirement
    if align > 1 && offset < align {
        Err(BadOffset)?
    }

    let bytes = input.as_bytes_mut();
    let len = bytes.len();
    if (len - offset) & 1 != 0 {
        Err(OddLength)?
    }

    let a = bytes.as_ptr().align_offset(align);
    let output = std::slice::from_raw_parts_mut(bytes.as_mut_ptr().add(a), (len - offset) / 2);

    let input = &bytes[offset..];

    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    if is_x86_feature_detected!("avx2") {
        return avx2::decode_noalloc(input, output).map(|_| output);
    } else if is_x86_feature_detected!("ssse3") {
        return sse2::decode_noalloc(input, output).map(|_| output);
    }

    fallback::decode_noalloc(input, output)?;
    Ok(output)
}

/// Decodes an hex string without allocating any memory
#[no_mangle]
pub fn decode_noalloc(input: &str, output: &mut [u8]) -> Result<(), DecodeError> {
    use DecodeError::*;

    let c = input.len();
    if c & 1 != 0 {
        Err(OddLength)?
    }

    let input = input.as_bytes();

    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    if is_x86_feature_detected!("avx2") {
        return unsafe { avx2::decode_noalloc(input, output) };
    } else if is_x86_feature_detected!("ssse3") {
        return unsafe { sse2::decode_noalloc(input, output) };
    }

    fallback::decode_noalloc(input, output)
}

///////////////////////////////////////////////////////////////////////////////

#[no_mangle]
pub fn encode(input: &[u8]) -> String {
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    if is_x86_feature_detected!("avx2") {
        return unsafe { avx2::encode(input) };
    } else if is_x86_feature_detected!("ssse3") {
        return unsafe { sse2::encode(input) };
    }

    fallback::encode(input)
}

#[cfg(test)]
mod tests_extra {
    const SAMPLES_ALIGNED: [(&'static [u8], &'static str, usize, usize, usize); 5] = [
        (b"\x02\x03\x04\x05", "----02030405", 4, 4, 0),
        (b"\x02\x03\x04\x05", "#----02030405", 5, 4, 0),
        (b"\x02\x03\x04\x05", "#--02030405", 3, 2, 0),
        (b"\x02\x03\x04\x05", "02030405", 0, 1, 0),
        (b"\x02\x03\x04\x05", "...#----02030405", 5, 4, 3),
    ];

    #[test]
    fn decoding_aligned() {
        for (expected, input, offset, align, start) in SAMPLES_ALIGNED.iter() {
            let mut v = input[*start..].to_string();
            let v = unsafe { super::decode_aligned(&mut v, *offset, *align).unwrap() };
            assert_eq!(v, *expected);
            assert_eq!(v.as_ptr().align_offset(*align), 0);
        }
    }
}
