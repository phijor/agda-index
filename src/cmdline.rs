use std::path::PathBuf;

use argh::FromArgs;

#[derive(Debug, FromArgs)]
/// Index top-level definitions found in Agda modules rendered to HTML
pub struct CommandLine {
    #[argh(positional)]
    /// paths to HTML files containing a rendered Agda modules
    pub module_paths: Vec<PathBuf>,
}

/// Parse arguments given on the command line.
pub fn parse() -> CommandLine {
    argh::from_env()
}
