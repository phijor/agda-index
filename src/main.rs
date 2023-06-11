// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

mod module;

use crate::module::ModuleParser;

use anyhow::{Context, Result};
use module::Module;

use std::env;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::sync::mpsc;

#[derive(Debug, PartialEq, Eq)]
struct ItemInfo {
    id: String,
    module: String,
}

fn process_module(path: &Path) -> Result<Module> {
    let parser = ModuleParser::from_path(path).with_context(|| {
        format!(
            "Failed to create module parser for file at {}",
            path.display()
        )
    })?;

    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read module file at {}", path.display()))?;
    parser
        .parse_module(&content)
        .with_context(|| format!("Failed to parse module {}", path.display()))
}

fn spawn_workers<P>(module_paths: P) -> mpsc::Receiver<(PathBuf, Module)>
where
    P: Iterator<Item = PathBuf>,
{
    let name = "agda-index-module-worker".into();
    let pool = threadpool::Builder::new().thread_name(name).build();

    let (tx, rx) = mpsc::channel();

    module_paths.for_each(|path| {
        let tx = tx.clone();
        pool.execute(move || match process_module(&path) {
            Ok(module) => tx
                .send((path, module))
                .expect("Failed to send processed module"),
            Err(err) => eprintln!("Failed to process module: {}", err),
        })
    });

    rx
}

fn main() -> Result<()> {
    for (path, module) in spawn_workers(env::args_os().skip(1).map(PathBuf::from)) {
        let path = fs::canonicalize(path)?;
        for item in module.items {
            println!(
                r"file:///{path}#{id} {module}.{identifier}",
                path = path.display(),
                id = item.id,
                module = module.name,
                identifier = item.identifier,
            );
        }
    }
    Ok(())
}
