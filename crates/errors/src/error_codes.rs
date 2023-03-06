macro_rules! register_diagnostics {
    ( $( $error_code:ident ),* $(,)* ) => {
        /// Hold all error diagnostic by error code.
        ///
        /// It's an array of tuples where the first item is the error code, and the second
        /// item is the diagnostic.
        ///
        /// It's best to query this array by using [`crate::Error::explain`].
        #[cfg(feature = "diagnostics")]
        pub static DIAGNOSTICS: &[(&str, &str)] = &[
            $(
                (
                    stringify!($error_code),
                    concat!(
                        // Header
                        "\n",

                        // Title
                        "# Error `",
                        stringify!($error_code),
                        "`\n\n",

                        // Body
                        include_str!(concat!("./error_codes/", stringify!($error_code), ".md")),

                        // Footer
                        "",
                    ),
                )
            ),*
        ];

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

register_diagnostics!(E000, E001, E002, E003);
