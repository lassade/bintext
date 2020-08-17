#[cfg(target_arch = "x86")]
use std::arch::x86::*;
#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

#[inline]
#[cfg(feature = "nightly")]
pub unsafe fn _mm_storeu_si64(mem_addr: *mut i8, a: __m128i) {
    asm!("movq {}, {}", in(reg) mem_addr, in(xmm_reg) a);
}

#[inline]
#[cfg(not(feature = "nightly"))]
pub unsafe fn _mm_storeu_si64(mem_addr: *mut i8, a: __m128i) {
    // Best effort should yield 2 instructions
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