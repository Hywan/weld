//! `weld-bin` is the executable of `weld`.
//!
//! This crate contains all the implementation to make `weld` an executable that
//! can be used by happy users.

mod error;

use std::{
    env,
    ffi::OsString,
    path::{Path, PathBuf},
    process,
};

use argh::FromArgs;
use error::Error;
use weld_errors::Result;
use weld_linker::{target::Triple, Configuration};

fn default_output_file() -> PathBuf {
    PathBuf::from("a.out")
}

/// The `weld` command is an experimental linker: it combines several object
/// files and libraries, resolves symbols, and produces an output file.
#[derive(Debug, FromArgs)]
struct Weld {
    /// explain a particular error based on its code (of kind `E...`).
    #[argh(option)]
    explain: Option<String>,

    /// target triple.
    #[argh(option, short = 't', default = "Triple::host()")]
    target: Triple,

    /// input files.
    #[argh(positional)]
    input_files: Vec<PathBuf>,

    /// specify the name and location of the output file. If not specified,
    /// `a.out` is used.
    #[argh(option, short = 'o', default = "default_output_file()")]
    output_file: PathBuf,
}

impl Weld {
    /// Creates a new `Self` type based on [`std::env::args_os`].
    fn new() -> Result<Self, Error> {
        // Collect all arguments.
        let arguments =
            env::args_os().map(OsString::into_string).collect::<Result<Vec<_>, _>>().map_err(
                |argument| Error::InvalidArgumentEncoding(argument.to_string_lossy().to_string()),
            )?;

        // Check whether `argv` is present.
        if arguments.is_empty() {
            return Err(Error::ProgramNameIsMissing);
        }

        // Extract the base command from a path.
        let command = Path::new(&arguments[0])
            .file_name()
            .and_then(|file_name| file_name.to_str())
            .unwrap_or(&arguments[0]);

        // Extract all arguments.
        let arguments =
            arguments.iter().skip(1).map(|argument| argument.as_str()).collect::<Vec<_>>();

        // Parse and build `Self`.
        match Weld::from_args(&[command], &arguments) {
            Ok(weld) => Ok(weld),
            Err(early_exit) => match early_exit.status {
                // The command was parsed successfully and the early exit is due to a flag like
                // `--help` causing early exit with output.
                Ok(()) => {
                    println!("{}", early_exit.output);

                    process::exit(0);
                }

                // The arguments were not successfully parsed.
                Err(()) => Err(Error::CommandLine(early_exit.output.trim().to_string())),
            },
        }
    }
}

fn main() -> Result<()> {
    // Install the error report.
    Error::install_and_configure()?;

    // Build the command-line arguments.
    let weld = Weld::new()?;

    // Handle the `--explain` option.
    if let Some(error_code) = weld.explain {
        println!("{}", Error::explain(&error_code)?);

        return Ok(());
    }

    // Configure and create the linker.
    let linker = Configuration::new(weld.target, weld.input_files, weld.output_file).linker();

    // Take a deep breath, and here we are!
    linker.link()?;

    Ok(())
}
