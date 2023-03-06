//! `weld_errors` provide basic features to declare human-understandable errors,
//! along with diagnostics.
//!
//! First off, `weld_errors` provides the [`error!`] macro that helps to declare
//! types that implement [`std::error::Error`], and also derive
//! [`thiserror::Error`] and [`miette::Diagnostic`]. The macro helps to define
//! error code, message, formatted message, and help. It automatically generates
//! documentation, with intra-links to the [`Diagnostics`] type. As an example,
//! see the [`Error`] type that is built with this macro!
//!
//! The second feature provided by `weld_errors` is [`Diagnostics`]. When an
//! error has a code, e.g. `E003`, it can be used to further explain an error
//! with `weld --explain E003`, à la `rustc`. That's almost the same mechanism.
//! But the diagnostics are also part of the documentation itself, check for
//! example [`Diagnostics::E003`]. There is 2 ways to get the detailed
//! diagnostics based on error code.

#![deny(unused)]
#![deny(warnings)]
#![deny(missing_docs)]
#![deny(clippy::all)]
#![deny(rustdoc::broken_intra_doc_links)]
#![deny(rustdoc::private_intra_doc_links)]
#![deny(rustdoc::missing_crate_level_docs)]
#![deny(rustdoc::invalid_codeblock_attributes)]
#![deny(rustdoc::invalid_rust_codeblocks)]

mod error_codes;

pub use error_codes::Diagnostics;
#[cfg(feature = "diagnostics")]
pub use error_codes::DIAGNOSTICS;
pub use miette::Result;

#[doc(hidden)]
#[macro_export]
macro_rules! as_item {
    ($item:item) => {
        $item
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! link_error_code {
    ($error_code:ident) => {
        concat!(
            "[`",
            stringify!($error_code),
            "`](weld_errors::Diagnostics::",
            stringify!($error_code),
            ")."
        )
    };
}

/// Use this macro to declare a type that acts like a human-understable error,
/// with diagnostics.
///
/// # Example
///
/// An example is better than a long text:
///
/// ```rust
/// use weld_errors::error;
///
/// error! {
///     pub enum Error {
///         #[code = E000]
///         #[message = "The given error code is invalid."]
///         #[formatted_message("`{0}` is not a valid error code.")]
///         #[help = "Did you mistype the error code?"]
///         InvalidCode(String),
///     }
/// }
///
/// # fn main() {
/// let error = Error::InvalidCode("xyz".to_string());
///
/// // Get the error as a string.
/// assert_eq!(
///     error.to_string(),
///     "`xyz` is not a valid error code.".to_string()
/// );
///
/// // Get more information on the errors, via `miette`.
///
/// use miette::Diagnostic;
///
/// assert_eq!(
///     error.code().map(|code| code.to_string()),
///     Some("E000".to_string())
/// );
/// assert_eq!(
///     error.help().map(|help| help.to_string()),
///     Some("Did you mistype the error code?".to_string())
/// );
/// # }
/// ```
///
/// This create an `Error` enum type, with an `InvalidCode` tuple variant.
/// This newly created `Error` enum type implements [`thiserror::Error`] and
/// [`miette::Diagnostic`].
///
/// # Syntax
///
/// So far, the macro only supports an `enum` declaration, no `struct` yet.
/// `enum` supports variant, or tuple variant only.
///
/// Each variant can have the following attributes:
///
/// * `#[cfg(…)]` (optional),
/// * `#[code = E...]` to define the error code (optional),
/// * `#[message = "…"]` to define a literal string message; it will be used as
///   documentation.
/// * `#[formatted_message("format {0} {}", .0.accessor)]` to define a “dynamic”
///   string message; it will be used for the [`std::fmt::Display`]
///   implementation, and follows the same rules as the `#[error(…)]` attribute
///   of [`thiserror`] (optional).
/// * `#[help = "…"]` to define a help, a hint, a tip, to drive the user to a
///   solution; note that this is mandatory.
///
/// Alternatively, it is possible to annotate a variant with `#[cfg(…)]`
/// (optional) and `#[transparent]` only, which makes the variant “transparent”
/// and forwards everything to the first tuple item of the variant. Note that
/// tuple items can use the same [`thiserror`] attributes, like `#[from]`.
///
/// ```rust
/// use weld_errors::error;
///
/// error! {
///     pub enum Error {
///         #[transparent]
///         Other(#[from] Box<dyn std::error::Error>),
///     }
/// }
/// ```
#[macro_export]
macro_rules! error {
    // Error declaration with a static literal message.
    (
        @variant
        [ $( $declaration:tt )* ]
        [ $( $accumulator:tt )* ]
        $( #[cfg( $cfg:meta )] )*
        $( #[code = $error_code:ident] )?
        #[message = $error_message:expr]
        #[help = $error_help:literal]
        $( $tail:tt )*
    ) => {
        error! {
            @variant
            [ $( $declaration )* ]
            [
                $( $accumulator )*

                $(
                    #[doc = concat!("Error code: ", $crate::link_error_code!($error_code))]
                    #[doc = "\n"]
                )?
                #[doc = $error_message]
                #[error($error_message)]
                #[diagnostic(
                    $( code($error_code), )?
                    help($error_help),
                )]
                $( #[cfg( $cfg )] )*
            ]
            $( $tail )*
        }
    };

    // Error declaration with a dynamic message.
    (
        @variant
        [ $( $declaration:tt )* ]
        [ $( $accumulator:tt )* ]
        $( #[cfg( $cfg:meta )] )*
        $( #[code = $error_code:ident] )?
        #[message = $error_message:expr]
        #[formatted_message( $error_message_format:literal $( , . $error_message_arguments:expr )* $( , )* )]
        #[help = $error_help:literal]
        $( $tail:tt )*
    ) => {
        error! {
            @variant
            [ $( $declaration )* ]
            [
                $( $accumulator )*

                $(
                    #[doc = concat!("Error code: ", $crate::link_error_code!($error_code))]
                    #[doc = "\n"]
                )?
                #[doc = $error_message]
                #[error( $error_message_format $( , . $error_message_arguments ),* )]
                #[diagnostic(
                    $( code($error_code), )?
                    help($error_help),
                )]
                $( #[cfg( $cfg )] )*
            ]
            $( $tail )*
        }
    };

    // Transparent error.
    (
        @variant
        [ $( $declaration:tt )* ]
        [ $( $accumulator:tt )* ]
        $( #[cfg( $cfg:meta )] )*
        #[transparent]
        $( $tail:tt )*
    ) => {
        error! {
            @variant
            [ $( $declaration )* ]
            [
                $( $accumulator )*

                #[doc = "Transparent error. Please see the inner fields."]
                #[error(transparent)]
                $( #[cfg( $cfg )] )*
            ]
            $( $tail )*
        }
    };

    // Unit variant.
    (
        @variant
        [ $( $declaration:tt )* ]
        [ $( $accumulator:tt )* ]
        $variant_name:ident ,
        $( $tail:tt )*
    ) => {
        error! {
            @variant
            [ $( $declaration )* ]
            [
                $( $accumulator )*
                $variant_name,
            ]
            $( $tail )*
        }
    };

    // Tuple variant.
    (
        @variant
        [ $( $declaration:tt )* ]
        [ $( $accumulator:tt )* ]
        $variant_name:ident (
            $(
                $( #[ $field_meta:meta ] )*
                $field_visibility:vis $field_type:ty
            ),*
            $( , )?
        ) ,
        $( $tail:tt )*
    ) => {
        error! {
            @variant
            [ $( $declaration )* ]
            [
                $( $accumulator )*
                $variant_name (
                    $(
                        $( #[ $field_meta ] )*
                        $field_visibility $field_type,
                    )*
                ) ,
            ]
            $( $tail )*
        }
    };


    // End point.
    (
        @variant
        [ $( $declaration:tt )* ]
        [ $( $accumulator:tt )* ]
    ) => {
        $crate::as_item! {
            $( $declaration )* {
                $( $accumulator )*
            }
        }
    };

    // Entry point.
    (
        $( #[doc = $documentation:expr ] )*
        $visibility:vis enum $error_name:ident {
            $( $variants:tt )*
        }
    ) => {
        error! {
            @variant
            [
                $( #[doc = $documentation ] )*
                #[derive(Debug, thiserror::Error, miette::Diagnostic)]
                $visibility enum $error_name
            ]
            []
            $( $variants )*
        }
    };
}

// The `error!` macro generates links to `weld_errors::Diagnostics::E...`. To
// avoid having a warning, since this `Error` type below is living inside
// `weld_errors` itself, a new alias is created from `crate` to `weld_errors`,
// and tadaa, no more warning.
#[cfg(doc)]
use crate as weld_errors;

error! {
    #[doc = "Error type for this crate."]
    #[doc = "\n"]
    #[doc = "The major interests of this type is its `Self::explain` method that can be"]
    #[doc = "used to fetch the diagnostic of a particular error code."]
    pub enum Error {
        #[code = E000]
        #[message = "The given error code is invalid."]
        #[formatted_message("`{0}` is not a valid error code.")]
        #[help = "Did you mistype the error code? The pattern is `E[0-9]{{3}}`, i.e. an `E` followed by 3 digits, such as `E000`."]
        InvalidCode(String),

    }
}

impl Error {
    /// Given a specific error code, this method returns the associated
    /// diagnostic, if the error exists.
    ///
    /// ```
    /// use weld_errors::Error;
    ///
    /// # fn main() {
    /// // Explain a valid error.
    /// assert!(Error::explain("E000").is_ok());
    ///
    /// // Explain an invalid error.
    /// assert!(Error::explain("oops").is_err());
    /// # }
    /// ```
    #[cfg(feature = "diagnostics")]
    pub fn explain(error_code: &str) -> Result<&'static str, Self> {
        DIAGNOSTICS
            .iter()
            .find_map(
                |(current_error_code, diagnostic)| {
                    if *current_error_code == error_code {
                        Some(*diagnostic)
                    } else {
                        None
                    }
                },
            )
            .ok_or(Self::InvalidCode(error_code.to_owned()))
    }
}
