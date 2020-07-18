mod sse2;
mod fallback;

mod tests;

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

/// Allocates `Vec<u8>` of a given length with uninialized data
#[inline(always)]
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

/// Fast hex string decode. No error description is provided
#[no_mangle]
pub fn decode_no(input: &str) -> Result<Vec<u8>, ()> {
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    if is_x86_feature_detected!("sse2") && is_x86_feature_detected!("ssse3") {
        return unsafe { sse2::decode(input).map_err(|_| ()) };
    }

    fallback::decode(input).map_err(|_| ())
}

/// Decodes an hex string with all error messages, useful when dealing with
/// recoverable code logic or when a error message is required to facilitate
/// user action.
#[no_mangle]
#[allow(unreachable_code)]
pub fn decode(input: &str) -> Result<Vec<u8>, DecodeError> {
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    if is_x86_feature_detected!("sse2") && is_x86_feature_detected!("ssse3") {
        return unsafe { sse2::decode(input) };
    }

    fallback::decode(input)
}

///////////////////////////////////////////////////////////////////////////////

#[no_mangle]
#[allow(unreachable_code)]
pub fn encode(input: &[u8]) -> String {
    // TODO: AVX implementation

    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    if is_x86_feature_detected!("sse2") && is_x86_feature_detected!("ssse3")  {
        return unsafe { sse2::encode(input) };
    }

    fallback::encode(input)
}
