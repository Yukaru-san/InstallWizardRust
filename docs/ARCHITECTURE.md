# Architecture and coding conventions

## Dependency direction

`installwizard-core` contains the model, portable path policy, streaming ZIP and
manifest verification, icon conversion and portable export helpers. It does not
open native files or draw controls and is compiled for both native and WASM.

`installwizard-native` adapts filesystems, source traversal, project persistence,
capability-based extraction and streaming exports. Source collection creates a
sorted snapshot. Compression opens one file at a time, rejects changing lengths,
and reuses one ZIP for multiple selected native targets.

`installwizard` owns the CLI, Rust UI, native jobs and WASM entrypoints. The
presentation issues actions and consumes notices. Native work uses a background
thread, cancellation token and throttled channel. A browser worker invokes the
same Rust package code; JavaScript handles file pickers, transfer, template fetches
and downloads. No uploaded source code is ever executed.

Keep orchestration in adapters. Domain policy belongs to core; a widget should
not invent validation rules. Public types describe source mappings, target IDs,
manifest entries, progress and cancellation. Error types retain I/O causes.

## Package v1

A Windows/Linux installer is:

```
[executable template][standalone ZIP64 archive][UTF-8 manifest JSON][32-byte footer]
```

The footer contains `archive_offset: u64 LE`, `archive_size: u64 LE`,
`manifest_size: u64 LE`, and the eight bytes `IWPKG001`. Checked arithmetic must
place its end at EOF. Manifest size is limited to 32 MiB, entry count to 100,000.
A bounded seek adapter prevents ZIP parsing from reaching executable/manifest
bytes. No footer means the executable is a creator. A recognized but invalid
footer is an error, never a silent creator fallback.

The manifest has version 1, display name, optional normalized PNG bytes and ordered
entries containing path, directory flag, length, executable flag and SHA-256.
Directories have length zero and no digest. The ZIP must agree with the manifest
entry-for-entry. Digests are integrity checks, not digital signatures. Payloads
must be treated as untrusted; all limits and paths are validated before extraction.

On macOS, the executable template remains byte-for-byte unchanged. The same
package format, with archive offset zero, lives in
`Contents/Resources/payload.iwpkg`. The runtime discovers that relative to its
actual executable. Name and icon live in the app bundle metadata/resources.

A static web wizard is a ZIP of `index.html`, `manifest.json`, `payload.zip` and
optional `icon.png`. Its link remains usable without JavaScript. No template or
WASM runtime is needed just to download the payload.

## Style policy

Use the official [Rust Style Guide](https://doc.rust-lang.org/style-guide/) and
[Rust API Guidelines](https://rust-lang.github.io/api-guidelines/). Use stable
`rustfmt` defaults. Imports precede module declarations and other items; rustfmt
sorts imports. Split modules by responsibility, not arbitrary line counts.

The following are project conventions, not universal Rust mandates:

- Declare variables close to first use. Prefer immutable bindings, short scopes
  and explicit ownership over shared mutable state.
- Keep constructors first in an implementation, followed by related public
  operations and private helpers. Group by responsibility, not alphabetically.
- Order fields to explain the domain. Do not reorder serialized fields or repr(C)
  layouts mechanically. Keep related state together.
- Use `snake_case` for functions/modules/variables, `UpperCamelCase` for types,
  `SCREAMING_SNAKE_CASE` for constants. Crate directories describe their role.
- Document the purpose and failure contract of public APIs with `///`, adding
  `# Errors`, examples, panic or safety notes where they convey something useful.
  Crate/module documentation uses `//!`.
- Comments explain reasons, invariants and non-obvious platform behavior. Usually
  one to three lines suffice; no arbitrary hard limit. Longer architectural
  discussions belong here or in a focused decision document. Do not narrate code.
- Avoid panic/unwrap for user input and I/O. Fixed-size slices already validated
  by their format may use `expect` with the invariant stated. Tests may unwrap.
- Unit tests target format and validation; filesystem tests target real boundaries.
  Runtime/platform tests verify actual launch and artifacts. Do not equate a
  compile check with execution on a platform.

## Compatibility and deliberate limits

No old Go installer compatibility. No registry/service changes, shortcuts,
auto-start, uninstall, telemetry, encryption or automatic execution of installed
content. POSIX permission preservation is limited to the executable flag. Symlinks,
ACLs, xattrs, hard-link identity and non-UTF-8 filenames are not preserved.
Portable names are intentionally stricter than individual host filesystems.

Extraction verifies before any destination writes and again per staged file.
The opened destination capability confines path resolution; existing symlinks are
also rejected explicitly. Commit is atomic per file, not across the package.
Generated artifacts should be signed after resource/payload modifications.
