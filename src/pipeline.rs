use std::{
    path::PathBuf,
    sync::mpsc,
};

use anyhow::{Context, Result};
use threadpool::ThreadPool;

use crate::module::{Module, ModuleParser};

type Item = (PathBuf, Module);

#[derive(Debug)]
pub struct Pipeline {
    pool: ThreadPool,
    tx: mpsc::Sender<Item>,
    rx: mpsc::Receiver<Item>,
}

impl Pipeline {
    pub fn new() -> Self {
        let name = "agda-index-module-worker".into();
        let pool = threadpool::Builder::new().thread_name(name).build();

        let (tx, rx) = mpsc::channel();
        Self { pool, tx, rx }
    }

    pub fn process_module(&self, source_path: PathBuf) {
        let tx = self.tx.clone();
        self.pool.execute(move || {
            if let Err(err) = process_module(source_path, tx) {
                eprintln!("Failed to process module: {err}")
            }
        });
    }

    pub fn consume(self) -> Output {
        Output { rx: self.rx }
    }
}

fn process_module(source_path: PathBuf, result: mpsc::Sender<Item>) -> Result<()> {
    let parser = ModuleParser::from_path(&source_path).with_context(|| {
        format!(
            "Failed to create module parser for file at {}",
            source_path.display()
        )
    })?;

    let content = std::fs::read_to_string(&source_path)
        .with_context(|| format!("Failed to read module file at {}", source_path.display()))?;
    let module = parser
        .parse_module(&content)
        .with_context(|| format!("Failed to parse module {}", source_path.display()))?;

    result
        .send((source_path, module))
        .context("Failed to send result")?;

    Ok(())
}

pub struct Output {
    rx: mpsc::Receiver<Item>,
}

impl IntoIterator for Output {
    type Item = Item;

    type IntoIter = mpsc::IntoIter<Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.rx.into_iter()
    }
}
