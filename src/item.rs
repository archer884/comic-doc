use std::{io, path::PathBuf};

use crate::{args::ProcessingContext, processor};

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
            Item::Directory(path) => processor::create_tarball(context, path),
            Item::Tarball(path) => processor::repair_tarball(context, path),
        }
    }
}
