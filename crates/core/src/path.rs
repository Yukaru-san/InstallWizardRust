use crate::{Error, Result};

/// Reject ambiguous or platform-specific names rather than silently renaming.
///
/// # Errors
/// Rejects absolute/traversal paths, Windows device names, control characters,
/// backslashes and components that Windows would trim or reject.
pub fn validate_path(path: &str) -> Result<()> {
    let bad = || Error::Invalid(format!("Unsafe or non-portable package path: {path}"));
    if path.is_empty() || path.len() > 4096 || path.contains('\\') {
        return Err(bad());
    }
    for part in path.split('/') {
        if part.is_empty()
            || part == "."
            || part == ".."
            || part.len() > 255
            || part.ends_with(['.', ' '])
            || part
                .chars()
                .any(|c| c.is_control() || "<>:\"|?*".contains(c))
        {
            return Err(bad());
        }
        let device = part.split('.').next().unwrap_or("").to_ascii_uppercase();
        let numbered = device.len() == 4
            && (device.starts_with("COM") || device.starts_with("LPT"))
            && matches!(device.as_bytes()[3], b'1'..=b'9');
        let superscript = device
            .strip_prefix("COM")
            .or_else(|| device.strip_prefix("LPT"))
            .is_some_and(|suffix| matches!(suffix, "¹" | "²" | "³"));
        if matches!(device.as_str(), "CON" | "PRN" | "AUX" | "NUL") || numbered || superscript {
            return Err(bad());
        }
    }
    Ok(())
}
