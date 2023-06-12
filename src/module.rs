// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
//
use std::{
    fmt::{self, Display},
    path::Path,
};

use anyhow::{Context, Result};
use scraper::{ElementRef, Html, Selector};
use serde::Serialize;
use url::{self, Url};

#[derive(Debug)]
pub struct ModuleParser {
    items: Selector,
    title: Selector,
    base_url: Url,
}

impl ModuleParser {
    pub fn new(base_url: Url) -> ModuleParser {
        const ITEM_SELECTOR: &str = r"
            .Agda .Function ,
            .Agda .Datatype ,
            .Agda .InductiveConstructor ,
            .Agda .CoinductiveConstructor ,
            .Agda .Record ,
            .Agda .Field
            ";
        ModuleParser {
            items: Selector::parse(ITEM_SELECTOR).expect("item selector"),
            title: Selector::parse("html title").expect("title selector"),
            base_url,
        }
    }

    pub fn from_path(path: &Path) -> Option<ModuleParser> {
        let base_path = path.parent()?;

        let base_url = Url::parse(&format!("file://{}/", base_path.display())).ok()?;

        Some(ModuleParser::new(base_url))
    }

    fn parse_target_item(&self, target: &Url) -> Result<(String, String)> {
        let id = target.fragment().context("No target ID")?;
        let path = self
            .base_url
            .make_relative(target)
            .context("Not a valid URL relative to base URL")?;

        let module = Path::new(&path)
            .file_stem()
            .context("No file stem found")?
            .to_str()
            .context("File name is not a valid module name")?;

        Ok((id.into(), module.into()))
    }

    fn parse_item(
        &self,
        item: ElementRef,
        module_name: &str,
        url_parser: url::ParseOptions,
    ) -> Result<Option<Item>> {
        let identifier = item.text().next().context("Missing text")?;
        let element = item.value();
        let id = element.id().context("Missing ID")?;
        let target_url = match element.attr("href") {
            Some(href) => url_parser.parse(href).context("Invalid link target")?,
            None => {
                // Some items are anchors but do not point anywhere.
                // Assume that these are definitions like `Y` in
                //
                //      import Foo.Bar renaming (X to Y)
                //
                // and return them anyways:
                return Ok(Some(Item {
                    id: id.into(),
                    identifier: identifier.into(),
                }));
            }
        };

        let (target_id, target_module) = self.parse_target_item(&target_url)?;

        let item = if id == target_id && module_name == target_module {
            Some(Item {
                id: id.into(),
                identifier: identifier.into(),
            })
        } else {
            None
        };

        Ok(item)
    }

    pub fn parse_module(&self, content: &str) -> Result<Module> {
        let url_parser = Url::options().base_url(Some(&self.base_url));
        let document = Html::parse_document(content);

        let name = document
            .select(&self.title)
            .next()
            .map(|el| el.inner_html())
            .context("No module name")?;

        let items = document
            .select(&self.items)
            .filter_map(|item| match self.parse_item(item, &name, url_parser) {
                Ok(item) => item,
                Err(err) => {
                    eprintln!("Warning: Skipping item ({})", err);
                    None
                }
            })
            .collect();

        Ok(Module { name, items })
    }
}

#[derive(Debug, Serialize)]
pub struct Item {
    pub id: String,
    pub identifier: String,
}

impl Display for Item {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{identifier}#{id}",
            identifier = self.identifier,
            id = self.id
        )
    }
}

#[derive(Debug, Serialize)]
pub struct Module {
    pub name: String,
    pub items: Vec<Item>,
}

impl Display for Module {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&self.name)
    }
}
