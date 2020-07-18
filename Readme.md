Bin Text (WIP)
=========

Hex and base64 (wip) encoding and decoding with support for SIMD (AVX2 and
SSSE3) with good fallback peformance

### Why 

There is a lot of other crates that already does the job, but every single one
have some kind of performance downside:

* `hex` bad peformance, but smaller code size since doesn't rely on LUTs;
* `base16`, no SIMD, but have faster decoding and overal good encoding;
* `faster-hex` fastest encode using SSSE3 and AVX2, decode only AVX2 but is
missing SSE implementation, which has the best SIMD coverage of all
instructions sets.

This crate provides implementations that covers all their competitors
performance weaknesses. Just run `cargo bench`.

### Comparing with base64 encoding

* `base64`, faster againts any hex impl that doesn't use SIMD;
* `radix64`, better decode but bit worse encoder than base64, avx enabled;