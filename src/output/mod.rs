use anyhow::Result;

mod docset;
mod json;
mod plain;

pub use self::docset::DocsetOutput;
pub use self::json::JsonOutput;
pub use self::plain::PlainOutput;
pub use crate::pipeline::Output;

pub trait OutputWriter {
    fn write_output(&mut self, output: Output) -> Result<()>;
}
