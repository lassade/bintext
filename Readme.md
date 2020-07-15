Simd Hex
=========

Hex encoding and decoding with support for SIMD (avx2 and sse4) with good fallback peformance

### Why

* `hex` bad peformance, but smaller code size since doesn't realy on LUTs
* `base16`, doesn't support SIMD, but have faster decoding and overal good encoding
* `faster-hex` fastest encode (SSE4.1, AVX2), decode (AVX2) missing SSE4.1 impl hurt bad the peformance

### Comparing with base64 encoding

* `base64`, faster againts any hex impl that doesn't use SIMD
* `b64_ct`, the worst