#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

fn main() {
    #[cfg(not(target_arch = "wasm32"))]
    if let Err(error) = installwizard::cli::run() {
        eprintln!("{error}");
        std::process::exit(if matches!(error, installwizard_core::Error::Cancelled) {
            130
        } else {
            1
        });
    }
}
