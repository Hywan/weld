use std::path::PathBuf;

use argh::FromArgs;
use weld_linker::{target::Triple, Configuration, Error};

fn default_output_file() -> PathBuf {
    PathBuf::from("a.out")
}

#[derive(Debug, FromArgs)]
/// The `weld` command is an experimental linker, i.e. just like `ld` for
/// instance, it combines several object files and libraries, resolves
/// references, and produces an output file.
struct Weld {
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

fn main() -> Result<(), Error> {
    let args: Weld = argh::from_env();

    let linker_configuration = Configuration::new(args.target, args.input_files, args.output_file);

    let linker = linker_configuration.linker();

    // Take a deep breath, and here we are!
    linker.link()
}
