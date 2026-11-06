# InstallWizardRust

A Rust rewrite of InstallWizard by **Yukaru-san**. Create a portable file
installer, then extract its contents through a GUI or CLI. Native and browser
creators share the package core and use a Rust UI.

## Run

Install stable Rust and a C/C++ toolchain for your platform. Then:

```sh
cargo build --release -p installwizard
./target/release/installwizard --gui
./target/release/installwizard --help
```

On Windows use `target\release\installwizard.exe`. On macOS the build tool can
also wrap the creator in an `.app` bundle. A desktop launch opens the GUI; a
terminal launch selects CLI. `--gui` and `--cli` explicitly choose a mode.

## Create and extract

```sh
installwizard create --name "My application" \
  --source ./my-app=my-app --source ./README.txt=docs/README.txt \
  --target linux-x64 --output ./installers

./installers/InstallWizard-linux-x64 list
./installers/InstallWizard-linux-x64 verify
./installers/InstallWizard-linux-x64 extract --destination ./unpacked
```

A directory source preserves everything under it, including empty folders and
hidden files. `SOURCE=destination` sets its root inside the package. Use an empty
destination to package only a directory's contents. Individual files can be
mapped to any portable relative path. Symbolic links and special files are
rejected. Output must be outside source folders.

Supply `--icon image.png` or `--icon image.ico` for the package icon. It appears
inside the wizard, in Windows executable resources, and in macOS app bundles.
Linux executable files have no portable file-manager icon convention.

The GUI can save or reopen a JSON project; the CLI accepts `--project file.json`.
Source/icon paths are resolved relative to the project file. CLI arguments replace
the corresponding project fields. No arguments in a terminal starts an interactive
creator or, for an embedded package, asks for an extraction destination.
Non-interactive execution needs complete arguments and never waits for input.

## Platforms and templates

| ID | Platform | Output |
| --- | --- | --- |
| `windows-x64`, `windows-arm64` | Windows | One `.exe` |
| `linux-x64`, `linux-arm64` | Linux | One ELF executable |
| `macos-x64`, `macos-arm64` | macOS | `.app` in a ZIP |

The creator does not invoke a compiler when creating installers. It reuses
prebuilt runtime templates. The running creator supplies its own platform by
default. For other platforms, obtain/build the matching templates and pass
`--templates ./templates`. `--target all` selects all six and requires all six
templates. Neither the repository nor the browser fabricates missing runtimes.

```sh
python3 tools/build.py native
python3 tools/build.py catalog --templates templates
installwizard create --project project.json --target all --templates templates
```

See [build and distribution instructions](docs/BUILD.md), including platform
prerequisites, template publication and macOS signing. This repository contains
**no GitHub Actions**.

## Web targets

```sh
# Static creator, with compiled Rust/WASM and any published native templates:
python3 tools/build.py web
python3 -m http.server --directory dist/web 8080

# Static wizard whose download contains the original files:
installwizard create --project project.json --web --output ./exports
```

Extract `InstallWizard-web.zip` and host its files together. Embed either website
using an iframe; hosting paths may be subdirectories. File data stays in the
browser. Browser packages are capped at 500 MB of input; memory-constrained devices
may need smaller inputs. Native creation/extraction streams file contents and
supports ZIP64. A worker keeps compression off the browser UI thread.

Directory uploads use the File System Access API where available, including empty
folders. The fallback directory picker preserves file paths but browser APIs do
not report empty folders or Unix executable permissions. Native creation retains
executable bits. Browser-downloaded Linux installers need `chmod +x <filename>`
before running. The web wizard's direct ZIP link works even if its metadata
script fails.

For native browser exports, publish `templates/catalog.json` and each template
beside the static creator. The catalog contains target IDs, filenames and SHA-256
digests; unavailable or mismatched templates produce an error before compression.

## Safe extraction

All entry names are portable relative paths. Traversal, absolute paths, Windows
device names, case/Unicode-equivalent duplicates and file/directory conflicts are
rejected. Every file is checked against its manifest digest before extraction;
a second check protects each file while it is written. Capability-relative
filesystem operations constrain writes to the chosen root.

Existing destination files are preserved by default. `--overwrite` explicitly
allows replacement; symbolic links remain forbidden. Each file is committed
atomically. Cancellation or disk errors can leave files that already completed;
temporary files are cleaned up. There is no whole-directory transaction or
rollback of pre-existing files. SHA-256 detects damage, not publisher identity.

## Development

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo check -p installwizard --lib --target wasm32-unknown-unknown
```

Architecture, format and style rules are in [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md).
Platform runtime tests require the corresponding OS; a cross-compile check is
not evidence that GUI launch, signing or shell integration works on that OS.

MIT license. Copyright © 2026 Yukaru-san. Dependency notices are embedded and available through `--licenses`; regenerate
them with `python3 tools/notices.py` when changing dependencies.
