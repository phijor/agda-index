use std::{path::PathBuf, str::FromStr};

use argh::FromArgs;

#[derive(Debug, FromArgs)]
/// Index top-level definitions found in Agda modules rendered to HTML
pub struct CommandLine {
    #[argh(option, default = "OutputFormat::Plain")]
    /// textual format of the finished index.
    /// Either
    /// "plain" (space-separated plaintext, default),
    /// "json" (JSON dictionary {{<source file>: <module items>}}), or
    /// "docset" (a Dash Docset)
    pub output_format: OutputFormat,

    #[argh(option, default = r#"("agda".into())"#)]
    /// name of the Agda library (field `name` in .agda-lib)
    pub library_name: String,

    #[argh(option, default = r#"("index.html".into())"#)]
    /// path to the main page, relative to <html_dir> (default: index.html)
    pub main_page: PathBuf,

    #[argh(positional)]
    /// paths to directory containing HTML files of rendered Agda modules
    pub html_dir: PathBuf,
}

#[derive(Debug, Clone, Copy)]
pub enum OutputFormat {
    Plain,
    Json,
    Docset,
}

impl FromStr for OutputFormat {
    type Err = &'static str;

    fn from_str(fmt: &str) -> Result<Self, Self::Err> {
        match fmt {
            "plain" => Ok(Self::Plain),
            "json" => Ok(Self::Json),
            "docset" => Ok(Self::Docset),
            _ => Err("expected one of 'plain', 'json' or 'docset'"),
        }
    }
}

/// Parse arguments given on the command line.
pub fn parse() -> CommandLine {
    argh::from_env()
}
