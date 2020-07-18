#![allow(dead_code)]

#[cfg(target_arch = "x86")]
use std::arch::x86::*;
#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

use std::alloc::{alloc, Layout};
use std::mem::align_of;
use super::*;


///////////////////////////////////////////////////////////////////////////////


// Inverted to make error handle works
const I: u8 = 0;
const HEX_DECODE_64LUT_X30_1: i64 = i64::from_le_bytes([!0x8, !0x9,   I,   I,   I,   I,   I,   I]); // [0-9]
const HEX_DECODE_64LUT_X30_0: i64 = i64::from_le_bytes([!0x0, !0x1, !0x2, !0x3, !0x4, !0x5, !0x6, !0x7]);
const HEX_DECODE_64LUT_AZ: i64 = i64::from_le_bytes([  I, !0xa, !0xb, !0xc, !0xd, !0xe, !0xf,   I]); // [a-z] [A-Z]

#[inline(always)]
pub unsafe fn decode(input: &str) -> Result<Vec<u8>, DecodeError> {
    use DecodeError::*;

    // Input check
    let c = input.len();
    if c & 1 != 0 { Err(OddLength)? }

    // Constants
    let lutx3 = _mm256_set_epi64x(
        HEX_DECODE_64LUT_X30_1, HEX_DECODE_64LUT_X30_0,
        HEX_DECODE_64LUT_X30_1, HEX_DECODE_64LUT_X30_0
    );
    let lutx4and6 = _mm256_set_epi64x(0, HEX_DECODE_64LUT_AZ, 0, HEX_DECODE_64LUT_AZ);
    
    let on = _mm256_set1_epi8(-1);
    let x30 = _mm256_set1_epi8(0x30u8 as i8);
    let x3f = _mm256_set1_epi8(0x3fu8 as i8);
    let x40 = _mm256_set1_epi8(0x40u8 as i8);
    let x4f = _mm256_set1_epi8(0x4fu8 as i8);
    let x5f = _mm256_set1_epi8(0x5fu8 as i8);
    let x60 = _mm256_set1_epi8(0x60u8 as i8);
    
    let m = _mm256_set1_epi16(0x00FFu16 as i16);
    let idec = _mm256_set_epi64x(
        0x0f_0d_0b_09_07_05_03_01u64 as i64, -1,
        0x0f_0d_0b_09_07_05_03_01u64 as i64, -1
    );
    let tmpsll = _mm_set1_epi64x(12);
    
    // Input pointers
    let mut p = input.as_ptr() as *const i8;
    let p_end = p.add(c);
    
    // Allocate chunks of 8 bytes with alignment of 8
    let e = c >> 1;
    let v = alloc(Layout::from_size_align(e, align_of::<i64>()).unwrap());
    let mut b = v as *mut i64;

    // Main loop loop
    while p.offset(31) < p_end {
        let slice = _mm256_set_epi8(
            *p.add(31), *p.add(30), *p.add(29), *p.add(28),
            *p.add(27), *p.add(26), *p.add(25), *p.add(24),
            *p.add(23), *p.add(22), *p.add(21), *p.add(20),
            *p.add(19), *p.add(18), *p.add(17), *p.add(16),
            *p.add(15), *p.add(14), *p.add(13), *p.add(12),
            *p.add(11), *p.add(10), *p.add( 9), *p.add( 8),
            *p.add( 7), *p.add( 6), *p.add( 5), *p.add( 4),
            *p.add( 3), *p.add( 2), *p.add( 1), *p,
        );
        
        // Calculates LUT range masks
        let mx6 = _mm256_cmpgt_epi8(slice, x5f);
        let mx4 = _mm256_andnot_si256(_mm256_cmpgt_epi8(slice, x4f), _mm256_cmpgt_epi8(slice, x3f));
        let mx3 = {
            let temp = _mm256_cmpgt_epi8(slice, x3f); // x < 0x40 == !(x > 0x39)
            _mm256_andnot_si256(temp, on)
        };

        // LUT indexes
        let ix3 = _mm256_sub_epi8(slice, x30);
        let ix4 = _mm256_sub_epi8(slice, x40);
        let ix6 = _mm256_sub_epi8(slice, x60);
        
        // LUT sample
        let vx3 = _mm256_shuffle_epi8(lutx3, ix3);
        let vx4 = _mm256_shuffle_epi8(lutx4and6, ix4);
        let vx6 = _mm256_shuffle_epi8(lutx4and6, ix6);
        
        // Aggregate results
        let dec = _mm256_or_si256(
            _mm256_or_si256(
                _mm256_and_si256(vx3, mx3),
                _mm256_and_si256(vx4, mx4)
            ),
            _mm256_and_si256(vx6, mx6)
        );
        
        // NOTE: To make the error handling possible I inverted all
        // operations and constants of the algorithm, this way when
        // `_mm_shuffle_epi8` recives an out of bounds index it will
        // return 0 which is not ok
        let ok = _mm256_movemask_epi8(dec) as u32;
        if ok != 0xffffffff {
            // TODO: Error index
            Err(InvalidCharAt(0))?
        }
        
        let dec = {
            // Pick even bytes containing most significant nibbles
            let temp = _mm256_andnot_si256(dec, m);
            // Peform a 12 bit shift
            let temp = _mm256_sll_epi64(temp, tmpsll);
            _mm256_andnot_si256(temp, dec)
        };
        
        // Takes only odd bytes
        let dec = _mm256_shuffle_epi8(dec, idec);
        
        // Final result, must be fliped
        *b = !_mm256_extract_epi64(dec, 1);
        *b.add(1) = !_mm256_extract_epi64(dec, 3);

        p = p.add(32);
        b = b.add(2);
    }

    // TODO: Should I add the SSE impl to process the remaining bytes ?

    // Handle the remaining of bytes
    let mut b = b as *mut u8;
    while p < p_end {
        let msn = *HEX_NIBBLE_DECODE.get_unchecked(*p as usize);
        if msn > 0xf { Err(InvalidCharAt(0))? }
        p = p.add(1);

        let lsn = *HEX_NIBBLE_DECODE.get_unchecked(*p as usize);
        if lsn > 0xf { Err(InvalidCharAt(0))? }
        p = p.add(1);

        *b = (msn << 4) | lsn;
        b = b.add(1);
    }

    Ok(Vec::from_raw_parts(v, e, e))
}

///////////////////////////////////////////////////////////////////////////////

#[inline(always)]
pub unsafe fn encode(input: &[u8]) -> String {
    // Constants
    let lut = _mm256_set_epi64x(HEX_ENCODE_64LUT_1, HEX_ENCODE_64LUT_0,
        HEX_ENCODE_64LUT_1, HEX_ENCODE_64LUT_0);
    let umask = _mm256_set1_epi32(MN_MASK);
    let lmask = _mm256_set1_epi32(LN_MASK);
    let srl = _mm_set1_epi64x(4);
    
    let c = input.len();
    let mut p = input.as_ptr() as *const i8;
    let p_end = p.add(c);
    
    // Allocate chunks of 8 bytes with alignment of 8
    // * NOTE: each byte need two other bytes, hence shift left 2 bits
    let e = c << 1;
    let v = alloc(Layout::from_size_align(e, align_of::<i64>()).unwrap());
    let mut b = v as *mut i64;

    while p.offset(31) < p_end {
        // * NOTE: no measurable change when taking 2 u64 at the time instead of 16 u8
        // but this will required forcing the input to be 8 bytes alingned, witch is
        // very complex to do
        let slice = _mm256_set_epi8(
            *p.add(31), *p.add(30), *p.add(29), *p.add(28),
            *p.add(27), *p.add(26), *p.add(25), *p.add(24),
            *p.add(23), *p.add(22), *p.add(21), *p.add(20),
            *p.add(19), *p.add(18), *p.add(17), *p.add(16),
            *p.add(15), *p.add(14), *p.add(13), *p.add(12),
            *p.add(11), *p.add(10), *p.add( 9), *p.add( 8),
            *p.add( 7), *p.add( 6), *p.add( 5), *p.add( 4),
            *p.add( 3), *p.add( 2), *p.add( 1), *p,
        );

        let mnibble = {
            let temp = _mm256_and_si256(slice, umask); // avx2 only
            // shift left the most significant nibble
            _mm256_srl_epi64(temp, srl)
        };
        let lnibble = _mm256_and_si256(slice, lmask);
        
        let mhex = _mm256_shuffle_epi8(lut, mnibble);
        let lhex = _mm256_shuffle_epi8(lut, lnibble);
        
        let hex0 = _mm256_unpacklo_epi8(mhex, lhex);
        let hex1 = _mm256_unpackhi_epi8(mhex, lhex);
        
        *b = _mm256_extract_epi64(hex0, 0);
        *b.add(1) = _mm256_extract_epi64(hex0, 1);
        *b.add(2) = _mm256_extract_epi64(hex1, 0);
        *b.add(3) = _mm256_extract_epi64(hex1, 1);

        *b.add(4) = _mm256_extract_epi64(hex0, 2);
        *b.add(5) = _mm256_extract_epi64(hex0, 3);
        *b.add(6) = _mm256_extract_epi64(hex1, 2);
        *b.add(7) = _mm256_extract_epi64(hex1, 3);

        p = p.add(32);
        b = b.add(8);
    }

    // TODO: Should I add the SSE impl to process the remaining bytes ?

    // loop through the remaining 31 or less bytes

    let mut b = b as *mut u8;
    while p < p_end {
        let j = ((*p as u8) as usize) << 1;
        *b = *HEX_ENCODE.get_unchecked(j);
        *b.add(1) = *HEX_ENCODE.get_unchecked(j | 1);
        
        p = p.add(1);
        b = b.add(2);
    }

    String::from_raw_parts(v,e, e)
}

#[inline(always)]
pub fn meet_requiriments() -> bool {
    if is_x86_feature_detected!("avx2") {
        return true;
    }

    return false;
}

crate::tests_hex!(super::encode, super::decode, super::meet_requiriments);
