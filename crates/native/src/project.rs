use std::{
    fs::{self, File},
    io::Write,
    path::{Path, PathBuf},
};

use installwizard_core::{
    Cancellation, Error, InputEntry, Manifest, Progress, Result, Target, create_archive,
    normalize_icon,
};
use serde::{Deserialize, Serialize};

/// A filesystem source mapped to a relative destination within the package.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Source {
    pub source: PathBuf,
    pub destination: String,
}

/// Saved authoring inputs. Source and icon paths are relative to this file.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Project {
    pub version: u32,
    pub name: String,
    pub icon: Option<PathBuf>,
    pub sources: Vec<Source>,
    pub targets: Vec<Target>,
}

impl Default for Project {
    fn default() -> Self {
        Self {
            version: 1,
            name: "My package".into(),
            icon: None,
            sources: vec![],
            targets: vec![host_target()],
        }
    }
}

impl Project {
    pub fn load(path: &Path) -> Result<Self> {
        let metadata = fs::metadata(path)?;
        if metadata.len() > 8 * 1024 * 1024 {
            return Err(Error::Invalid("Project file exceeds 8 MiB".into()));
        }
        let mut project: Self = serde_json::from_reader(File::open(path)?)?;
        if project.version != 1 {
            return Err(Error::Invalid("Unsupported project version".into()));
        }
        let base = path.parent().unwrap_or(Path::new("."));
        for source in &mut project.sources {
            source.source = base.join(&source.source);
        }
        project.icon = project.icon.map(|icon| base.join(icon));
        Ok(project)
    }
    pub fn save(&self, path: &Path) -> Result<()> {
        let base = fs::canonicalize(
            path.parent()
                .filter(|p| !p.as_os_str().is_empty())
                .unwrap_or(Path::new(".")),
        )?;
        let mut project = self.clone();
        for source in &mut project.sources {
            let absolute = fs::canonicalize(&source.source)?;
            source.source = absolute
                .strip_prefix(&base)
                .unwrap_or(&absolute)
                .to_path_buf();
        }
        if let Some(icon) = &project.icon {
            let absolute = fs::canonicalize(icon)?;
            project.icon = Some(
                absolute
                    .strip_prefix(&base)
                    .unwrap_or(&absolute)
                    .to_path_buf(),
            );
        }
        let mut temporary = tempfile::NamedTempFile::new_in(&base)?;
        serde_json::to_writer_pretty(&mut temporary, &project)?;
        temporary.write_all(b"\n")?;
        temporary
            .persist(path)
            .map_err(|error| Error::Io(error.error))?;
        Ok(())
    }
}

/// Stable, sorted snapshot of source paths and expected metadata.
pub struct Prepared {
    pub entries: Vec<InputEntry>,
    paths: Vec<PathBuf>,
}

impl Prepared {
    pub fn write(
        &self,
        output: impl std::io::Write + std::io::Seek,
        project: &Project,
        cancel: &Cancellation,
        progress: impl FnMut(Progress),
    ) -> Result<Manifest> {
        let icon = project
            .icon
            .as_ref()
            .map(|path| {
                if fs::metadata(path)?.len() > 8 * 1024 * 1024 {
                    return Err(Error::Invalid("Icon exceeds 8 MiB".into()));
                }
                normalize_icon(&fs::read(path)?)
            })
            .transpose()?;
        create_archive(
            output,
            &project.name,
            icon,
            &self.entries,
            |index| {
                if fs::symlink_metadata(&self.paths[index])?
                    .file_type()
                    .is_symlink()
                {
                    return Err(Error::Invalid("Source became a symbolic link".into()));
                }
                Ok(Box::new(File::open(&self.paths[index])?))
            },
            cancel,
            progress,
        )
    }
}

pub fn collect(sources: &[Source]) -> Result<Prepared> {
    if sources.is_empty() {
        return Err(Error::Invalid("Add at least one file or folder".into()));
    }
    let mut records = Vec::new();
    for source in sources {
        let metadata = fs::symlink_metadata(&source.source)?;
        if metadata.file_type().is_symlink() {
            return Err(Error::Invalid("Symbolic links are not supported".into()));
        }
        if metadata.is_dir() {
            for item in walkdir::WalkDir::new(&source.source)
                .follow_links(false)
                .sort_by_file_name()
            {
                let item = item.map_err(|e| Error::Invalid(e.to_string()))?;
                let relative = item
                    .path()
                    .strip_prefix(&source.source)
                    .map_err(|e| Error::Invalid(e.to_string()))?;
                let relative = relative
                    .components()
                    .map(|part| {
                        part.as_os_str()
                            .to_str()
                            .ok_or_else(|| Error::Invalid("Source filename is not UTF-8".into()))
                    })
                    .collect::<Result<Vec<_>>>()?
                    .join("/");
                let destination = if source.destination.is_empty() {
                    relative
                } else if relative.is_empty() {
                    source.destination.clone()
                } else {
                    format!("{}/{}", source.destination, relative)
                };
                if destination.is_empty() {
                    continue;
                }
                records.push(record(item.path(), destination)?);
            }
        } else {
            records.push(record(&source.source, source.destination.clone())?);
        }
    }
    records.sort_by(|a, b| a.0.path.cmp(&b.0.path));
    let (entries, paths) = records.into_iter().unzip();
    let prepared = Prepared { entries, paths };
    // The core owns collision and portable-name policy for every frontend.
    let check = Manifest {
        version: 1,
        name: "validation".into(),
        icon_png: None,
        entries: prepared
            .entries
            .iter()
            .map(|entry| installwizard_core::Entry {
                path: entry.path.clone(),
                directory: entry.directory,
                size: entry.size,
                executable: entry.executable,
                sha256: if entry.directory {
                    String::new()
                } else {
                    "0".repeat(64)
                },
            })
            .collect(),
    };
    check.validate()?;
    Ok(prepared)
}

fn record(path: &Path, destination: String) -> Result<(InputEntry, PathBuf)> {
    let metadata = fs::symlink_metadata(path)?;
    if !metadata.is_dir() && !metadata.is_file() {
        return Err(Error::Invalid(format!(
            "Unsupported source: {}",
            path.display()
        )));
    }
    #[cfg(unix)]
    let executable = {
        use std::os::unix::fs::PermissionsExt;
        metadata.is_file() && metadata.permissions().mode() & 0o111 != 0
    };
    #[cfg(not(unix))]
    let executable = false;
    Ok((
        InputEntry {
            path: destination,
            directory: metadata.is_dir(),
            size: if metadata.is_dir() { 0 } else { metadata.len() },
            executable,
        },
        path.to_path_buf(),
    ))
}

pub fn host_target() -> Target {
    match (std::env::consts::OS, std::env::consts::ARCH) {
        ("windows", "aarch64") => Target::WindowsArm64,
        ("windows", _) => Target::WindowsX64,
        ("macos", "aarch64") => Target::MacosArm64,
        ("macos", _) => Target::MacosX64,
        ("linux", "aarch64") => Target::LinuxArm64,
        _ => Target::LinuxX64,
    }
}
