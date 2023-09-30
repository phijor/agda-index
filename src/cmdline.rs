use std::{path::PathBuf, str::FromStr};

use argh::FromArgs;

#[derive(Debug, FromArgs)]
/// Index top-level definitions found in Agda modules rendered to HTML
pub struct CommandLine {
    #[argh(option, default = "OutputFormat::Plain")]
    /// textual format of the finished index.
    /// Either
    /// "plain" (space-separated plaintext, default),
    /// "json" (JSON dictionary {{<source file>: <module items>}})
    pub output_format: OutputFormat,

    #[argh(positional)]
    /// paths to directory containing HTML files of rendered Agda modules
    pub html_dir: PathBuf,
}

#[derive(Debug, Clone, Copy)]
pub enum OutputFormat {
    Plain,
    Json,
}

impl FromStr for OutputFormat {
    type Err = &'static str;

    fn from_str(fmt: &str) -> Result<Self, Self::Err> {
        match fmt {
            "plain" => Ok(Self::Plain),
            "json" => Ok(Self::Json),
            _ => Err("expected one of 'plain' or 'json'"),
        }
    }
}

/// Parse arguments given on the command line.
pub fn parse() -> CommandLine {
    argh::from_env()
}
