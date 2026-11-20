use std::collections::BTreeMap;
use std::fs::File;
use std::io::{self, Read};
use std::path::{Path, PathBuf};

use czkawka_core::tools::duplicate::blake3_digest;

use super::{EntryStamp, Plan, PlannedMove, at, invalid};

impl Plan {
    pub(super) fn match_hash_group(&mut self, sources: Vec<PathBuf>, destinations: Vec<PathBuf>) -> io::Result<()> {
        let source_groups = fingerprint_group(&self.a, sources, &self.a_entries, &mut self.source_hashes)?;
        let mut destination_groups = fingerprint_group(&self.b, destinations, &self.b_entries, &mut self.template_hashes)?;

        for (hash, mut sources) in source_groups {
            let mut destinations = destination_groups.remove(&hash).unwrap_or_default();
            if sources.len() == 1 && destinations.len() == 1 {
                self.move_files.push(PlannedMove {
                    source: sources.remove(0),
                    destination: destinations.remove(0),
                });
            } else {
                self.ambiguous.extend(sources);
                self.blocked.extend(destinations);
            }
        }
        self.blocked.extend(destination_groups.into_values().flatten());
        Ok(())
    }
}

fn fingerprint_group(
    root: &Path,
    candidates: Vec<PathBuf>,
    stamps: &BTreeMap<PathBuf, EntryStamp>,
    hashes: &mut BTreeMap<PathBuf, String>,
) -> io::Result<BTreeMap<String, Vec<PathBuf>>> {
    let mut groups: BTreeMap<String, Vec<PathBuf>> = BTreeMap::new();
    for relative in candidates {
        let path = root.join(&relative);
        let stamp = stamps.get(&relative).ok_or_else(|| invalid(&path, "Hash candidate is absent from the scan"))?;
        let digest = fingerprint(&path, stamp)?;
        hashes.insert(relative.clone(), digest.clone());
        groups.entry(digest).or_default().push(relative);
    }
    Ok(groups)
}

fn fingerprint(path: &Path, expected: &EntryStamp) -> io::Result<String> {
    let file = at(path, open_for_hash(path))?;
    if &handle_stamp(path, &file)? != expected {
        return Err(invalid(path, "File changed before hashing"));
    }

    let mut reader = (&file).take(expected.len);
    let digest = at(path, blake3_digest(&mut reader))?;
    if reader.limit() != 0 || &handle_stamp(path, &file)? != expected || &EntryStamp::read(path)? != expected {
        return Err(invalid(path, "File changed while hashing"));
    }
    Ok(digest)
}

fn handle_stamp(path: &Path, file: &File) -> io::Result<EntryStamp> {
    #[cfg(windows)]
    let (metadata, device, inode) = super::windows_handle_metadata(path, file)?;
    #[cfg(not(windows))]
    let metadata = at(path, file.metadata())?;
    EntryStamp::from_metadata(
        path,
        &metadata,
        #[cfg(windows)]
        (device, inode),
    )
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn open_for_hash(path: &Path) -> io::Result<File> {
    use rustix::fs::{Mode, OFlags, openat};

    let (parent, name) = super::super::execute::open_parent(path)?;
    let flags = OFlags::RDONLY | OFlags::NOFOLLOW | OFlags::NONBLOCK | OFlags::CLOEXEC;
    Ok(File::from(openat(parent, name, flags, Mode::empty())?))
}

#[cfg(windows)]
fn open_for_hash(path: &Path) -> io::Result<File> {
    use std::os::windows::fs::OpenOptionsExt;

    use windows_sys::Win32::Storage::FileSystem::FILE_FLAG_OPEN_REPARSE_POINT;

    File::options().read(true).custom_flags(FILE_FLAG_OPEN_REPARSE_POINT).open(path)
}

#[cfg(not(any(target_os = "linux", target_os = "macos", windows)))]
fn open_for_hash(_path: &Path) -> io::Result<File> {
    Err(io::Error::new(io::ErrorKind::Unsupported, "Secure mirror hashing is unavailable"))
}
