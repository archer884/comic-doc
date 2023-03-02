use std::{
    io,
    path::{Path, PathBuf}, fs,
};

use tar::Builder;
use walkdir::WalkDir;

use crate::args::ProcessingContext;

/// A work item
///
/// An item may be a tarball with the .cbz extension, or it may be a directory
/// containing image data arranged in lexical order.
pub enum Item {
    Directory(PathBuf),
    Tarball(PathBuf),
}

impl Item {
    pub fn process(&self, context: &ProcessingContext) -> io::Result<()> {
        match self {
            Item::Directory(path) => create_tarball(context, path),
            Item::Tarball(path) => repair_tarball(context, path),
        }
    }
}

fn create_tarball(context: &ProcessingContext, path: &Path) -> io::Result<()> {
    let walker = WalkDir::new(path)
        .into_iter()
        .filter_map(|x| x.ok());

    let mut archive = Builder::new(Vec::new());
    
    
    todo!()
}

fn repair_tarball(context: &ProcessingContext, path: &Path) -> io::Result<()> {
    todo!()
}

struct ArchiveObject {
    name: String,
    subdirs: Vec<ArchiveObject>,
    files: Vec<BinaryObject>,
}



impl ArchiveObject {
    fn new(path: &Path) -> io::Result<Self> {
        let mut dirs = Vec::new();
        let mut files = Vec::new();

        for candidate in fs::read_dir(path)? {
            let entry = candidate?;
            let info = entry.metadata()?;

            if info.is_dir() {
                dirs.push(ArchiveObject::new(&entry.path())?);
            }

            if info.is_file() {
                files.push(BinaryObject::from_path)
            }



            if entry
        }

        let (files, dirs) = fs::read_dir(path)?
            .filter_map(|entry| {
                let entry = entry.ok()?;
                let meta = entry.metadata().ok()?;
                Some((entry.path(), meta))
            })
            .partition(|pair| pair.1.is_file());

        todo!()
            
    }
}

struct BinaryObject {
    name: String,
    data: Vec<u8>,
}

impl BinaryObject {
    fn new(path: &Path) -> io::Result<Self> {
        let name = path
            .file_name()
            .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "not a file"))?;

    }
}