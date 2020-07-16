#[cfg(target_arch = "x86")]
use std::arch::x86::*;
#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

use crate::{DecodeError, HEX_ENCODE};

const I: u8 = 255;
const HEX_DECODE_64LUT_X30_1: i64 = i64::from_be_bytes([0x0, 0x1, 0x2, 0x3, 0x4, 0x5, 0x6, 0x7]);
const HEX_DECODE_64LUT_X30_0: i64 = i64::from_be_bytes([0x8, 0x9,   I,   I,   I,   I,   I,   I]); // [0-9]
const HEX_DECODE_64LUT_X40_1: i64 = i64::from_be_bytes([  I, 0xa, 0xb, 0xc, 0xd, 0xe, 0xf,   I]); // [a-z]
const HEX_DECODE_64LUT_X60_1: i64 = i64::from_be_bytes([  I, 0xa, 0xb, 0xc, 0xd, 0xe, 0xf,   I]); // [A-Z]

#[inline(always)]
pub unsafe fn decode(input: &str) -> Result<Vec<u8>, DecodeError> {
    let _lut_x30 = _mm_set_epi64x(HEX_DECODE_64LUT_X30_1, HEX_DECODE_64LUT_X30_0);
    let _lut_x40 = _mm_set_epi64x(HEX_DECODE_64LUT_X40_1, -1);
    let _lut_x60 = _mm_set_epi64x(HEX_DECODE_64LUT_X60_1, -1);

    todo!()
}

// (L) least (M) more significant mibble masks
const MN_MASK: i32 = 0xF0F0F0F0u32 as i32;
const LN_MASK: i32 = 0x0F0F0F0F;
const HEX_ENCODE_64LUT_1: i64 = i64::from_be_bytes(*b"fedcba98");
const HEX_ENCODE_64LUT_0: i64 = i64::from_be_bytes(*b"76543210");

#[inline(always)]
pub unsafe fn encode(input: &[u8]) -> String {
    // Constants
    let lut = _mm_set_epi64x(HEX_ENCODE_64LUT_1, HEX_ENCODE_64LUT_0);
    let umask = _mm_set1_epi32(MN_MASK);
    let lmask = _mm_set1_epi32(LN_MASK);
        
    let c = input.len();
    let mut i = 0;
    let mut p = input.as_ptr() as *const i8;
    let p_end = p.add(c);
    
    // Allocate chunks of 8 bytes with alignment of 8
    let e = c >> 2; // * NOTE: each byte need two other bytes, hence shift left 2 bits
    let mut v = std::mem::ManuallyDrop::new(Vec::<i64>::with_capacity(e));
    v.set_len(e);

    let mut b = v.as_mut_ptr();
    
    while p < p_end {
        let slice = _mm_set_epi8(
            *p.add(15), *p.add(14), *p.add(13), *p.add(12),
            *p.add(11), *p.add(10), *p.add( 9), *p.add( 8),
            *p.add( 7), *p.add( 6), *p.add( 5), *p.add( 4),
            *p.add( 3), *p.add( 2), *p.add( 1), *p,
        );

        let mnibble = {
            let temp = _mm_and_si128(slice, umask);
            // shift left the most significant nibble
            _mm_set_epi64x(
                (_mm_extract_epi64(temp, 1) as u64 >> 4) as i64,
                (_mm_extract_epi64(temp, 0) as u64 >> 4) as i64,
            )
        };
        let lnibble = _mm_and_si128(slice, lmask);
        
        let mhex = _mm_shuffle_epi8(lut, mnibble);
        let lhex = _mm_shuffle_epi8(lut, lnibble);
        
        let hex0 = _mm_unpacklo_epi8(mhex, lhex);
        let hex1 = _mm_unpackhi_epi8(mhex, lhex);
        
        *b = _mm_extract_epi64(hex0, 0);
        *b.add(1) = _mm_extract_epi64(hex0, 1);
        *b.add(2) = _mm_extract_epi64(hex1, 0);
        *b.add(3) = _mm_extract_epi64(hex1, 1);

        i += 16;
        p = p.add(16);
        b = b.add(4);
    }

    // loop through the remaining 15 or less bytes
    assert!((c - i) < 16, "{} left bytes", c - 1);

    let mut b = b as *mut u8;
    while i < c {
        let j = (*b as usize) << 1;
        *b = *HEX_ENCODE.get_unchecked(j);
        *b.add(1) = *HEX_ENCODE.get_unchecked(j | 1);
        
        b = b.add(2);
        i += 1;
    }

    let e = e << 3; // each i64 have 8 bytes
    String::from_raw_parts(v.as_mut_ptr() as *mut u8, e, e)
}