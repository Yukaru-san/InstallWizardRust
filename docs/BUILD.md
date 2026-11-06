# Building and distributing

No GitHub Actions or hosted CI is used. Run the scripts locally or on machines
you control. A native template must be built and smoke-tested on its target OS.

## Toolchains

- Stable Rust (minimum 1.89), `rustfmt`, `clippy`, Python 3.11+ for build helpers.
- Windows: Visual Studio C++ Build Tools and the Windows SDK; use an MSVC Rust
  target. The GUI subsystem avoids a console window on desktop launch. CLI mode
  attaches to its parent's console. In shells that do not wait for GUI subsystem
  programs, use PowerShell `Start-Process -Wait` or a pipeline to wait explicitly.
- Linux: C toolchain and pkg-config. The app uses dynamically loaded X11/Wayland
  and graphics libraries; the GUI needs a desktop session and OpenGL support.
  Native file dialogs use the desktop portal. The CLI does not create a window.
  glibc compatibility follows the machine used to build the executable: build on
  the oldest distribution you intend to support, then test on newer ones.
- macOS: Xcode command line tools and the SDK. Use the x86-64 or aarch64 Rust target.
  Build/sign/notarize the final bundle on macOS when distributing to other people.

Build each architecture on a corresponding machine, or provide a configured cross
linker/SDK. Merely running `rustup target add` does not install linkers or SDKs.

```sh
python3 tools/build.py native --target linux-x64
python3 tools/build.py native --target windows-x64
python3 tools/build.py native --target macos-arm64
```

Each command builds the selected target and copies its runtime to
`templates/wizard-<id>[.exe]`. Collect all six templates in the same directory;
`python3 tools/build.py catalog` writes a catalog of those that actually exist.
Do not edit target IDs or copy a binary under another architecture's filename.
The creator verifies the executable header; the browser also verifies the catalog
SHA-256. Publish the catalog and templates atomically to avoid mixed releases.

A template is the creator executable without an embedded payload. Generated
installers automatically switch into extraction mode. Their CLI accepts `list`,
`verify`, and `extract`; it will not create another installer from its payload.

## Static web creator

Install `wasm-bindgen-cli` at exactly the version of `wasm-bindgen` in Cargo.lock,
then add the WASM target and run:

```sh
rustup target add wasm32-unknown-unknown
python3 tools/build.py web
python3 -m http.server --directory dist/web 8080
```

The helper builds WASM, generates the JavaScript bindings, copies the UI bridge
and worker, and publishes existing runtime templates. Serve over HTTPS in
production; localhost works for development. `file://` is not supported. Embed:

```html
<iframe src="/installwizard/" title="InstallWizard creator"
        style="width:100%;height:800px;border:0"
        allow="file-system-access"></iframe>
```

For a sandboxed iframe allow scripts, same-origin and downloads. Cross-origin
embeds may disable the directory API; the fallback picker still supports files.
Directory pickers may omit empty folders; browser file APIs do not expose Unix
executable bits. Keep the native creator available for those requirements.

The browser does not upload user files. Template fetches must stay on the same
origin. Source data is capped at 500 MB before reading; a device's memory limit
can be lower. Compression runs in a worker, and cancellation terminates that
worker. Output buttons start explicit downloads, avoiding automatic multi-download
blocking. No certificate or signing private key belongs in the website.

## Signing and macOS applications

The native helper emits a macOS creator bundle as well as the runtime template.
Generated macOS wizards are app bundles in ZIP files. Their main executable is
unchanged; the metadata and payload resource are specific to each package.

Signing a template does not sign later payloads or modified icons. After creating
the final bundle, sign it with your Developer ID, notarize it with Apple's tools,
and staple the result. For local ARM64 builds an ad-hoc signature may be required
on code; the template's original executable bytes are preserved in the bundle.
The complete generated bundle is not advertised as Developer-ID signed until you
perform the final signing step. Windows Authenticode signing likewise happens
**after** icon/payload changes. The static browser creator cannot notarize or
sign artifacts with a publisher's credentials. Unsigned downloads may trigger OS
security prompts. See Apple's [code signing guide](https://developer.apple.com/library/archive/technotes/tn2206/).

## Release checks

Run format, Clippy, core/filesystem tests, native build and WASM checks. Then test
on Windows/Linux/macOS for every shipped architecture: creator GUI, creator CLI,
installer GUI, installer CLI, paths with spaces/Unicode, overwritten files,
cancellation, icon display, and startup from a file manager or terminal.
Browser checks cover desktop and narrow layouts, uploads/folder mapping,
worker cancellation, unavailable/corrupt templates and downloaded ZIP contents.
A working Linux build is not a macOS/Windows runtime test.

Run `python3 tools/notices.py` to collect dependency license notices for binary
redistribution. Include the resulting `dist/THIRD_PARTY_NOTICES.txt` with the
release. Do not remove required third-party copyright/license notices.

For the optional browser smoke test, install `playwright-core` in a local Node
workspace, set `IW_PLAYWRIGHT_MODULE` to its import path if outside the project,
and run `node tools/browser-smoke.mjs` against the locally served creator.
`CHROMIUM_PATH` selects Chromium; `IW_WEB_URL` changes the server URL. Test artifacts
are written under `dist/browser-check`.
