use std::{
    fs::{self, File},
    io::{self, Seek, SeekFrom, Write},
    path::{Path, PathBuf},
};

use installwizard_core::{
    Cancellation, Error, Manifest, Progress, Result, Target, append_package, icns, plist,
    web_bundle,
};
use zip::{ZipWriter, write::SimpleFileOptions};

use crate::{Project, collect, host_target};

pub struct ExportOptions {
    pub output: PathBuf,
    pub templates: Option<PathBuf>,
    pub overwrite: bool,
}

pub fn template_path(target: Target, directory: Option<&Path>) -> Result<PathBuf> {
    if let Some(directory) = directory {
        let path = directory.join(target.filename());
        if path.is_file() {
            return Ok(path);
        }
    }
    let executable = std::env::current_exe()?;
    if let Some(parent) = executable.parent() {
        let path = parent.join("templates").join(target.filename());
        if path.is_file() {
            return Ok(path);
        }
    }
    if target == host_target() {
        return Ok(executable);
    }
    Err(Error::Invalid(format!(
        "Missing {} template. Supply --templates <directory>; build it on the target platform first.",
        target.id()
    )))
}

/// Compress once, then reuse the payload for each selected runtime template.
pub fn build(
    project: &Project,
    options: &ExportOptions,
    cancel: &Cancellation,
    mut progress: impl FnMut(Progress),
) -> Result<Vec<PathBuf>> {
    if project.targets.is_empty() {
        return Err(Error::Invalid("Select at least one target".into()));
    }
    let mut targets = project.targets.clone();
    targets.sort_by_key(|target| target.id());
    targets.dedup();
    let templates = targets
        .iter()
        .map(|target| template_path(*target, options.templates.as_deref()))
        .collect::<Result<Vec<_>>>()?;
    reject_nested_output(project, &options.output)?;
    fs::create_dir_all(&options.output)?;
    let destinations = targets
        .iter()
        .map(|target| {
            options.output.join(format!(
                "InstallWizard-{}{}",
                target.id(),
                if target.macos() {
                    ".app.zip"
                } else if target.windows() {
                    ".exe"
                } else {
                    ""
                }
            ))
        })
        .collect::<Vec<_>>();
    for path in &destinations {
        if path.exists() && !options.overwrite {
            return Err(Error::Invalid(format!("Output exists: {}", path.display())));
        }
    }
    let prepared = collect(&project.sources)?;
    let mut payload = tempfile::tempfile()?;
    let manifest = prepared.write(&mut payload, project, cancel, &mut progress)?;
    for ((target, template), destination) in targets.iter().zip(&templates).zip(&destinations) {
        cancel.check()?;
        let mut header = [0; 4096];
        let count = std::io::Read::read(&mut File::open(template)?, &mut header)?;
        installwizard_core::validate_template(&header[..count], *target)?;
        let mut output = tempfile::NamedTempFile::new_in(&options.output)?;
        payload.seek(SeekFrom::Start(0))?;
        if target.macos() {
            macos_bundle(&mut output, &manifest, &mut payload, template)?;
        } else {
            if target.windows() && manifest.icon_png.is_some() {
                output.write_all(&installwizard_core::windows_icon(
                    &fs::read(template)?,
                    manifest.icon_png.as_deref().expect("checked icon"),
                )?)?;
            } else {
                io::copy(&mut File::open(template)?, &mut output)?;
            }
            append_package(&mut output, &mut payload, &manifest)?;
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                output
                    .as_file()
                    .set_permissions(fs::Permissions::from_mode(0o755))?;
            }
        }
        output.as_file().sync_all()?;
        if options.overwrite {
            output
                .persist(destination)
                .map_err(|e| Error::Io(e.error))?;
        } else {
            output
                .persist_noclobber(destination)
                .map_err(|e| Error::Io(e.error))?;
        }
    }
    Ok(destinations)
}

fn macos_bundle(
    output: impl Write + Seek,
    manifest: &Manifest,
    payload: &mut File,
    template: &Path,
) -> Result<()> {
    let mut writer = ZipWriter::new(output);
    let options = SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated)
        .unix_permissions(0o644)
        .large_file(true);
    let root = "InstallWizard.app/Contents";
    writer.start_file(
        format!("{root}/MacOS/InstallWizard"),
        options.unix_permissions(0o755),
    )?;
    io::copy(&mut File::open(template)?, &mut writer)?;
    writer.start_file(format!("{root}/Resources/payload.iwpkg"), options)?;
    let mut package = tempfile::tempfile()?;
    append_package(&mut package, payload, manifest)?;
    package.seek(SeekFrom::Start(0))?;
    io::copy(&mut package, &mut writer)?;
    writer.start_file(format!("{root}/Info.plist"), options)?;
    writer.write_all(plist(manifest).as_bytes())?;
    if let Some(icon) = &manifest.icon_png {
        writer.start_file(format!("{root}/Resources/AppIcon.icns"), options)?;
        writer.write_all(&icns(icon))?;
    }
    writer.finish()?;
    Ok(())
}

pub fn export_web(
    project: &Project,
    options: &ExportOptions,
    cancel: &Cancellation,
    progress: impl FnMut(Progress),
) -> Result<PathBuf> {
    reject_nested_output(project, &options.output)?;
    fs::create_dir_all(&options.output)?;
    let destination = options.output.join("InstallWizard-web.zip");
    let prepared = collect(&project.sources)?;
    let mut payload = tempfile::tempfile()?;
    let manifest = prepared.write(&mut payload, project, cancel, progress)?;
    let mut temporary = tempfile::NamedTempFile::new_in(&options.output)?;
    payload.seek(SeekFrom::Start(0))?;
    web_bundle(&mut temporary, &mut payload, &manifest)?;
    if options.overwrite {
        temporary
            .persist(&destination)
            .map_err(|e| Error::Io(e.error))?;
    } else {
        temporary
            .persist_noclobber(&destination)
            .map_err(|e| Error::Io(e.error))?;
    }
    Ok(destination)
}

fn reject_nested_output(project: &Project, output: &Path) -> Result<()> {
    let absolute = std::path::absolute(output)?;
    let mut ancestor = absolute.as_path();
    let mut suffix = Vec::new();
    while !ancestor.exists() {
        suffix.push(
            ancestor
                .file_name()
                .ok_or_else(|| Error::Invalid("Invalid output directory".into()))?
                .to_os_string(),
        );
        ancestor = ancestor
            .parent()
            .ok_or_else(|| Error::Invalid("Invalid output directory".into()))?;
    }
    let mut resolved = fs::canonicalize(ancestor)?;
    for part in suffix.into_iter().rev() {
        resolved.push(part);
    }
    for source in &project.sources {
        if source.source.is_dir() && resolved.starts_with(fs::canonicalize(&source.source)?) {
            return Err(Error::Invalid(
                "Output must be outside source folders to avoid packaging generated installers"
                    .into(),
            ));
        }
    }
    Ok(())
}
