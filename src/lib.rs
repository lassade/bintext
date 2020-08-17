//! 
//! Encoder and decoder of binary data for hex or base64 (wip).
//!
//! Provides SIMD implementations using SSE (SSE2 + SSSE3) and AVX2
//! instructions sets with as good as it gets fallback functions.
//!
//!
//! ### Differentials from similar crates
//!
//! - SSE for both `encode` and `decode` (for better performance and accessibility)
//! - As good as it gets default impl
//! - Decode aligned, decodes and aligns memory when enough padding is given

#![cfg_attr(feature = "asm", feature(asm))]

pub mod hex;
mod missing;