use std::{
    ffi::OsStr,
    fs, io,
    path::{Path, PathBuf},
};

use tar::Builder;

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

struct PathMeta {
    path: PathBuf,
    metadata: fs::Metadata,
}

impl PathMeta {
    fn new(source: impl PathMetaSource) -> io::Result<Self> {
        source.into()
    }
}

trait PathMetaSource {
    fn into(self) -> io::Result<PathMeta>;
}

impl PathMetaSource for fs::DirEntry {
    fn into(self) -> io::Result<PathMeta> {
        Ok(PathMeta {
            metadata: self.metadata()?,
            path: self.path(),
        })
    }
}

fn create_tarball(context: &ProcessingContext, path: &Path) -> io::Result<()> {
    let mut archive = Builder::new(Vec::new());

    populate_archive(path, &mut archive)?;

    let path = context.output_path(path);
    if path.exists() && !context.force {
        return Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            "intended path exists",
        ));
    }

    fs::write(path, archive.into_inner()?)
}

fn repair_tarball(context: &ProcessingContext, path: &Path) -> io::Result<()> {
    todo!("We might never implement this...")
}

// PLAN: This function will recurse down each path. The level of recursion will
// be sharply limited, because these files were created by a human, and
// recursion will only occur in the even of a legitimate need.
//
// SPECIFICALLY: webp subdirectories will NOT incur recursion, because the
// contents of those will be archived IN LIEU of the contents of the parent.
fn populate_archive<T>(path: &Path, archive: &mut Builder<T>) -> io::Result<()>
where
    T: io::Write,
{
    let (files, directories) = read_level(path)?;

    // If we have a webp subdirectory, we're probably going to completely
    // ignore these files.

    if let Some(webp) = get_webp_dir(&directories) {
        let files = fs::read_dir(&webp.path)?.filter_map(|entry| {
            let entry = entry.ok()?;
            entry.file_type().ok()?.is_file().then(|| entry.path())
        });
        append_files(archive, files)?;
        return Ok(());
    }

    // We apparently DON'T have a webp directory, so we're going to continue.

    append_files(archive, files.into_iter().map(|meta| meta.path))?;

    for dir in directories {
        populate_archive(&dir.path, archive)?;
    }

    Ok(())
}

fn append_files<T, I>(archive: &mut Builder<T>, files: I) -> io::Result<()>
where
    T: io::Write,
    I: IntoIterator<Item = PathBuf>,
{
    for path in files {
        let mut file = fs::File::open(&path)?;
        archive.append_file(path.file_name().unwrap(), &mut file)?;
    }
    Ok(())
}

fn read_level(path: &Path) -> io::Result<(Vec<PathMeta>, Vec<PathMeta>)> {
    Ok(fs::read_dir(path)?
        .filter_map(|entry| PathMeta::new(entry.ok()?).ok())
        .partition(|meta| meta.metadata.is_file()))
}

fn get_webp_dir(dirs: &[PathMeta]) -> Option<&PathMeta> {
    if let [dir] = dirs {
        if dir.path.ends_with(OsStr::new("webp")) {
            return Some(dir);
        }
    }
    None
}
