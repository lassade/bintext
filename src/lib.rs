//! Binary text encoding and decoding with support for SIMD (AVX2 and
//! SSSE3) with good fallback performance.
//!
//! The main idea of this crate is to have a zero copy binary deserialization
//! for text formats.
//!
//! ### How it works
//!
//! Alignment can't be guaranteed in a text format, so no matter what the data will need
//! to be re-aligned while decoding, if the required alignment is `N` maximum amount
//! of offset need to move the bytes is less than `N` thus by providing an start padding
//! in the binary encoded text of `N - 1` it's possible to align the data up to `N`.
//!
//! **Quick note** this crate will only accept padding equal or grater than `N`, because
//! it's a bit cheap to do this way.
//!
//! ```rust
//! // Padding of 8 (suppose it was read form a file)
//! let mut hex = "--------a1f7d5e8d14f0f76".to_string();
//!
//! unsafe {
//!     // Decode with padding of 8 and alignment of 8
//!     let slice = bintext::hex::decode_aligned(&mut hex, 8, 8).unwrap();
//!     // Data is aligned so you can safely do this:
//!     let slice: &[u64] = std::slice::from_raw_parts(slice.as_mut_ptr() as *mut _, 2);
//! }
//! ```

pub mod hex;
