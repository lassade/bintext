
#[doc(hidden)]
#[macro_export]
macro_rules! tests_hex {
    ($encode:path, $decode:path, $feat:path) => {
        #[cfg(test)]
        mod tests {
            const SAMPLES: [(&'static [u8], &'static str); 6] = [
                    (b"\xAd\x87\x7F", "ad877f"), // 3 bytes
                    (b"\x34\xcD\x6f\x62\xAf\xa9\x1a\x82\xC7\x24", "34cd6f62afa91a82c724"), // 10 bytes
                    (b"\x0a\x86\x16\x81\x45\x16\x51\xb7\x97\x4e\x81\x7f\xc7\xe8\x9e\xee\xbe\x61\x45\xe7",
                      "0a861681451651b7974e817fc7e89eeebe6145e7"), // 20 bytes
                    (b"\x0a\x86\x16\x81\x45\x16\x51\xb7\x97\x4e\x81\x7f\xc7\xe8\x9e\xee\
                       \xbe\x61\x45\xe7",
                      "0a861681451651b7974e817fc7e89eee\
                       be6145e7"),
                    (b"\x61\xa9\xa7\x25\x9e\x8b\x08\x82\x4c\xc7\xd5\xa7\x4f\xd6\x13\x8f\
                       \x97\x03\x79\x5a\xa5\x09\xf6\xa2\xf9\xa9\x1c\x0e\xfb\xad\x23\x72\
                       \x84\xe4\x0c\x8e\x6d\xb3\xb3\x4e",
                      "61a9a7259e8b08824cc7d5a74fd6138f\
                       9703795aa509f6a2f9a91c0efbad2372\
                       84e40c8e6db3b34e"), // 40
                    (b"\x82\xf1\x80\xca\x2c\xc7\xfe\x31\xef\x44\xe4\xae\xce\x70\x63\x42\
                       \xfa\x95\xf6\x4e\x2c\xbf\xac\x65\xa2\x51\xab\xf3\x21\x17\x4c\x28\
                       \x94\x9b\x80\x8c\x7d\xf1\x2c\xd0\xda\xfb\xb7\x5c\xc4\x13\xba\xe7\
                       \xf2\xe1\xa6\x74\x59\x76\x0d\x34\x2c\x3b\x49\xc2\x05\xb8\x7d\xaa\
                       \xdd\x39\x6f\x64\xee\x14",
                      "82f180ca2cc7fe31ef44e4aece706342\
                       fa95f64e2cbfac65a251abf321174c28\
                       949b808c7df12cd0dafbb75cc413bae7\
                       f2e1a67459760d342c3b49c205b87daa\
                       dd396f64ee14") // 70
                ];

                
            #[test]
            #[allow(unused_unsafe)]
            fn encoding() {
                if !$feat() {
                    panic!("doesn't have the required instruction set");
                }

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

                for (expected, input) in SAMPLES.iter() {
                    let r = unsafe { $decode(&str::to_uppercase(input)) };
                    assert_eq!(r.unwrap(), *expected);
                }
            }
        }
    }
}
