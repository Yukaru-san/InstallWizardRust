#!/usr/bin/env python3
"""Collect installed dependency licenses for a local binary release."""
import hashlib
import json
from pathlib import Path
import subprocess

root = Path(__file__).resolve().parents[1]
metadata = json.loads(subprocess.check_output(['cargo','metadata','--locked','--format-version=1'], cwd=root))
notices = ['InstallWizardRust — dependency notices\nCopyright (c) 2026 Yukaru-san\n']
seen = set()
for package in sorted(metadata['packages'], key=lambda package: (package['name'], package['version'])):
    notices.append(f"\n{package['name']} {package['version']} — {package.get('license') or 'See license file'}\n{package.get('repository') or ''}\n")
    directory = Path(package['manifest_path']).parent
    candidates = [path for path in directory.iterdir() if path.is_file() and path.name.lower().startswith(('license','copying','copyright','notice'))]
    if package.get('license_file'): candidates.append(directory / package['license_file'])
    for path in candidates:
        text = path.read_text(errors='replace')
        digest = hashlib.sha256(text.encode()).hexdigest()
        if digest in seen: continue
        seen.add(digest)
        notices.append(f'\n{package["name"]}/{path.name}\n{text}\n')
notices.append('\nBundled CJK font (derived from Noto Sans JP):\n' + (root / 'assets/fonts/OFL.txt').read_text())
output = root / 'assets' / 'THIRD_PARTY_NOTICES.txt'
output.parent.mkdir(parents=True, exist_ok=True)
output.write_text('\n'.join(notices))
print(output)

release = root / 'dist' / output.name
release.parent.mkdir(parents=True, exist_ok=True)
release.write_text(output.read_text())
