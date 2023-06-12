// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

mod cmdline;
mod module;
mod output;
mod pipeline;

use anyhow::Result;
use cmdline::OutputFormat;

use crate::output::JsonOutput;
use crate::output::{OutputWriter, PlainOutput};
use crate::pipeline::Pipeline;

#[derive(Debug, PartialEq, Eq)]
struct ItemInfo {
    id: String,
    module: String,
}

fn get_output_writer(format: OutputFormat) -> Result<Box<dyn OutputWriter>> {
    let stdout = std::io::stdout();
    let stdout = stdout.lock();

    match format {
        OutputFormat::Plain => {
            let plain = PlainOutput::new(stdout);
            Ok(Box::new(plain))
        }
        OutputFormat::Json => {
            let json = JsonOutput::new(stdout);
            Ok(Box::new(json))
        }
    }
}

fn main() -> Result<()> {
    let cmdline = cmdline::parse();

    let pipeline = {
        let pipeline = Pipeline::new();

        cmdline
            .module_paths
            .into_iter()
            .for_each(|path| pipeline.process_module(path));

        pipeline
    };

    let mut output = get_output_writer(cmdline.output_format)?;

    output.write_output(pipeline.consume())
}
