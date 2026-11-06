//! Shared Rust presentation for the native application and browser creator.
#[cfg(target_arch = "wasm32")]
mod browser;
#[cfg(not(target_arch = "wasm32"))]
pub mod cli;
mod components;
#[cfg(not(target_arch = "wasm32"))]
mod native_ui;
mod theme;
mod ui;
