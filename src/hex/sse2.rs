#![allow(dead_code)]

#[cfg(target_arch = "x86")]
use std::arch::x86::*;
#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

use std::alloc::{alloc, Layout};
use std::mem::align_of;
use super::*;

///////////////////////////////////////////////////////////////////////////////


#[inline(always)]
pub unsafe fn decode(input: &str) -> Result<Vec<u8>, DecodeError> {
    use DecodeError::*;

    // Input check
    let c = input.len();
    if c & 1 != 0 { Err(OddLength)? }
    
    let mut v = super::alloc(c >> 1);
    decode_noalloc(input.as_bytes(), v.as_mut_slice())?;

    Ok(v)
}


///////////////////////////////////////////////////////////////////////////////


#[inline(always)]
pub unsafe fn decode_aligned(input: &mut [u8], offset: usize, align: usize) -> Result<&mut [u8], DecodeError> {
    let _ = input;
    let _ = offset;
    let _ = align;
    todo!()
}

pub unsafe fn decode_noalloc(input: &[u8], output: &mut [u8]) -> Result<(), DecodeError> {
    use DecodeError::*;

    // Input check
    let c = input.len();
    //if c & 1 != 0 { Err(OddLength)? }

    // Constants
    let lutx3 = _mm_set_epi64x(HEX_DECODE_64LUT_X30_1, HEX_DECODE_64LUT_X30_0);
    let lutx4and6 = _mm_set_epi64x(0, HEX_DECODE_64LUT_AZ);
    
    let x30 = _mm_set1_epi8(0x30u8 as i8);
    let x3f = _mm_set1_epi8(0x3fu8 as i8);
    let x40 = _mm_set1_epi8(0x40u8 as i8);
    let x50 = _mm_set1_epi8(0x50u8 as i8);
    let x5f = _mm_set1_epi8(0x5fu8 as i8);
    let x60 = _mm_set1_epi8(0x60u8 as i8);
    
    let m = _mm_set1_epi16(0x00FFu16 as i16);
    let idec = _mm_set_epi64x(-1, 0x0f_0d_0b_09_07_05_03_01u64 as i64);
    let tmpsll = _mm_set1_epi64x(12);
    let filled = _mm_set1_epi64x(-1);
    
    // Input pointers
    let mut p = input.as_ptr() as *const i8;
    let p_end = p.add(c);
    
    let mut b = output.as_mut_ptr() as *mut i8;

    // Main loop loop
    while p.offset(15) < p_end {
        // TODO: how about _mm_lddqu_si128?
        let slice = _mm_loadu_si128(p as *const __m128i);
        
        // Calculates LUT range masks
        let mx6 = _mm_cmpgt_epi8(slice, x5f);
        let mx4 = _mm_and_si128(_mm_cmpgt_epi8(slice, x3f), _mm_cmplt_epi8(slice, x50));
        let mx3 = _mm_cmplt_epi8(slice, x40);
        
        // LUT indexes
        let ix3 = _mm_sub_epi8(slice, x30);
        let ix4 = _mm_sub_epi8(slice, x40);
        let ix6 = _mm_sub_epi8(slice, x60);
        
        // LUT sample
        let vx3 = _mm_shuffle_epi8(lutx3, ix3);
        let vx4 = _mm_shuffle_epi8(lutx4and6, ix4);
        let vx6 = _mm_shuffle_epi8(lutx4and6, ix6);
        
        // Aggregate results
        let dec = _mm_or_si128(
            _mm_or_si128(
                _mm_and_si128(vx3, mx3),
                _mm_and_si128(vx4, mx4)
            ),
            _mm_and_si128(vx6, mx6)
        );
        
        // NOTE: To make the error handling possible I inverted all
        // operations and constants of the algorithm, this way when
        // `_mm_shuffle_epi8` recives an out of bounds index it will
        // return 0 which is not ok
        let ok = _mm_movemask_epi8(dec) as u32;
        if ok != 0xffff {
            // TODO: Error index
            Err(InvalidCharAt(0))?
        }
        
        let dec = {
            // Pick even bytes containing most significant nibbles
            let temp = _mm_andnot_si128(dec, m);
            // Peform a 12 bit shift
            let temp = _mm_sll_epi64(temp, tmpsll);
            _mm_andnot_si128(temp, dec)
        };
        
        // Takes only odd bytes
        // Final result, must be fliped
        let dec = _mm_andnot_si128(_mm_shuffle_epi8(dec, idec), filled);

        // Saves the final result
        crate::missing::_mm_storeu_si64(b, dec);
        b = b.add(8);

        p = p.add(16);
    }

    // Handle the remaining of bytes
    while p < p_end {
        let msn = *HEX_NIBBLE_DECODE.get_unchecked(*p as usize);
        if msn > 0xf { Err(InvalidCharAt(0))? }
        p = p.add(1);

        let lsn = *HEX_NIBBLE_DECODE.get_unchecked(*p as usize);
        if lsn > 0xf { Err(InvalidCharAt(0))? }
        p = p.add(1);

        *b = ((msn << 4) | lsn) as i8;
        b = b.add(1);
    }

    Ok(())
}

///////////////////////////////////////////////////////////////////////////////


#[inline(always)]
pub unsafe fn encode(input: &[u8]) -> String {
    // Constants
    let lut = _mm_set_epi64x(HEX_ENCODE_64LUT_1, HEX_ENCODE_64LUT_0);
    let umask = _mm_set1_epi32(MN_MASK);
    let lmask = _mm_set1_epi32(LN_MASK);
    let srl = _mm_set1_epi64x(4);
    
    let c = input.len();
    let mut p = input.as_ptr() as *const i8;
    let p_end = p.add(c);
    
    // Allocate chunks of 8 bytes with alignment of 8
    // * NOTE: each byte need two other bytes, hence shift left 2 bits
    let e = c << 1;
    let v = alloc(Layout::from_size_align(e, align_of::<i64>()).unwrap());
    let mut b = v as *mut i64;

    while p.offset(15) < p_end {
        // TODO: how about _mm_lddqu_si128?
        // * NOTE: no measurable change when taking 2 u64 at the time instead of 16 u8
        // but this will required forcing the input to be 8 bytes aligned, witch is
        // very complex to do
        let slice = _mm_loadu_si128(p as *const __m128i);

        let mnibble = {
            let temp = _mm_and_si128(slice, umask);
            // shift left the most significant nibble
            _mm_srl_epi64(temp, srl)
        };
        let lnibble = _mm_and_si128(slice, lmask);
        
        let mhex = _mm_shuffle_epi8(lut, mnibble);
        let lhex = _mm_shuffle_epi8(lut, lnibble);
        
        let hex0 = _mm_unpacklo_epi8(mhex, lhex);
        let hex1 = _mm_unpackhi_epi8(mhex, lhex);
        
        // ! FIXME SSE4
        *b = _mm_extract_epi64(hex0, 0);
        *b.add(1) = _mm_extract_epi64(hex0, 1);
        *b.add(2) = _mm_extract_epi64(hex1, 0);
        *b.add(3) = _mm_extract_epi64(hex1, 1);

        p = p.add(16);
        b = b.add(4);
    }

    // loop through the remaining 15 or less bytes

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
pub fn meet_requirements() -> bool {
    if is_x86_feature_detected!("sse2") && is_x86_feature_detected!("ssse3")  {
        return true;
    }

    return false;
}

crate::tests_hex!(super::encode, super::decode, super::meet_requirements);
