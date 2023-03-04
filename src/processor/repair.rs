use std::{
    ffi::OsStr,
    fs,
    io::{self, Cursor, Read, Seek, SeekFrom},
    path::{Path, PathBuf},
    time,
};

use hashbrown::HashMap;
use tar::{Archive, Builder, Header};

use crate::args::ProcessingContext;

pub fn repair_tarball(context: &ProcessingContext, path: &Path) -> io::Result<()> {
    // We begin by dumping the archive into memory.

    let source = fs::read(path)?;
    let mtime = get_mtime();

    // We're going to need the archive reader itself, the entries contained by
    // the archive, and a list of the directories in the archive. I say all
    // that as if I actually KNOW what we need, which I don't... but work with
    // me, ok?

    let mut archive = Archive::new(Cursor::new(&source));
    let mut entries = HashMap::new();
    let mut directories = Vec::new();

    for entry in archive.entries_with_seek()? {
        let entry = entry?;
        let entry: Entry = entry.into();

        // If the entry is of size zero, it's a directory. THis means it isn't
        // really an entry, and we're going to make a note of it and move on.
        if entry.len == 0 {
            directories.push(entry.path);
            continue;
        }

        entries
            .entry(entry.parent())
            .or_insert_with(Vec::new)
            .push(entry);
    }

    // We need to sort the directories listing in order to ensure that it's in
    // order. The FIRST of these should PROBABLY be the root. I think.

    directories.sort();
    entries
        .values_mut()
        .for_each(|listing| listing.sort_by(|a, b| a.path.cmp(&b.path)));

    // From here, we're going to iterate over the set of a_dirs (the set of
    // directories not containing webp files) while assuming that there MAY
    // be a b_dir for any a_dir containing original webp files.

    let a_dirs = directories.iter().filter(|&dir| !dir.ends_with("webp"));

    // We also need a destination archive.

    let mut builder = Builder::new(Vec::new());

    // ****************************************
    // I have no idea what this next part does.
    // Neither does the ******* compiler.
    // ****************************************

    for dir in a_dirs {
        let candidate_b_dir = dir.join("webp");
        let a_files = match entries.get(dir) {
            Some(a_files) => a_files,
            None => continue,
        };

        let b_files = entries
            .get(&candidate_b_dir)
            .map(|b_files| {
                b_files
                    .iter()
                    .map(|entry| (entry.file_stem(), entry))
                    .collect()
            })
            .unwrap_or_else(HashMap::new);

        let combined_files = a_files.iter().map(|a| {
            let b_entry = b_files.get(a.file_stem()).copied();
            b_entry.unwrap_or(a)
        });

        // We now need to insert our files into the new archive. Assuming
        // these are actually files and that they can be read and that they
        // aren't corrupted by the fact that we're reading this out of order
        // and...

        for file in combined_files {
            let path = dir.join(file.file_name());
            let mut header = Header::new_gnu();
            header.set_size(file.len);
            header.set_mtime(mtime);
            builder.append_data(&mut header, path, file.read(&source))?;
        }
    }

    let path = context.output_path(path);
    if path.exists() && !context.force {
        return Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            "intended path exists",
        ));
    }

    fs::write(path, builder.into_inner()?)
}

struct Entry {
    path: PathBuf,
    offset: u64,
    len: u64,
}

impl Entry {
    fn parent(&self) -> PathBuf {
        self.path.parent().unwrap().into()
    }

    fn file_name(&self) -> &OsStr {
        self.path.file_name().unwrap()
    }

    fn file_stem(&self) -> &OsStr {
        self.path.file_stem().unwrap()
    }

    fn read<'a>(&self, source: &'a [u8]) -> impl io::Read + 'a {
        let mut cursor = Cursor::new(source);
        cursor.seek(SeekFrom::Start(self.offset)).unwrap();
        cursor.take(self.len)
    }
}

impl<T: io::Read> From<tar::Entry<'_, T>> for Entry {
    fn from(value: tar::Entry<T>) -> Self {
        // I have no idea if any of this garbage will work.
        // What do you want from me? Fuck bitches get money.
        Self {
            path: value.path().unwrap().into(),
            offset: value.raw_file_position(),
            len: value.size(),
        }
    }
}

fn get_mtime() -> u64 {
    let time = time::SystemTime::now();
    let duration = time.duration_since(time::UNIX_EPOCH).unwrap();
    duration.as_secs()
}
