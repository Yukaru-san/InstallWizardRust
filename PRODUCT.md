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
