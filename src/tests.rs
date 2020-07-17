
#[doc(hidden)]
#[macro_export]
macro_rules! hex {
    ($encode:path, $decode:path) => {
        #[cfg(test)]
        mod tests {
            #[test]
            fn encoding() {
                todo!()
            }

            #[test]
            fn dencoding() {
                todo!()
            }
        }
    }
}
