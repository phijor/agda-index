use std::io::Write;

use anyhow::{Context, Result};

use super::OutputWriter;
use crate::{module::Module, pipeline::Output};

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

        for item in output.into_iter() {
            let path = fs::canonicalize(&item.source_path).with_context(|| {
                format!(
                    "Failed to canonicalize path {}",
                    &item.source_path.display()
                )
            })?;

            let Module {
                name: module_name,
                items,
            } = item.module;

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
