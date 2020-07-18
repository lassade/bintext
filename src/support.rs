
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