# InstallWizard

Owner and author: Yukaru-san.

InstallWizard packages files for sharing. A recipient chooses a destination and
extracts the files with their original folder structure. It does not register
software, execute payloads, create shortcuts, or install services.

The creator and generated wizard each provide GUI and CLI modes. A desktop launch
opens the GUI; an invocation from a terminal uses the CLI. Explicit mode flags
resolve ambiguous launch contexts. Windows and Linux ship single executables;
macOS uses an application bundle distributed as ZIP.

The creator also runs in a browser. Up to 500 MB of input stays on the visitor's
device. A worker calls the Rust/WASM package core and combines the archive with
published runtime templates. A web wizard is a static website offering its ZIP.

Audience: developers and people distributing folders of data, including people
unfamiliar with CLI tools. Success means an intelligible file-to-package mapping
and a predictable, non-destructive extraction flow. Defaults must preserve files
and show actionable errors. Native operations must remain cancellable.

Constraints: shared Rust business logic and Rust UI; Impeccable design guidance;
Windows, Linux, macOS x86-64 and ARM64; no GitHub Actions; no added AI attribution.
English and German UI are supported. Dependency license obligations remain intact.

Visual commitment: clean, simple and modern on every supported platform, with
flat colors only and no gradients. Use the same embedded UI fonts and shared
visual tokens for native and browser creator/extraction views.

Contrast requirement: WCAG AA text contrast of at least 4.5:1, and at least 3:1
for interactive boundaries and focus marks. Do not communicate state by color
alone. Decorative separators are not input boundaries.
