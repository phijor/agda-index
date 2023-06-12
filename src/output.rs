use std::{fmt, io::Write};

use anyhow::{Context, Result};
use serde_json::Serializer;

use crate::{module::Module, pipeline::Output};

pub trait OutputWriter {
    fn write_output(&mut self, output: Output) -> Result<()>;
}

pub struct JsonOutput<W> {
    serializer: Serializer<W>,
}

impl<W> fmt::Debug for JsonOutput<W> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("JsonOutput").finish_non_exhaustive()
    }
}

impl<W> JsonOutput<W>
where
    W: Write,
{
    pub fn new(writer: W) -> Self {
        let serializer = Serializer::new(writer);
        Self { serializer }
    }
}

impl<W> OutputWriter for JsonOutput<W>
where
    W: Write,
{
    fn write_output(&mut self, output: Output) -> Result<()> {
        use serde::Serializer;
        self.serializer
            .collect_map(output.into_iter())
            .context("Failed to write JSON output")
    }
}

#[derive(Debug)]
pub struct PlainOutput<W> {
    writer: W,
}

impl<W> PlainOutput<W> {
    pub fn new(writer: W) -> Self {
        Self { writer }
    }
}

impl<W> OutputWriter for PlainOutput<W>
where
    W: Write,
{
    fn write_output(&mut self, output: Output) -> Result<()> {
        use std::fs;

        for (path, module) in output.into_iter() {
            let path = fs::canonicalize(&path)
                .with_context(|| format!("Failed to canonicalize path {}", &path.display()))?;

            let Module {
                name: module_name,
                items,
            } = module;

            for item in items {
                writeln!(
                    &mut self.writer,
                    r"file:///{path}#{id} {module_name}.{identifier}",
                    path = path.display(),
                    id = item.id,
                    identifier = item.identifier,
                )
                .with_context(|| format!("Failed to write entry {module_name}.{item}"))?;
            }
        }

        Ok(())
    }
}
