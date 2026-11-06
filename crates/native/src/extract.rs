use std::{
    io::{Read, Seek, Write},
    path::{Path, PathBuf},
};

use sha2::{Digest, Sha256};

use cap_std::{
    ambient_authority,
    fs::{Dir, OpenOptions},
};
use installwizard_core::{Cancellation, Error, Package, Progress, Result};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Overwrite {
    Refuse,
    Replace,
}

/// Verify the full package, then write through a directory capability.
///
/// Existing symlinks are rejected, and capability-relative operations cannot
/// escape the selected root even if another process races path checks.
/// Each file is committed atomically; cancellation may leave completed files.
pub fn extract(
    input: &mut (impl Read + Seek),
    package: &Package,
    destination: &Path,
    overwrite: Overwrite,
    cancel: &Cancellation,
    mut progress: impl FnMut(Progress),
) -> Result<()> {
    package.verify(input, cancel, &mut progress)?;
    std::fs::create_dir_all(destination)?;
    let root = Dir::open_ambient_dir(destination, ambient_authority())?;
    for entry in &package.manifest.entries {
        check_components(&root, &entry.path)?;
        match root.symlink_metadata(&entry.path) {
            Ok(metadata) if entry.directory && metadata.is_dir() => {}
            Ok(metadata)
                if !entry.directory && metadata.is_file() && overwrite == Overwrite::Replace => {}
            Ok(_) => {
                return Err(Error::Invalid(format!(
                    "Destination already exists: {}",
                    entry.path
                )));
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(error.into()),
        }
    }
    // Reserve an unguessable staging directory. All temporary files remain
    // inside the root capability and are removed, including on cancellation.
    let staging = tempfile::Builder::new()
        .prefix(".installwizard-")
        .tempdir_in(destination)?;
    let stage_name = PathBuf::from(
        staging
            .path()
            .file_name()
            .ok_or_else(|| Error::Invalid("Missing staging name".into()))?,
    );
    let mut archive = package.archive(input)?;
    let mut completed = 0;
    let total = package.manifest.total_size()?;
    let mut buffer = [0_u8; 64 * 1024];
    for (index, entry) in package.manifest.entries.iter().enumerate() {
        cancel.check()?;
        check_components(&root, &entry.path)?;
        if entry.directory {
            root.create_dir_all(&entry.path)?;
            continue;
        }
        if let Some(parent) = Path::new(&entry.path).parent() {
            root.create_dir_all(parent)?;
        }
        let temporary = stage_name.join(index.to_string());
        let mut options = OpenOptions::new();
        options.write(true).create_new(true);
        let mut file = root.open_with(&temporary, &options)?;
        let mut source = archive.by_index(index)?;
        let mut hash = Sha256::new();
        let mut size = 0_u64;
        loop {
            cancel.check()?;
            let count = source.read(&mut buffer)?;
            if count == 0 {
                break;
            }
            size += count as u64;
            if size > entry.size {
                return Err(Error::Invalid("Archive changed during extraction".into()));
            }
            hash.update(&buffer[..count]);
            file.write_all(&buffer[..count])?;
            completed += count as u64;
            progress(Progress {
                path: entry.path.clone(),
                completed,
                total,
            });
        }
        if size != entry.size || hex::encode(hash.finalize()) != entry.sha256 {
            return Err(Error::Invalid(format!(
                "Archive changed during extraction: {}",
                entry.path
            )));
        }
        #[cfg(unix)]
        {
            use cap_std::fs::PermissionsExt;
            file.set_permissions(cap_std::fs::Permissions::from_mode(if entry.executable {
                0o755
            } else {
                0o644
            }))?;
        }
        file.sync_all()?;
        drop(file);
        check_components(&root, &entry.path)?;
        match overwrite {
            Overwrite::Refuse => {
                // A hard link provides atomic no-replace behavior on every host.
                root.hard_link(&temporary, &root, &entry.path)?;
                root.remove_file(&temporary)?;
            }
            Overwrite::Replace => root.rename(&temporary, &root, &entry.path)?,
        }
    }
    Ok(())
}

fn check_components(root: &Dir, path: &str) -> Result<()> {
    let mut current = PathBuf::new();
    for component in Path::new(path).components() {
        current.push(component);
        match root.symlink_metadata(&current) {
            Ok(metadata) if metadata.file_type().is_symlink() => {
                return Err(Error::Invalid(format!(
                    "Destination contains a symbolic link: {}",
                    current.display()
                )));
            }
            Ok(_) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => break,
            Err(error) => return Err(error.into()),
        }
    }
    Ok(())
}
