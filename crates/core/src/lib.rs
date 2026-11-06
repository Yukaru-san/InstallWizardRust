//! Portable package model, validation and streaming archive format.
//!
//! Filesystem and presentation concerns belong to adapters. Package paths always
//! use `/`, regardless of the creator's operating system.
//!
//! ```
//! use installwizard_core::validate_path;
//! assert!(validate_path("assets/logo.png").is_ok());
//! assert!(validate_path("../outside.txt").is_err());
//! ```

mod archive;
mod error;
mod executable;
mod export;
mod model;
mod path;
mod target;

pub use archive::{Package, append_package, create_archive, inspect_package, normalize_icon};
pub use error::{Error, Result};
pub use model::{Cancellation, Entry, InputEntry, Manifest, Progress};
pub use path::validate_path;
pub use target::Target;

pub use export::{icns, memory_artifact, plist, web_bundle};

pub use executable::{validate_template, windows_icon};
