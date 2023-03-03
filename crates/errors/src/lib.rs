mod error_codes;

#[cfg(doc)]
pub use error_codes::Diagnostics;
pub use error_codes::DIAGNOSTICS;
use miette::{Diagnostic, Result};
use thiserror::Error;

/// Error type for this crate.
///
/// The major interests of this type is its [`Self::explain`] method that can be
/// used to fetch the diagnostic of a particular error code.
#[derive(Debug, Diagnostic, Error)]
pub enum Error {
    #[error("{0} is not a valid error code.")]
    #[diagnostic(
        code(E000),
        help("Did you mistype the error code? The pattern is `E[0-9]{{3}}`, i.e. an `E` followed by 3 digits, such as `E000`.")
    )]
    InvalidCode(String),
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
