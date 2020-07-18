
#[doc(hidden)]
#[macro_export]
macro_rules! tests_hex {
    ($encode:path, $decode:path) => {
        #[cfg(test)]
        mod tests {
            const SAMPLES: [(&'static [u8], &'static str); 3] = [
                    (b"\xAd\x87\x7F", "ad877f"),
                    (b"\x34\xcD\x6f\x62\xAf\xa9\x1a\x82\xC7\x24", "34cd6f62afa91a82c724"),
                    (b"\x0a\x86\x16\x81\x45\x16\x51\xb7\x97\x4e\x81\x7f\xc7\xe8\x9e\xee\xbe\x61\x45\xe7", "0a861681451651b7974e817fc7e89eeebe6145e7"),
                ];

                
            #[test]
            #[allow(unused_unsafe)]
            fn encoding() {
                for (input, expected) in SAMPLES.iter() {
                    let r = unsafe { $encode(input) };
                    assert_eq!(r, *expected);
                }
            }

            #[test]
            #[allow(unused_unsafe)]
            fn decoding() {
                for (expected, input) in SAMPLES.iter() {
                    let r = unsafe { $decode(input) };
                    assert_eq!(r.unwrap(), *expected);
                }
            }
        }
    }
}
