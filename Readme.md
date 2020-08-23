Bin Text
=========

*Encode and decodes binary encoded text into aligned binary blobs using SIMD*

![https://github.com/lassade/bintext/blob/main/.github/workflows/rust.yml](https://github.com/lassade/bintext/workflows/Build/badge.svg)

Binary text encoding and decoding with support for SIMD (AVX2 and
SSSE3) with good fallback performance.

The main idea of this crate is to have a zero copy binary deserialization
for text formats.

### How it works

Alignment can't be guaranteed in a text format, so no matter what the data will need
to be re-aligned while decoding, if the required alignment is `N` maximum amount
of offset need to move the bytes is less than `N` thus by providing an start padding
in the binary encoded text of `N - 1` it's possible to align the data up to `N`.

**Quick note** this crate will only accept padding equal or grater than `N`, because
it's a bit cheap to do this way.

```rust
// Padding of 8 (suppose it was read form a file)
let hex = "--------a1f7d5e8d14f0f76".to_string();

unsafe {
    // Decode with padding of 8 and alignment of 8
    let slice = bintext::hex::decode_aligned(&mut hex, 8, 8).unwrap();
    // Data is aligned so you can safely do this:
    let slice: &[u64] = std::slice::from_raw_parts(
        slice.as_ptr() as *const _,
        slice.len() / std::mem::size_of::<u64>()
    );
}
```

### TODO

- [ ] NEON instruction set
- [ ] Base64

### Other similar crates

There is a lot of other crates that already does the job, but every single one
have some kind of performance downside:

* `hex` bad performance, but smaller code size since doesn't rely on LUTs;
* `base16`, no SIMD, but have faster decoding and overall good encoding;
* `faster-hex` fastest encode using SSSE3 and AVX2, decode only has an AVX2.
SSE has the best SIMD coverage of all instructions sets, so it should be a must.

This crate provides implementations that covers all their competitors
performance weaknesses. Just run `cargo bench`.

All these crates doesn't seem to provide functions to decode aligned data,
so after decoding you will need and extra alignment step.
