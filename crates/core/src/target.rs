use serde::{Deserialize, Serialize};

use crate::{Error, Result};

/// Supported desktop targets. Template filenames are stable public identifiers.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Target {
    WindowsX64,
    WindowsArm64,
    LinuxX64,
    LinuxArm64,
    MacosX64,
    MacosArm64,
}

impl Target {
    pub const ALL: [Self; 6] = [
        Self::WindowsX64,
        Self::WindowsArm64,
        Self::LinuxX64,
        Self::LinuxArm64,
        Self::MacosX64,
        Self::MacosArm64,
    ];

    pub fn id(self) -> &'static str {
        match self {
            Self::WindowsX64 => "windows-x64",
            Self::WindowsArm64 => "windows-arm64",
            Self::LinuxX64 => "linux-x64",
            Self::LinuxArm64 => "linux-arm64",
            Self::MacosX64 => "macos-x64",
            Self::MacosArm64 => "macos-arm64",
        }
    }
    pub fn triple(self) -> &'static str {
        match self {
            Self::WindowsX64 => "x86_64-pc-windows-msvc",
            Self::WindowsArm64 => "aarch64-pc-windows-msvc",
            Self::LinuxX64 => "x86_64-unknown-linux-gnu",
            Self::LinuxArm64 => "aarch64-unknown-linux-gnu",
            Self::MacosX64 => "x86_64-apple-darwin",
            Self::MacosArm64 => "aarch64-apple-darwin",
        }
    }
    pub fn macos(self) -> bool {
        matches!(self, Self::MacosX64 | Self::MacosArm64)
    }
    pub fn windows(self) -> bool {
        matches!(self, Self::WindowsX64 | Self::WindowsArm64)
    }
    pub fn filename(self) -> String {
        format!(
            "wizard-{}{}",
            self.id(),
            if self.windows() { ".exe" } else { "" }
        )
    }
    pub fn parse(id: &str) -> Result<Self> {
        Self::ALL
            .into_iter()
            .find(|target| target.id() == id || target.triple() == id)
            .ok_or_else(|| Error::Invalid(format!("Unknown target: {id}")))
    }
}
