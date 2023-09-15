// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

mod cmdline;
mod module;
mod output;
mod pipeline;

use anyhow::Result;
use cmdline::{CommandLine, OutputFormat};

use crate::output::DocsetOutput;
use crate::output::JsonOutput;
use crate::output::{OutputWriter, PlainOutput};
use crate::pipeline::Pipeline;

#[derive(Debug, PartialEq, Eq)]
struct ItemInfo {
    id: String,
    module: String,
}

fn get_output_writer(cmdline: &CommandLine) -> Result<Box<dyn OutputWriter>> {
    let stdout = std::io::stdout();
    let stdout = stdout.lock();

    match cmdline.output_format {
        OutputFormat::Plain => {
            let plain = PlainOutput::new(stdout);
            Ok(Box::new(plain))
        }
        OutputFormat::Json => {
            let json = JsonOutput::new(stdout);
            Ok(Box::new(json))
        }
        OutputFormat::Docset => {
            let docset = DocsetOutput::new(
                cmdline.library_name.clone(),
                std::env::current_dir()?,
                cmdline.html_dir.clone(),
            );
            Ok(Box::new(docset))
        }
    }
}

fn main() -> Result<()> {
    let cmdline = cmdline::parse();

    let mut output = get_output_writer(&cmdline)?;

    let pipeline = {
        let pipeline = Pipeline::new();

        let html_dir = &cmdline.html_dir;

        let module_paths = std::fs::read_dir(html_dir)?;

        module_paths.into_iter().for_each(|entry| match entry {
            Ok(entry) => {
                let path = entry.path();
                if path.extension().is_some_and(|ext| ext == "html") {
                    pipeline.process_module(path);
                } else {
                    eprintln!("Skipping non-HTML file {}", path.display());
                }
            }

            Err(err) => eprintln!("Failed to read entry from {}: {err}", html_dir.display()),
        });

        pipeline
    };

    output.write_output(pipeline.consume())
}
