use crate::{alloc, HEX_ENCODE};

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