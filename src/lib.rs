//! 
//! Encoder and decoder of binary data for hex or base64 (wip).
//!
//! Provides SIMD implementations using SSE (SSE2 + SSSE3) and AVX2
//! instructions sets with as good as it gets fallback functions.
//!
//!
//! ### Differentials from similar crates
//!
//! - SSE for both `encode` and `decode` (for better performace and
//! assecibility)
//! - As good as it get's default impl
//! 

pub mod hex;