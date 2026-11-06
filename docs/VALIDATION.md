# Validation record

Validated in the Linux cloud workspace on 2026-11-06. Source and build helpers
are delivered on `main`; no GitHub Actions, PR or release is part of this work.

## Passed

- `cargo fmt --all --check` and native/WASM Clippy with warnings denied.
- Workspace tests: nine package tests, seven filesystem tests and one Rustdoc
  example. Includes traversal/device names, case/Unicode collisions, implicit
  parent aliases, footer overflow, corruption, cancellation, empty folders,
  hidden/Unicode files, Unix executable bits, overwrite and destination symlinks.
- Windows resource test: a minimal real PE fixture receives the chosen icon,
  still parses as a PE and carries a verifiable package afterward. The fixture
  is never executed; it does not replace Windows runtime testing.
- Native Linux creation -> executable -> list -> verify -> extract, with exact
  file bytes/structure and executable permissions checked.
- CLI SIGINT during creation: exit 130 and no committed partial export.
- Embedded dependency notices available through `--licenses`.
- Browser Rust/WASM application startup in Chromium, source picker, worker export,
  downloaded Linux installer execution/extraction, and nested web ZIP contents.
- Browser traversal and missing-template errors. No browser page exceptions.
- Actual shared Rust UI captures at desktop and 390 px widths, including German
  populated mappings, name/export views and busy/error states.
- Independent Impeccable fix review: stale preview, mapping clarity, input label
  associations and narrow layout findings all resolved, at that fix-list scope.
- Cross-compile checks for Windows x86-64 and macOS ARM64.

## Limits

Windows and macOS programs were compile-checked, not launched on those operating
systems. Other CPU targets are exposed by the build helpers and template format;
their native runtime tests are still required before shipping those artifacts.
Linux GUI window integration/file dialogs, browser screen readers, Firefox/Safari,
and a full 500 MB browser export were not exercised in this environment. Browser
memory availability varies by device. Signing/notarization requires the owner's
platform tools and credentials after the final payload/icon is assembled.

Templates for all platforms are built artifacts. They must be produced on the
appropriate machines and published with the static creator before all native
browser exports can be offered. Missing templates fail explicitly.

## Modern UI and contrast refresh

The shared native/WASM UI now uses embedded Inter Regular/Semibold, a bounded
workspace and flat colors. The HTML wizard follows the same visual vocabulary.
Desktop and German 390 px captures cover sources, name, platform selection,
busy/error states and the web wizard. Actual browser interaction verified the
localized missing-template error, preserved inputs, switching to Web-Wizard,
retrying successfully and downloading the generated ZIP. ZIP contents were checked.
The independent Impeccable finish review approved the visual changes and scored
its required error-recovery fix resolved. The automated detector was unavailable.

`python3 tools/check-contrast.py` passes all 15 enabled-state palette checks:
text minimum 5.64:1 and input boundaries minimum 3.35:1. See CONTRAST.md for the
scope and WCAG references. Rust tests, native/WASM Clippy and Windows x64/macOS
ARM64 compile checks pass. Existing native GUI/screen-reader limitations remain.
