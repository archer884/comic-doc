use std::{
    collections::HashMap,
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

    // "Relative path" is used to track directories within the archive, so we
    // start with a blank directory.
    let has_content = populate_archive(path, Path::new(path.file_name().unwrap()), &mut archive)?;
    if !has_content {
        return Ok(());
    }

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

fn populate_archive<T>(
    path: &Path,
    relative_path: &Path,
    archive: &mut Builder<T>,
) -> io::Result<bool>
where
    T: io::Write,
{
    let (files, directories) = read_level(path)?;

    if files.is_empty() && directories.is_empty() {
        return Ok(false);
    }

    // If we have a webp subdirectory, we're probably going to completely
    // ignore these files.

    if let Some(webp) = get_webp_dir(&directories) {
        // What the code below does is that it appends files under webp as if
        // they were contained in the working directory. What we need to do
        // instead is combine those files with png files found in the working
        // directory, preferring the webp files.

        // E.g., if the files foo.png and foo.webp are found, take foo.webp,
        // because it's the original. If, on the other hand, files bar.png
        // and baz.webp are found, take both files. This is because the pnk
        // process did not create png duplicates of non-webp files.

        let webp_files = fs::read_dir(&webp.path)?.filter_map(|entry| {
            let entry = entry.ok()?;
            entry.file_type().ok()?.is_file().then(|| entry.path())
        });

        let webp_files: HashMap<_, _> = webp_files
            .filter_map(|file| {
                let name = file.file_stem().map(|name| name.to_owned());
                name.map(|name| (name, file))
            })
            .collect();

        let combined_files = files.into_iter().filter_map(|primary| {
            let name = primary.path.file_stem()?;
            let backup = webp_files.get(name).cloned();
            Some(backup.unwrap_or(primary.path))
        });

        append_files(archive, relative_path, combined_files)?;
    } else {
        append_files(
            archive,
            relative_path,
            files.into_iter().map(|meta| meta.path),
        )?;
    }

    // Do not descend into webp subdirectories
    let webp = Path::new("webp");
    let filtered_directories = directories
        .into_iter()
        .filter(|dir| !dir.path.ends_with(webp));

    for dir in filtered_directories {
        populate_archive(
            &dir.path,
            &relative_path.join(dir.path.file_name().unwrap()),
            archive,
        )?;
    }

    Ok(true)
}

fn append_files<T, I>(archive: &mut Builder<T>, relative_path: &Path, files: I) -> io::Result<()>
where
    T: io::Write,
    I: IntoIterator<Item = PathBuf>,
{
    for path in files {
        let mut file = fs::File::open(&path)?;
        archive.append_file(relative_path.join(path.file_name().unwrap()), &mut file)?;
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
