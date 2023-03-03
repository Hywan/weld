macro_rules! register_diagnostics {
    ( $( $error_code:ident ),* $(,)* ) => {
        /// An array of `(error_code, diagnostic)`.
        pub static DIAGNOSTICS: &[(&str, &str)] = &[
            $(
                (
                    stringify!($error_code),
                    concat!(
                        // Header
                        "\n",

                        // Body
                        include_str!(concat!("./error_codes/", stringify!($error_code), ".md")),

                        // Footer
                        "",
                    ),
                )
            ),*
        ];

        #[cfg(doc)]
        /// This type exists only for documentation purposes. It doesn't exist in the code otherwise.
        ///
        /// This type has 2 goals:
        ///
        /// 1. To provide an idiomatic Rust documentation for all error codes,
        /// 2. To be able to test error code diagnostics with `cargo test --doc`.
        pub enum Diagnostics {
            $(
                #[doc = include_str!(concat!("./error_codes/", stringify!($error_code), ".md"))]
                $error_code
            ),*
        }
    };
}

register_diagnostics!(E000, E001, E002);
