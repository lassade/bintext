//! Rust doesn't have a `_mm_storeu_si64` where is provide 2 alternatives
//! on nightly + feature `asm` uses the `asm!` macro to output the right
//! assembly instruction, on stable a best effort function is provided,
//! it should yield 2 instructions.
//!
//! The throughput using `asm` is about 45% greater than not using it,
//! thus it will be enabled by default.
//!

#[cfg(target_arch = "x86")]
use std::arch::x86::*;
#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

#[inline]
#[cfg(feature = "asm")]
pub unsafe fn _mm_storeu_si64(mem_addr: *mut i8, a: __m128i) {
    asm!("movq [{}], {}", in(reg) mem_addr, in(xmm_reg) a);
}

#[inline]
#[cfg(not(feature = "asm"))]
pub unsafe fn _mm_storeu_si64(mem_addr: *mut i8, a: __m128i) {
    let v: [i8; 16] =  std::mem::transmute(a);
    *mem_addr.add(0) = *v.get_unchecked(0);
    *mem_addr.add(1) = *v.get_unchecked(1);
    *mem_addr.add(2) = *v.get_unchecked(2);
    *mem_addr.add(3) = *v.get_unchecked(3);
    *mem_addr.add(4) = *v.get_unchecked(4);
    *mem_addr.add(5) = *v.get_unchecked(5);
    *mem_addr.add(6) = *v.get_unchecked(6);
    *mem_addr.add(7) = *v.get_unchecked(7);
}


#[cfg(test)]
mod tests {
    #[cfg(target_arch = "x86")]
    use std::arch::x86::*;
    #[cfg(target_arch = "x86_64")]
    use std::arch::x86_64::*;

    #[test]
    fn intrinsic_mm_set_epi64x() {
        unsafe { 
            let mut vals = [0x0Fu8; 20];
            let a = _mm_set_epi64x(0, 0x07_06_05_04_03_02_01_00);

            let mut ofs = 0;
            let mut p = vals.as_mut_ptr() as *mut i8;

            // Make sure p is **not** aligned to 16-byte boundary
            if p.align_offset(16) == 0 {
                ofs = 1;
                p = p.offset(1);
            }

            super::_mm_storeu_si64(p, a);

            if ofs > 0 {
                assert_eq!(vals[ofs - 1], 0x0F);
            }
            for i in 0..8 {
                assert_eq!(vals[ofs + i], i as u8);
            }
            assert_eq!(vals[ofs + 8], 0x0F);
            assert_eq!(vals[19], 0x0F);
        }
    }
}