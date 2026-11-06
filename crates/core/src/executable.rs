use crate::{Error, Result, Target};

/// Reject templates for the wrong OS/architecture before packaging user files.
pub fn validate_template(bytes: &[u8], target: Target) -> Result<()> {
    let valid = if target.windows() {
        if bytes.get(..2) != Some(b"MZ") {
            false
        } else {
            let pe = bytes
                .get(0x3c..0x40)
                .map(|n| u32::from_le_bytes(n.try_into().expect("4-byte field")) as usize);
            pe.is_some_and(|offset| {
                bytes.get(offset..offset.saturating_add(4)) == Some(b"PE\0\0")
                    && bytes.get(offset.saturating_add(4)..offset.saturating_add(6))
                        == Some(if target == Target::WindowsX64 {
                            &[0x64, 0x86][..]
                        } else {
                            &[0x64, 0xaa][..]
                        })
            })
        }
    } else if target.macos() {
        bytes.get(..4) == Some(&[0xcf, 0xfa, 0xed, 0xfe])
            && bytes.get(4..8)
                == Some(if target == Target::MacosX64 {
                    &[7, 0, 0, 1][..]
                } else {
                    &[12, 0, 0, 1][..]
                })
    } else {
        bytes.get(..6) == Some(&[0x7f, b'E', b'L', b'F', 2, 1])
            && bytes.get(18..20)
                == Some(if target == Target::LinuxX64 {
                    &[62, 0][..]
                } else {
                    &[183, 0][..]
                })
    };
    if valid {
        Ok(())
    } else {
        Err(Error::Invalid(format!(
            "Runtime template does not match {}",
            target.id()
        )))
    }
}

/// Add a PNG-compressed ICO resource before appending the package or signing.
pub fn windows_icon(template: &[u8], png: &[u8]) -> Result<Vec<u8>> {
    let mut icon = vec![0, 0, 1, 0, 1, 0, 0, 0, 0, 0, 1, 0, 32, 0];
    icon.extend_from_slice(&(png.len() as u32).to_le_bytes());
    icon.extend_from_slice(&22_u32.to_le_bytes());
    icon.extend_from_slice(png);
    let mut image = editpe::Image::parse(template)
        .map_err(|e| Error::Invalid(format!("Invalid Windows template: {e:?}")))?;
    let mut resources = image.resource_directory().cloned().unwrap_or_default();
    resources
        .set_main_icon(icon.as_slice())
        .map_err(|e| Error::Invalid(format!("Could not set Windows icon: {e:?}")))?;
    image
        .set_resource_directory(resources)
        .map_err(|e| Error::Invalid(format!("Could not update Windows resources: {e:?}")))?;
    Ok(image.data().to_vec())
}
