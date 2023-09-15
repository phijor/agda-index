use std::{fmt, io::Write};

use anyhow::Result;
use serde::{ser::SerializeSeq, Serialize};
use serde_json::Serializer;

use super::OutputWriter;
use crate::{module::Module, pipeline::Output};

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

#[derive(Serialize)]
struct IndexItem<'n> {
    module: &'n str,
    identifier: String,
    href: String, // TODO: Extract the proper href from HTML
}

impl<W> OutputWriter for JsonOutput<W>
where
    W: Write,
{
    fn write_output(&mut self, output: Output) -> Result<()> {
        use serde::Serializer;

        let mut ser = self.serializer.serialize_seq(None)?;

        for item in output {
            let Module { name, items } = item.module;

            for item in items {
                ser.serialize_element(&IndexItem {
                    module: &name,
                    identifier: item.identifier,
                    href: format!("{}.html#{}", name, item.id),
                })?;
            }
        }

        ser.end()?;

        Ok(())
    }
}
