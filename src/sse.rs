#[cfg(target_arch = "x86")]
use std::arch::x86::*;
#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

use crate::{DecodeError, HEX_ENCODE, HEX_NIBBLE_DECODE};

// (L) least (M) more significant mibble masks
const MN_MASK: i32 = 0xF0F0F0F0u32 as i32;
const LN_MASK: i32 = 0x0F0F0F0F;


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
    let lutx3 = _mm_set_epi64x(HEX_DECODE_64LUT_X30_1, HEX_DECODE_64LUT_X30_0);
    let lutx4 = _mm_set_epi64x(-1, HEX_DECODE_64LUT_AZ);
    let lutx6 = _mm_set_epi64x(-1, HEX_DECODE_64LUT_AZ);
    
    let x30 = _mm_set1_epi8(0x30u8 as i8);
    let x3f = _mm_set1_epi8(0x3fu8 as i8);
    let x40 = _mm_set1_epi8(0x40u8 as i8);
    let x50 = _mm_set1_epi8(0x50u8 as i8);
    let x5f = _mm_set1_epi8(0x5fu8 as i8);
    let x60 = _mm_set1_epi8(0x60u8 as i8);
    
    let m = _mm_set1_epi16(0xFF00u16 as i16);
    let slb = _mm_set1_epi64x(0xfff as i64);
    let idec = _mm_set_epi64x(0x0f_0d_0b_09_07_05_03_01u64 as i64, -1);
    
    // Input pointers
    let mut p = input.as_ptr() as *const i8;
    let p_end = p.add(c);
    
    // Allocate chunks of 8 bytes with alignment of 8
    let e = (c >> 1) >> 3;
    let mut v = std::mem::ManuallyDrop::new(Vec::<i64>::with_capacity(e));
    v.set_len(e);
    let mut b = v.as_mut_ptr();

    // Main loop loop
    while p.offset(15) < p_end {
        let slice = _mm_set_epi8(
            *p.add(15), *p.add(14), *p.add(13), *p.add(12),
            *p.add(11), *p.add(10), *p.add( 9), *p.add( 8),
            *p.add( 7), *p.add( 6), *p.add( 5), *p.add( 4),
            *p.add( 3), *p.add( 2), *p.add( 1), *p,
        );
        
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
        let vx4 = _mm_shuffle_epi8(lutx4, ix4);
        let vx6 = _mm_shuffle_epi8(lutx6, ix6);
        
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
            Err(InvalidCharAt(0))?
        }
        
        let dec = {
            // Pick even bytes containing most significant nibbles
            let temp = _mm_or_si128(dec, m);
            
            // Peform a 12 bit shift (sse doesn't have intrisicts for it)
            let temp = _mm_set_epi64x(
                ((_mm_extract_epi64(temp, 1) as u64) << 12) as i64,
                ((_mm_extract_epi64(temp, 0) as u64) << 12) as i64,
            );
            
            // NOTE: Set the lest significant 12 bit, cleared by prev left bit shift
            let temp = _mm_or_si128(temp, slb);
            _mm_and_si128(temp, dec)
        };
        
        // Takes only odd bytes
        let dec = _mm_shuffle_epi8(dec, idec);
        
        // Final result, must be fliped
        *b = !_mm_extract_epi64(dec, 1);

        p = p.add(16);
        b = b.add(1);
    }

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

    Ok(
        Vec::from_raw_parts(
            v.as_mut_ptr() as *mut u8, 
            c >> 1, // use only the necessary ammount of bytes
            e << 3 // each i64 have 8 bytes
    ))
}


const HEX_ENCODE_64LUT_1: i64 = i64::from_be_bytes(*b"fedcba98");
const HEX_ENCODE_64LUT_0: i64 = i64::from_be_bytes(*b"76543210");

#[inline(always)]
pub unsafe fn encode(input: &[u8]) -> String {
    // Constants
    let lut = _mm_set_epi64x(HEX_ENCODE_64LUT_1, HEX_ENCODE_64LUT_0);
    let umask = _mm_set1_epi32(MN_MASK);
    let lmask = _mm_set1_epi32(LN_MASK);
    
    let c = input.len();
    let mut p = input.as_ptr() as *const i8;
    let p_end = p.add(c);
    
    // Allocate chunks of 8 bytes with alignment of 8
    let e = c >> 2; // * NOTE: each byte need two other bytes, hence shift left 2 bits
    let mut v = std::mem::ManuallyDrop::new(Vec::<i64>::with_capacity(e));
    v.set_len(e);

    let mut b = v.as_mut_ptr();

    while p.offset(15) < p_end {
        // * NOTE: no measurable change when taking 2 u64 at the time instead of 16 u8
        // but this will required forcing the input to be 8 bytes alingned, witch is
        // very complex to do
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

    String::from_raw_parts(
        v.as_mut_ptr() as *mut u8,
        c << 1, // use only the necessary ammount of bytes
        e << 3 // each i64 have 8 bytes)
    )
}