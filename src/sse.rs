#[cfg(target_arch = "x86")]
use std::arch::x86::*;
#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

use crate::{DecodeError, HEX_ENCODE};

// (L) least (M) more significant mibble masks
const MN_MASK: i32 = 0xF0F0F0F0u32 as i32;
const LN_MASK: i32 = 0x0F0F0F0F;


const I: u8 = 255;
// offseted by 1
const HEX_DECODE_64LUT_X30_1: i64 = i64::from_le_bytes([0x8, 0x9,   I,   I,   I,   I,   I,   I]); // [0-9]
const HEX_DECODE_64LUT_X30_0: i64 = i64::from_le_bytes([0x0, 0x1, 0x2, 0x3, 0x4, 0x5, 0x6, 0x7]);
const HEX_DECODE_64LUT_AZ: i64 = i64::from_le_bytes([  I, 0xa, 0xb, 0xc, 0xd, 0xe, 0xf,   I]); // [a-z] [A-Z]

const SAMPLE: &'static str = "aadbe337162c5d615138f425a2d94e4f1";
const MATCH: [u8; 16] = [
    0xaa, 0xdb, 0xe3, 0x37, 0x16, 0x2c, 0x5d, 0x61,
    0x51, 0x38, 0xf4, 0x25, 0xa2, 0xd9, 0x4e, 0x4f
];

#[inline(always)]
pub unsafe fn decode(_input: &str) -> Result<Vec<u8>, DecodeError> {
    // Only deals with valid inputs!
    let lutx3 = _mm_set_epi64x(HEX_DECODE_64LUT_X30_1, HEX_DECODE_64LUT_X30_0);
    let lutx4 = _mm_set_epi64x(-1, HEX_DECODE_64LUT_AZ);
    //let lutx4 = _mm_set_epi64x(-1, -1);
    let lutx6 = _mm_set_epi64x(-1, HEX_DECODE_64LUT_AZ);
    
    let p = SAMPLE.as_bytes().as_ptr() as *const i8;
    
    let slice = _mm_set_epi8(
        *p.add(15), *p.add(14), *p.add(13), *p.add(12),
        *p.add(11), *p.add(10), *p.add( 9), *p.add( 8),
        *p.add( 7), *p.add( 6), *p.add( 5), *p.add( 4),
        *p.add( 3), *p.add( 2), *p.add( 1), *p,
    );
    
    print!("slice: {:016x}", _mm_extract_epi64(slice, 1));
    println!("{:016x}", _mm_extract_epi64(slice, 0));
    

    let cx3 = _mm_set1_epi8(0x2fu8 as i8);
    let cx4 = _mm_set1_epi8(0x3fu8 as i8);
    let cx6 = _mm_set1_epi8(0x5fu8 as i8);
    let mx6 = _mm_cmpgt_epi8(slice, cx6);
    let mx4 = _mm_andnot_si128(mx6, _mm_cmpgt_epi8(slice, cx4));
    let mx3 = _mm_andnot_si128(mx4, _mm_andnot_si128(mx6, _mm_cmpgt_epi8(slice, cx3)));
    
    print!("mx3:   {:016x}", _mm_extract_epi64(mx3, 1));
    println!("{:016x}", _mm_extract_epi64(mx3, 0));
    
    print!("mx4:   {:016x}", _mm_extract_epi64(mx4, 1));
    println!("{:016x}", _mm_extract_epi64(mx4, 0));
    
    print!("mx6:   {:016x}", _mm_extract_epi64(mx6, 1));
    println!("{:016x}", _mm_extract_epi64(mx6, 0));
    
    let x3 = _mm_set1_epi8(0x30u8 as i8);
    let x4 = _mm_set1_epi8(0x40u8 as i8);
    let x6 = _mm_set1_epi8(0x60u8 as i8);
    let ix3 = _mm_subs_epu8(slice, x3);
    let ix4 = _mm_subs_epu8(slice, x4);
    let ix6 = _mm_subs_epu8(slice, x6);
    
    print!("ix3:   {:016x}", _mm_extract_epi64(ix3, 1));
    println!("{:016x}", _mm_extract_epi64(ix3, 0));
    
    print!("ix4:   {:016x}", _mm_extract_epi64(ix4, 1));
    println!("{:016x}", _mm_extract_epi64(ix4, 0));
    
    print!("ix6:   {:016x}", _mm_extract_epi64(ix6, 1));
    println!("{:016x}", _mm_extract_epi64(ix6, 0));
    
    let vx3 = _mm_shuffle_epi8(lutx3, ix3);
    let vx4 = _mm_shuffle_epi8(lutx4, ix4);
    let vx6 = _mm_shuffle_epi8(lutx6, ix6);
    
    print!("vx3:   {:016x}", _mm_extract_epi64(vx3, 1));
    println!("{:016x}", _mm_extract_epi64(vx3, 0));
    
    print!("vx4:   {:016x}", _mm_extract_epi64(vx4, 1));
    println!("{:016x}", _mm_extract_epi64(vx4, 0));
    
    print!("vx6:   {:016x}", _mm_extract_epi64(vx6, 1));
    println!("{:016x}", _mm_extract_epi64(vx6, 0));
    
    let dec = _mm_or_si128(
        _mm_or_si128(
            _mm_and_si128(vx3, mx3),
            _mm_and_si128(vx4, mx4)
        ),
        _mm_and_si128(vx6, mx6)
    );
    
    let dec = {
        let m = _mm_set1_epi16(0x00FFu16 as i16);
        let temp = _mm_and_si128(dec, m);
        let temp = _mm_set_epi64x(
            ((_mm_extract_epi64(temp, 1) as u64) << 12) as i64,
            ((_mm_extract_epi64(temp, 0) as u64) << 12) as i64,
        );
        
        print!("temp:  {:016x}", _mm_extract_epi64(temp, 1));
        println!("{:016x}", _mm_extract_epi64(temp, 0));
        _mm_or_si128(temp, dec)
    };
    
    print!("dec:   {:016x}", _mm_extract_epi64(dec, 1));
    println!("{:016x}", _mm_extract_epi64(dec, 0));
    
    let idec = _mm_set_epi64x(0x0f_0d_0b_09_07_05_03_01u64 as i64, -1);
    let dec = _mm_shuffle_epi8(dec, idec);
    
    let b = _mm_extract_epi64(dec, 1);
    println!("res:   {:016x}", b);
    
    assert_eq!(&b.to_le_bytes()[..], &MATCH[0..8]);

    todo!()
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
    
    while p < p_end {
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

    let e = e << 3; // each i64 have 8 bytes
    String::from_raw_parts(v.as_mut_ptr() as *mut u8, e, e)
}