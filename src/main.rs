// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

mod module;

use crate::module::ModuleParser;

use anyhow::{Context, Result};

use std::env;
use std::fs;
use std::path::Path;

#[derive(Debug, PartialEq, Eq)]
struct ItemInfo {
    id: String,
    module: String,
}

fn main() -> Result<()> {
    for file_arg in env::args_os().skip(1) {
        let path = Path::new(&file_arg);
        let content = fs::read_to_string(path).context("Failed to read file")?;

        let Some(parser) = ModuleParser::from_path(path) else {
            eprintln!("Failed to create module parser for {}", path.display());
            continue;
        };

        match parser.parse_module(&content) {
            Ok(module) => {
                for item in module.items {
                    println!(
                        r"file:///{path}#{id} {module}.{identifier}",
                        path = fs::canonicalize(path)?.display(),
                        id = item.id,
                        module = module.name,
                        identifier = item.identifier,
                    );
                }
            }
            Err(err) => {
                eprintln!("Failed to parse module: {}", err)
            }
        };
    }

    Ok(())
}
