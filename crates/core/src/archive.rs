use std::io::{self, Cursor, Read, Seek, SeekFrom, Write};

use sha2::{Digest, Sha256};
use zip::{ZipArchive, ZipWriter, write::SimpleFileOptions};

use crate::model::validate_name;
use crate::{Cancellation, Entry, Error, InputEntry, Manifest, Progress, Result};

const MAGIC: &[u8; 8] = b"IWPKG001";
const FOOTER_SIZE: u64 = 32;
const MAX_MANIFEST: u64 = 32 * 1024 * 1024;

/// A range of an executable containing the standalone ZIP payload.
#[derive(Clone, Debug)]
pub struct Package {
    pub manifest: Manifest,
    pub archive_offset: u64,
    pub archive_size: u64,
}

/// Normalize an optional icon to a small PNG with bounded decoding dimensions.
pub fn normalize_icon(bytes: &[u8]) -> Result<Vec<u8>> {
    if bytes.len() > 8 * 1024 * 1024 {
        return Err(Error::Invalid("Icon exceeds 8 MiB".into()));
    }
    let mut reader = image::ImageReader::new(Cursor::new(bytes)).with_guessed_format()?;
    let mut limits = image::Limits::default();
    limits.max_image_width = Some(4096);
    limits.max_image_height = Some(4096);
    reader.limits(limits);
    let image = reader
        .decode()
        .map_err(|e| Error::Invalid(format!("Invalid PNG/ICO: {e}")))?;
    let resized = image
        .resize(256, 256, image::imageops::FilterType::Lanczos3)
        .to_rgba8();
    let mut square = image::RgbaImage::new(256, 256);
    image::imageops::overlay(
        &mut square,
        &resized,
        i64::from((256 - resized.width()) / 2),
        i64::from((256 - resized.height()) / 2),
    );
    let mut output = Cursor::new(Vec::new());
    image::DynamicImage::ImageRgba8(square)
        .write_to(&mut output, image::ImageFormat::Png)
        .map_err(|e| Error::Invalid(e.to_string()))?;
    Ok(output.into_inner())
}

/// Build a standalone ZIP from lazily opened sources.
///
/// # Errors
/// Fails on invalid paths, changing source lengths, cancellation or I/O errors.
/// The caller owns and discards partial output on failure.
pub fn create_archive<W: Write + Seek>(
    output: W,
    name: &str,
    icon_png: Option<Vec<u8>>,
    entries: &[InputEntry],
    mut open: impl FnMut(usize) -> Result<Box<dyn Read>>,
    cancel: &Cancellation,
    mut progress: impl FnMut(Progress),
) -> Result<Manifest> {
    validate_name(name)?;
    let mut manifest = Manifest {
        version: 1,
        name: name.into(),
        icon_png,
        entries: entries
            .iter()
            .map(|e| Entry {
                path: e.path.clone(),
                directory: e.directory,
                size: e.size,
                executable: e.executable,
                sha256: if e.directory {
                    String::new()
                } else {
                    "0".repeat(64)
                },
            })
            .collect(),
    };
    manifest.validate()?;
    let total = manifest.total_size()?;
    let mut writer = ZipWriter::new(output);
    let mut completed = 0;
    let mut buffer = [0_u8; 64 * 1024];
    for (index, entry) in entries.iter().enumerate() {
        cancel.check()?;
        let options = SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated)
            .last_modified_time(zip::DateTime::default())
            .large_file(entry.size >= u32::MAX as u64)
            .unix_permissions(if entry.directory || entry.executable {
                0o755
            } else {
                0o644
            });
        if entry.directory {
            writer.add_directory(format!("{}/", entry.path), options)?;
            continue;
        }
        writer.start_file(&entry.path, options)?;
        let mut input = open(index)?;
        let mut hash = Sha256::new();
        let mut size = 0_u64;
        loop {
            cancel.check()?;
            let count = input.read(&mut buffer)?;
            if count == 0 {
                break;
            }
            size += count as u64;
            if size > entry.size {
                return Err(Error::Invalid(format!("Source changed: {}", entry.path)));
            }
            hash.update(&buffer[..count]);
            writer.write_all(&buffer[..count])?;
            completed += count as u64;
            progress(Progress {
                path: entry.path.clone(),
                completed,
                total,
            });
        }
        if size != entry.size {
            return Err(Error::Invalid(format!("Source changed: {}", entry.path)));
        }
        manifest.entries[index].sha256 = hex::encode(hash.finalize());
    }
    writer.finish()?;
    Ok(manifest)
}

/// Append a ZIP and manifest to an executable; the footer locates both without
/// parsing platform executable headers. Offsets refer to the complete output.
pub fn append_package(
    mut output: impl Write + Seek,
    mut archive: impl Read,
    manifest: &Manifest,
) -> Result<()> {
    manifest.validate()?;
    let offset = output.seek(SeekFrom::End(0))?;
    let size = io::copy(&mut archive, &mut output)?;
    let metadata = serde_json::to_vec(manifest)?;
    if metadata.len() as u64 > MAX_MANIFEST {
        return Err(Error::Invalid("Manifest too large".into()));
    }
    output.write_all(&metadata)?;
    output.write_all(&offset.to_le_bytes())?;
    output.write_all(&size.to_le_bytes())?;
    output.write_all(&(metadata.len() as u64).to_le_bytes())?;
    output.write_all(MAGIC)?;
    Ok(())
}

/// Inspect a package without decompressing it. Integrity verification is a
/// separate pass and must succeed before extraction begins.
pub fn inspect_package(input: &mut (impl Read + Seek)) -> Result<Option<Package>> {
    let length = input.seek(SeekFrom::End(0))?;
    if length < FOOTER_SIZE {
        return Ok(None);
    }
    input.seek(SeekFrom::End(-(FOOTER_SIZE as i64)))?;
    let mut footer = [0_u8; FOOTER_SIZE as usize];
    input.read_exact(&mut footer)?;
    if &footer[24..] != MAGIC {
        return Ok(None);
    }
    let number = |start: usize| {
        u64::from_le_bytes(footer[start..start + 8].try_into().expect("8-byte field"))
    };
    let offset = number(0);
    let size = number(8);
    let metadata_size = number(16);
    let end = offset
        .checked_add(size)
        .and_then(|n| n.checked_add(metadata_size))
        .and_then(|n| n.checked_add(FOOTER_SIZE));
    if end != Some(length) || metadata_size > MAX_MANIFEST {
        return Err(Error::Invalid("Invalid package footer".into()));
    }
    input.seek(SeekFrom::Start(offset + size))?;
    let mut bytes = vec![0; metadata_size as usize];
    input.read_exact(&mut bytes)?;
    let manifest: Manifest = serde_json::from_slice(&bytes)?;
    manifest.validate()?;
    Ok(Some(Package {
        manifest,
        archive_offset: offset,
        archive_size: size,
    }))
}

/// Limit ZIP seeking to the declared archive rather than the surrounding binary.
pub struct ArchiveSlice<'a, R> {
    input: &'a mut R,
    offset: u64,
    size: u64,
    position: u64,
}

impl<'a, R: Read + Seek> ArchiveSlice<'a, R> {
    pub fn new(input: &'a mut R, package: &Package) -> Self {
        Self {
            input,
            offset: package.archive_offset,
            size: package.archive_size,
            position: 0,
        }
    }
}
impl<R: Read + Seek> Read for ArchiveSlice<'_, R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let limit = (self.size - self.position).min(buf.len() as u64) as usize;
        self.input
            .seek(SeekFrom::Start(self.offset + self.position))?;
        let count = self.input.read(&mut buf[..limit])?;
        self.position += count as u64;
        Ok(count)
    }
}
impl<R: Read + Seek> Seek for ArchiveSlice<'_, R> {
    fn seek(&mut self, from: SeekFrom) -> io::Result<u64> {
        let position = match from {
            SeekFrom::Start(n) => i128::from(n),
            SeekFrom::Current(n) => i128::from(self.position) + i128::from(n),
            SeekFrom::End(n) => i128::from(self.size) + i128::from(n),
        };
        if position < 0 || position > i128::from(self.size) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Seek outside archive",
            ));
        }
        self.position = position as u64;
        Ok(self.position)
    }
}

impl Package {
    pub fn archive<'a, R: Read + Seek>(
        &self,
        input: &'a mut R,
    ) -> Result<ZipArchive<ArchiveSlice<'a, R>>> {
        let archive = ZipArchive::new(ArchiveSlice::new(input, self))?;
        if archive.len() != self.manifest.entries.len() {
            return Err(Error::Invalid("ZIP and manifest disagree".into()));
        }
        Ok(archive)
    }

    /// Verify names, lengths and SHA-256 of every uncompressed file.
    pub fn verify(
        &self,
        input: &mut (impl Read + Seek),
        cancel: &Cancellation,
        mut progress: impl FnMut(Progress),
    ) -> Result<()> {
        let mut archive = self.archive(input)?;
        let mut completed = 0;
        let total = self.manifest.total_size()?;
        let mut buffer = [0_u8; 64 * 1024];
        for (index, entry) in self.manifest.entries.iter().enumerate() {
            cancel.check()?;
            let mut file = archive.by_index(index)?;
            let expected_name = if entry.directory {
                format!("{}/", entry.path)
            } else {
                entry.path.clone()
            };
            if file.name() != expected_name
                || file.is_dir() != entry.directory
                || file.size() != entry.size
            {
                return Err(Error::Invalid(format!(
                    "ZIP metadata mismatch: {}",
                    entry.path
                )));
            }
            if entry.directory {
                continue;
            }
            let mut hash = Sha256::new();
            let mut size = 0;
            loop {
                cancel.check()?;
                let count = file.read(&mut buffer)?;
                if count == 0 {
                    break;
                }
                size += count as u64;
                if size > entry.size {
                    return Err(Error::Invalid("ZIP exceeds declared size".into()));
                }
                hash.update(&buffer[..count]);
                completed += count as u64;
                progress(Progress {
                    path: entry.path.clone(),
                    completed,
                    total,
                });
            }
            if size != entry.size || hex::encode(hash.finalize()) != entry.sha256 {
                return Err(Error::Invalid(format!(
                    "Integrity check failed: {}",
                    entry.path
                )));
            }
        }
        Ok(())
    }

    pub fn copy_archive(
        &self,
        input: &mut (impl Read + Seek),
        mut output: impl Write,
    ) -> Result<()> {
        let count = io::copy(&mut ArchiveSlice::new(input, self), &mut output)?;
        if count != self.archive_size {
            return Err(Error::Invalid("Truncated archive".into()));
        }
        Ok(())
    }
}
