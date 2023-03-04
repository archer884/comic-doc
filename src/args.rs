use std::path::{Path, PathBuf};

use clap::Parser;

use crate::item::Item;

#[derive(Debug, Parser)]
pub struct Args {
    /// input file paths
    #[arg(required = true)]
    paths: Vec<String>,

    /// output directory
    #[arg(short, long)]
    output: Option<String>,

    /// overwrite existing files
    #[arg(short, long)]
    force: bool,
}

impl Args {
    pub fn parse() -> Self {
        Parser::parse()
    }

    pub fn items(&'_ self) -> impl Iterator<Item = Item> + '_ {
        self.paths.iter().filter_map(|s| {
            let path = Path::new(&s);

            if path.is_file() && s.ends_with(".cbz") {
                return Some(Item::Tarball(s.into()));
            }

            if path.is_dir() {
                return Some(Item::Directory(s.into()));
            }

            eprintln!("warn: bad path: {s}");
            None
        })
    }

    pub fn as_processing_context(&self) -> ProcessingContext {
        ProcessingContext {
            force: self.force,
            target: self.output.as_deref(),
        }
    }
}

#[derive(Debug)]
pub struct ProcessingContext<'a> {
    /// overwrite existing files or directories
    pub force: bool,

    /// output directory
    target: Option<&'a str>,
}

impl ProcessingContext<'_> {
    pub fn output_path(&self, path: &Path) -> PathBuf {
        match self.target {
            Some(target) => {
                if path.ends_with("cbz") {
                    let base = Path::new(target);
                    let name = path.file_name().expect("must not be root directory");
                    base.join(name)
                } else {
                    let name = path.file_name().expect("must not be root directory");
                    Path::new(target).join(name).with_extension("cbz")
                }
            }
            None => {
                if path.ends_with("cbz") {
                    path.into()
                } else {
                    path.with_extension("cbz")
                }
            }
        }
    }
}
