use std::{
    ffi::OsStr,
    fs,
    io::Write,
    path::{Path, PathBuf},
};

use anyhow::{bail, Context, Result};
use rusqlite::params;

use super::{Output, OutputWriter};
use crate::module::{Item, Module};

#[derive(Debug)]
pub struct DocsetOutput {
    name: String,
    output_directory: PathBuf,
    input_directory: PathBuf,
    main_page: PathBuf,
}

impl DocsetOutput {
    pub fn new(
        name: String,
        output_directory: PathBuf,
        input_directory: PathBuf,
        main_page: PathBuf,
    ) -> Self {
        Self {
            name,
            output_directory,
            input_directory,
            main_page,
        }
    }

    fn docset_dir(&self) -> PathBuf {
        self.output_directory.join(format!("{}.docset", self.name))
    }

    fn documents_dir(&self) -> PathBuf {
        self.docset_dir().join("Contents/Resources/Documents/")
    }

    fn index_database_path(&self) -> PathBuf {
        self.docset_dir().join("Contents/Resources/docSet.dsidx")
    }

    fn check_exists(&self) -> Result<()> {
        let docset_dir = self.docset_dir();
        if docset_dir.exists() {
            bail!("Docset at {} already exists!", docset_dir.display());
        }

        Ok(())
    }

    fn create_skeleton(&self) -> Result<()> {
        let documents_dir = self.documents_dir();

        fs::create_dir_all(&documents_dir).with_context(|| {
            format!(
                "Failed to create docset directory at {}",
                documents_dir.display()
            )
        })
    }

    fn write_metadata(&self) -> Result<()> {
        let mut info_file = fs::File::create(self.docset_dir().join("Contents/Info.plist"))?;

        indoc::writedoc!(
            info_file,
            r#"
                <?xml version="1.0" encoding="UTF-8"?>
                <!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
                <plist version="1.0">
                <dict>
                    <key>CFBundleIdentifier</key>
                    <string>{name}</string>
                    <key>CFBundleName</key>
                    <string>{name}</string>
                    <key>DocSetPlatformFamily</key>
                    <string>{name}</string>
                    <key>isDashDocset</key>
                    <true/>
                    <key>dashIndexFilePath</key>
                    <string>{index_file_path}</string>
                </dict>
                </plist>
            "#,
            name = self.name,
            index_file_path = self.main_page.display(),
        )?;
        Ok(())
    }

    fn write_icon(&self) -> Result<()> {
        const AGDA_SVG: &'static [u8] = include_bytes!("../../resources/Agda.svg");

        let icon_path = self.docset_dir().join("icon.svg");

        eprintln!("Writing Docset icon to '{}'", icon_path.display());
        std::fs::write(&icon_path, AGDA_SVG)
            .with_context(|| format!("Failed to write Docset icon to '{}'", icon_path.display()))
    }

    fn index_database(&mut self) -> Result<IndexDatabase> {
        IndexDatabase::new(self.index_database_path(), &self.input_directory)
    }
}

#[derive(Debug)]
struct IndexDatabase {
    connection: rusqlite::Connection,
    documents_dir: PathBuf,
    input_directory: PathBuf,
}

impl IndexDatabase {
    pub fn new<DbPath, InPath>(db_path: DbPath, input_directory: InPath) -> Result<Self>
    where
        DbPath: AsRef<Path>,
        InPath: AsRef<Path>,
    {
        let documents_dir = db_path
            .as_ref()
            .parent()
            .ok_or_else(|| anyhow::anyhow!("No parent directory for index database"))?
            .join("Documents");
        let connection = rusqlite::Connection::open(db_path)?;
        Ok(Self {
            connection,
            documents_dir,
            input_directory: input_directory.as_ref().into(),
        })
    }

    pub fn create_schema(&self) -> Result<()> {
        self.connection.execute(
            r"CREATE TABLE searchIndex(
                id INTEGER PRIMARY KEY,
                name TEXT,
                type TEXT,
                path TEXT
            );",
            rusqlite::params![],
        )?;

        self.connection.execute(
            "CREATE UNIQUE INDEX anchor ON searchIndex (name, type, path);",
            rusqlite::params![],
        )?;

        Ok(())
    }
}

impl OutputWriter for IndexDatabase {
    fn write_output(&mut self, output: Output) -> Result<()> {
        let css_in = self.input_directory.join("Agda.css");
        if css_in.exists() {
            let css_out = self.documents_dir.join("Agda.css");
            eprintln!(
                "Found custom Agda CSS: Copying {} → {}",
                css_in.display(),
                css_out.display()
            );
            std::fs::copy(css_in, css_out)?;
        }

        let tsx = self.connection.transaction()?;
        {
            let mut insert_item = tsx.prepare(
                r"INSERT OR IGNORE INTO searchIndex(name, type, path) VALUES (?1, ?2, ?3);",
            )?;

            for item in output.into_iter() {
                let Module {
                    name: module_name,
                    items,
                } = item.module;

                let module_path = ModulePath(
                    item.source_path
                        .file_name()
                        .ok_or_else(|| anyhow::anyhow!("No file name on source module"))?,
                );

                {
                    let module_target_path = self.documents_dir.join(module_path.0);
                    eprintln!(
                        "Copying module {} → {}",
                        item.source_path.display(),
                        module_target_path.display()
                    );
                    std::fs::copy(&item.source_path, &module_target_path).with_context(|| {
                        format!(
                            "Failed to copy {module_name} to {}",
                            module_target_path.display()
                        )
                    })?;
                }

                insert_item.execute(params![module_name, "Module", module_path])?;

                for Item { id, identifier, classes } in items {
                    let fqn = format!("{module_name}.{identifier}");
                    insert_item
                        .execute(params![
                            &fqn,
                            "Function",
                            module_path.anchored_index_path(&id)
                        ])
                        .with_context(|| format!("Failed to write entry {fqn}"))?;
                }
            }
        }
        tsx.commit()?;

        Ok(())
    }
}

impl OutputWriter for DocsetOutput {
    fn write_output(&mut self, output: Output) -> Result<()> {
        self.check_exists()?;
        self.create_skeleton()
            .context("Failed to create Docset skeleton")?;
        self.write_metadata()
            .context("Failed to write Docset metadata")?;
        self.write_icon().context("Failed to write Docset icon")?;

        let mut db = self
            .index_database()
            .context("Failed to connect to index database")?;

        db.create_schema()
            .context("Failed to create index database schema")?;

        db.write_output(output)
    }
}

struct ModulePath<'file>(&'file OsStr);

impl<'file> ModulePath<'file> {
    fn anchored_index_path(&self, anchor: &str) -> String {
        let file_name = self.0.to_string_lossy();
        format!("{file_name}#{anchor}")
    }
}

impl rusqlite::ToSql for ModulePath<'_> {
    fn to_sql(&self) -> rusqlite::Result<rusqlite::types::ToSqlOutput> {
        let module_path_str = self.0.to_str().expect("Invalid module path");
        module_path_str.to_sql()
    }
}
