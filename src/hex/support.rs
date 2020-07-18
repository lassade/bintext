
/// Considers SSE2 and SSSE3 feature set to be present in very computer
/// this library will ever run, consider not using it.
///
/// Uses this to skip run time checks
#[cfg(feature = "sse_ubiquitous")]
#[doc(hidden)]
#[macro_export]
macro_rules! is_sse_ubiquitous {
    () => { true };
}

/// Considers SSE2 and SSSE3 feature set to be present in very computer
/// this library will ever run, consider not using it.
#[cfg(not(feature = "sse_ubiquitous"))]
#[doc(hidden)]
#[macro_export]
macro_rules! is_sse_ubiquitous {
    () => { false };
}

#[doc(hidden)]
#[macro_export]
macro_rules! _mm256_dbg {
    ($v:ident) => {
        print!("{:8}: {:016x}", stringify!($v), _mm256_extract_epi64($v, 3));
        print!("{:016x}", _mm256_extract_epi64($v, 2));
        print!("{:016x}", _mm256_extract_epi64($v, 1));
        println!("{:016x}", _mm256_extract_epi64($v, 0));
    }
}

#[doc(hidden)]
#[macro_export]
macro_rules! _mm128_dbg {
    ($v:ident) => {
        print!("{:8}: {:016x}", stringify!($v), _mm_extract_epi64($v, 1));
        println!("{:016x}", _mm_extract_epi64($v, 0));
    }
}