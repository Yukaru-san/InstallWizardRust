use std::io::{self, Cursor, Read, Seek, Write};

use zip::{ZipWriter, write::SimpleFileOptions};

use crate::{Manifest, Result, append_package};

/// Export a static website with a direct ZIP download and no server runtime.
pub fn web_bundle(
    output: impl Write + Seek,
    mut payload: impl Read,
    manifest: &Manifest,
) -> Result<()> {
    manifest.validate()?;
    let mut writer = ZipWriter::new(output);
    let stored = SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Stored)
        .large_file(true);
    writer.start_file("payload.zip", stored)?;
    io::copy(&mut payload, &mut writer)?;
    writer.start_file("manifest.json", stored)?;
    serde_json::to_writer(&mut writer, manifest)?;
    writer.start_file("index.html", stored)?;
    writer.write_all(include_bytes!("../../../web/wizard.html"))?;
    if let Some(icon) = &manifest.icon_png {
        writer.start_file("icon.png", stored)?;
        writer.write_all(icon)?;
    }
    writer.finish()?;
    Ok(())
}

/// Browser export uses in-memory buffers; the native adapter streams large data.
pub fn memory_artifact(
    template: &[u8],
    payload: &[u8],
    manifest: &Manifest,
    target: crate::Target,
) -> Result<Vec<u8>> {
    crate::validate_template(template, target)?;
    let mut output = Cursor::new(Vec::new());
    if !target.macos() {
        if target.windows() {
            if let Some(icon) = &manifest.icon_png {
                output.write_all(&crate::windows_icon(template, icon)?)?;
            } else {
                output.write_all(template)?;
            }
        } else {
            output.write_all(template)?;
        }
        append_package(&mut output, payload, manifest)?;
        return Ok(output.into_inner());
    }
    let mut writer = ZipWriter::new(&mut output);
    let options = SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated)
        .unix_permissions(0o644)
        .large_file(true);
    writer.start_file(
        "InstallWizard.app/Contents/MacOS/InstallWizard",
        options.unix_permissions(0o755),
    )?;
    writer.write_all(template)?;
    let mut resource = Cursor::new(Vec::new());
    append_package(&mut resource, payload, manifest)?;
    writer.start_file(
        "InstallWizard.app/Contents/Resources/payload.iwpkg",
        options,
    )?;
    writer.write_all(resource.get_ref())?;
    writer.start_file("InstallWizard.app/Contents/Info.plist", options)?;
    writer.write_all(plist(manifest).as_bytes())?;
    if let Some(icon) = &manifest.icon_png {
        writer.start_file("InstallWizard.app/Contents/Resources/AppIcon.icns", options)?;
        writer.write_all(&icns(icon))?;
    }
    writer.finish()?;
    Ok(output.into_inner())
}

pub fn plist(manifest: &Manifest) -> String {
    let name = manifest
        .name
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;");
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0"><dict>
<key>CFBundleExecutable</key><string>InstallWizard</string>
<key>CFBundleIdentifier</key><string>io.github.yukaru-san.installwizard</string>
<key>CFBundleName</key><string>{name}</string>
<key>CFBundleDisplayName</key><string>{name}</string>
<key>CFBundlePackageType</key><string>APPL</string>
<key>CFBundleVersion</key><string>1</string>
<key>CFBundleIconFile</key><string>AppIcon</string>
<key>NSHighResolutionCapable</key><true/>
</dict></plist>"#
    )
}

pub fn icns(png: &[u8]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(png.len() + 16);
    bytes.extend_from_slice(b"icns");
    bytes.extend_from_slice(&((png.len() + 16) as u32).to_be_bytes());
    bytes.extend_from_slice(b"ic08");
    bytes.extend_from_slice(&((png.len() + 8) as u32).to_be_bytes());
    bytes.extend_from_slice(png);
    bytes
}
