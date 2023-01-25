use std::env;
use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use scraper::node::Element;
use scraper::{Html, Selector};
use url::{self, Url};

#[derive(Debug, PartialEq, Eq)]
struct ItemInfo {
    id: String,
    module: String,
}

fn parse_module_name(path: &str) -> Option<String> {
    let path = Path::new(path);
    path.file_stem()?.to_str().map(String::from)
}

fn parse_item_info(element: &Element, parse_opts: &url::ParseOptions<'_>) -> Option<ItemInfo> {
    let target = element.attr("href")?;
    let url = parse_opts.parse(target).ok()?;

    let id = url.fragment()?.into();
    let module = parse_module_name(url.path())?;

    Some(ItemInfo { id, module })
}

fn scrape(module_path: String, content: &str) -> Result<()> {
    let document = Html::parse_document(content);
    let functions =
        Selector::parse("a.Function , a.Datatype , a.InductiveConstructor , a.Record , a.Field")
            .expect("Function selector");

    let base = Url::parse(&format!("file:///{}", module_path))?;
    let parse_opts = Url::options().base_url(Some(dbg!(&base)));
    let module = parse_module_name(&module_path).context("Not a valid module name")?;

    document
        .select(&functions)
        .filter_map(|func| {
            let this_item = ItemInfo {
                id: func.value().id()?.into(),
                module: module.clone(),
            };
            let that_item = parse_item_info(func.value(), &parse_opts)?;
            let ident = func.text().next()?;

            (this_item == that_item).then_some((this_item, ident))
        })
        .for_each(|(info, ident)| {
            println!("{:}.{:}", info.module, ident);
        });

    Ok(())
}

fn main() -> Result<()> {
    for file_arg in env::args_os().skip(1) {
        let path = Path::new(&file_arg);
        let content = fs::read_to_string(path).context("Failed to read file")?;

        scrape(
            path.file_name()
                .context("Path has no file name")?
                .to_str()
                .context("Not a valid string")?
                .into(),
            &content,
        )?;
    }

    Ok(())
}
