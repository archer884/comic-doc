use std::path::Path;

use clap::Parser;

use crate::item::Item;

#[derive(Debug, Parser)]
pub struct Args {
    /// input file paths
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
    force: bool,

    /// output directory
    target: Option<&'a str>,
}
