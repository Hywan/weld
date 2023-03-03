#[cfg(feature = "fancy-errors")]
use miette::{set_hook, MietteHandlerOpts};
use miette::{Diagnostic, InstallError, Result};
use thiserror::Error;
use weld_errors::Error as WeldError;

#[derive(Error, Diagnostic, Debug)]
pub(crate) enum Error {
    #[error("The argument `{0}` contains invalid Unicode data.")]
    InvalidArgumentEncoding(String),

    #[error("The program name is missing from the command-line.")]
    ProgramNameIsMissing,

    #[error("I was not able to read the command-line properly:\n{0}")]
    #[diagnostic(code(E001), help("See the command-line usage with `weld --help`."))]
    CommandLine(String),
}

impl Error {
    pub(crate) fn install_and_configure() -> Result<(), InstallError> {
        #[cfg(feature = "fancy-errors")]
        set_hook(Box::new(|_| {
            Box::new(
                MietteHandlerOpts::new()
                    .with_cause_chain()
                    .footer(
                        "For more information about an error, try \
                        `weld --explain <error>` where `<error>` \
                        has the `E[0-9]{{3}} pattern."
                            .to_string(),
                    )
                    .width(85)
                    .terminal_links(false)
                    .build(),
            )
        }))?;

        Ok(())
    }

    pub(crate) fn explain(error_code: &str) -> Result<&'static str, WeldError> {
        WeldError::explain(error_code)
    }
}
