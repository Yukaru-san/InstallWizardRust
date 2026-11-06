use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

use serde::{Deserialize, Serialize};
use unicode_normalization::UnicodeNormalization;

use crate::{Error, Result, validate_path};

/// A source before its bytes have been read. `path` is its path in the package.
#[derive(Clone, Debug)]
pub struct InputEntry {
    pub path: String,
    pub directory: bool,
    pub size: u64,
    pub executable: bool,
}

/// A verified description of a ZIP entry. Directories have no digest.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Entry {
    pub path: String,
    pub directory: bool,
    pub size: u64,
    pub executable: bool,
    pub sha256: String,
}

/// Versioned package metadata, independent of an executable's platform.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Manifest {
    pub version: u32,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon_png: Option<Vec<u8>>,
    pub entries: Vec<Entry>,
}

impl Manifest {
    /// Validate names, portable paths and collisions before writing anything.
    pub fn validate(&self) -> Result<()> {
        if self.version != 1 {
            return Err(Error::Invalid("Unsupported package version".into()));
        }
        validate_name(&self.name)?;
        if self
            .icon_png
            .as_ref()
            .is_some_and(|icon| icon.len() > 1024 * 1024)
        {
            return Err(Error::Invalid("Icon exceeds 1 MiB".into()));
        }
        if let Some(icon) = &self.icon_png {
            let dimensions = image::ImageReader::with_format(
                std::io::Cursor::new(icon),
                image::ImageFormat::Png,
            )
            .into_dimensions()
            .map_err(|error| Error::Invalid(format!("Invalid package icon: {error}")))?;
            if dimensions.0 > 256 || dimensions.1 > 256 {
                return Err(Error::Invalid("Package icon exceeds 256 × 256".into()));
            }
        }
        if self.entries.len() > 100_000 {
            return Err(Error::Invalid("Package exceeds 100,000 entries".into()));
        }
        let mut paths = std::collections::BTreeMap::new();
        for entry in &self.entries {
            validate_path(&entry.path)?;
            let key = entry.path.nfc().collect::<String>().to_lowercase();
            if paths.insert(key, entry.directory).is_some() {
                return Err(Error::Invalid(format!("Duplicate path: {}", entry.path)));
            }
            if entry.directory {
                if entry.size != 0 || !entry.sha256.is_empty() || entry.executable {
                    return Err(Error::Invalid("Invalid directory metadata".into()));
                }
            } else if entry.sha256.len() != 64 || hex::decode(&entry.sha256).is_err() {
                return Err(Error::Invalid(format!("Invalid digest: {}", entry.path)));
            }
        }
        let mut directories = std::collections::BTreeMap::new();
        for entry in &self.entries {
            let mut parent = if entry.directory {
                entry.path.as_str()
            } else {
                entry.path.rsplit_once('/').map_or("", |(prefix, _)| prefix)
            };
            while !parent.is_empty() {
                let key = parent.nfc().collect::<String>().to_lowercase();
                if let Some(previous) = directories.insert(key, parent)
                    && previous != parent
                {
                    return Err(Error::Invalid(format!(
                        "Ambiguous directory names: {previous} and {parent}"
                    )));
                }
                parent = parent.rsplit_once('/').map_or("", |(prefix, _)| prefix);
            }
        }
        for path in paths.keys() {
            let mut parent = path.as_str();
            while let Some((prefix, _)) = parent.rsplit_once('/') {
                if paths.get(prefix) == Some(&false) {
                    return Err(Error::Invalid(format!("File is also a parent: {prefix}")));
                }
                parent = prefix;
            }
        }
        self.total_size()?;
        Ok(())
    }

    pub fn total_size(&self) -> Result<u64> {
        self.entries.iter().try_fold(0_u64, |sum, entry| {
            sum.checked_add(entry.size)
                .ok_or_else(|| Error::Invalid("Package size overflow".into()))
        })
    }
}

pub fn validate_name(name: &str) -> Result<()> {
    if name.trim().is_empty() || name.chars().count() > 120 || name.chars().any(char::is_control) {
        return Err(Error::Invalid(
            "Name must contain 1–120 printable characters".into(),
        ));
    }
    Ok(())
}

/// Cooperative cancellation shared with a background native operation.
#[derive(Clone, Default)]
pub struct Cancellation(Arc<AtomicBool>);

impl Cancellation {
    pub fn cancel(&self) {
        self.0.store(true, Ordering::Relaxed);
    }
    pub fn check(&self) -> Result<()> {
        if self.0.load(Ordering::Relaxed) {
            Err(Error::Cancelled)
        } else {
            Ok(())
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct Progress {
    pub path: String,
    pub completed: u64,
    pub total: u64,
}
