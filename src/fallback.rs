use crate::{alloc, DecodeError, HEX_ENCODE, HEX_NIBBLE_DECODE};

#[inline(always)]
pub fn decode(input: &str) -> Result<Vec<u8>, DecodeError> {
    use DecodeError::*;

    let p = input.as_bytes();
    let l = input.len();
    if l & 1 != 0 { Err(OddLength)? }

    let mut i = 0;
    let mut v = alloc(l >> 1);
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

#[inline(always)]
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

crate::tests_hex!(super::encode, super::decode);