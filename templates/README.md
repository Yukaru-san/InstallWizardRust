# Runtime templates

Use `python3 tools/build.py native` on each supported platform/architecture.
Collect the generated `wizard-<target>[.exe]` files here, then run
`python3 tools/build.py catalog`. Templates and the generated catalog are build
artifacts and are not checked into Git. See `docs/BUILD.md`.
