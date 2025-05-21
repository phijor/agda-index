use std::{
    path::PathBuf,
    sync::mpsc,
};

use anyhow::{Context, Result};
use threadpool::ThreadPool;

use crate::module::{Module, ModuleParser};

#[derive(Debug)]
pub struct Item {
    pub source_path: PathBuf,
    pub module: Module,
}

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
        self.pool
            .execute(move || match process_module(source_path) {
                Err(err) => eprintln!("Failed to process module: {err}"),
                Ok(item) => tx.send(item).context("Failed to send result for module")?,
            });
    }

    pub fn consume(self) -> Output {
        Output { rx: self.rx }
    }
}

fn process_module(source_path: PathBuf) -> Result<Item> {
    let parser = ModuleParser::new();

    let content = std::fs::read_to_string(&source_path)
        .with_context(|| format!("Failed to read module file at {}", source_path.display()))?;
    let module = parser
        .parse_module(&content)
        .with_context(|| format!("Failed to parse module {}", source_path.display()))?;

    Ok(Item {
        source_path,
        module,
    })
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
