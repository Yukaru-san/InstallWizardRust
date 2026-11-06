// Minimal PE fixture for resource-editing tests. Never executed.
// Rebuild with rustc for x86_64-pc-windows-msvc using rust-lld, /entry:mainCRTStartup,
// /subsystem:windows and /nodefaultlib; no Windows SDK or CRT is needed.
#![no_std]
#![no_main]

#[unsafe(no_mangle)]
pub extern "C" fn mainCRTStartup() -> ! {
    loop { core::hint::spin_loop(); }
}

#[panic_handler]
fn panic(_: &core::panic::PanicInfo<'_>) -> ! {
    loop { core::hint::spin_loop(); }
}
