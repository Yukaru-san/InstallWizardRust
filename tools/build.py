#!/usr/bin/env python3
"""Local release tooling. Does not use GitHub Actions or upload anything."""
import argparse
import hashlib
import json
from pathlib import Path
import platform
import shutil
import subprocess
import tomllib

ROOT = Path(__file__).resolve().parents[1]
TARGETS = {
    'windows-x64': 'x86_64-pc-windows-msvc',
    'windows-arm64': 'aarch64-pc-windows-msvc',
    'linux-x64': 'x86_64-unknown-linux-gnu',
    'linux-arm64': 'aarch64-unknown-linux-gnu',
    'macos-x64': 'x86_64-apple-darwin',
    'macos-arm64': 'aarch64-apple-darwin',
}

def run(*args):
    subprocess.run(args, cwd=ROOT, check=True)

def host():
    os_name = {'Windows': 'windows', 'Linux': 'linux', 'Darwin': 'macos'}[platform.system()]
    arch = 'arm64' if platform.machine().lower() in ('arm64', 'aarch64') else 'x64'
    return f'{os_name}-{arch}'

def catalog(directory):
    directory.mkdir(parents=True, exist_ok=True)
    entries = []
    for target in TARGETS:
        name = f'wizard-{target}' + ('.exe' if target.startswith('windows') else '')
        path = directory / name
        if path.is_file():
            with path.open('rb') as source:
                digest = hashlib.file_digest(source, 'sha256').hexdigest()
            entries.append({'id': target, 'file': name, 'sha256': digest})
    (directory / 'catalog.json').write_text(json.dumps({'version': 1, 'targets': entries}, indent=2) + '\n')
    print(f'Catalog: {len(entries)} runtime(s) in {directory}')

def native(args):
    target = args.target or host()
    triple = TARGETS[target]
    run('cargo', 'build', '--locked', '--release', '-p', 'installwizard', '--target', triple)
    executable = ROOT / 'target' / triple / 'release' / ('installwizard.exe' if target.startswith('windows') else 'installwizard')
    args.templates.mkdir(parents=True, exist_ok=True)
    destination = args.templates / (f'wizard-{target}' + ('.exe' if target.startswith('windows') else ''))
    shutil.copy2(executable, destination)
    if target.startswith('macos'):
        import plistlib
        bundle = ROOT / 'dist' / target / 'InstallWizard.app' / 'Contents'
        (bundle / 'MacOS').mkdir(parents=True, exist_ok=True)
        shutil.copy2(executable, bundle / 'MacOS' / 'InstallWizard')
        (bundle / 'Info.plist').write_bytes(plistlib.dumps({
            'CFBundleExecutable': 'InstallWizard', 'CFBundleName': 'InstallWizard',
            'CFBundleIdentifier': 'io.github.yukaru-san.installwizard',
            'CFBundlePackageType': 'APPL', 'CFBundleVersion': '1',
            'NSHighResolutionCapable': True,
        }))
    catalog(args.templates)
    print(destination)

def web(args):
    lock = tomllib.loads((ROOT / 'Cargo.lock').read_text())
    version = next(p['version'] for p in lock['package'] if p['name'] == 'wasm-bindgen')
    try:
        installed = subprocess.check_output(['wasm-bindgen', '--version'], text=True).strip().split()[-1]
    except FileNotFoundError:
        raise SystemExit(f'Install bindings first: cargo install wasm-bindgen-cli --version {version} --locked')
    if installed != version:
        raise SystemExit(f'wasm-bindgen-cli must be {version}, found {installed}')
    run('cargo', 'build', '--locked', '--release', '-p', 'installwizard', '--lib', '--target', 'wasm32-unknown-unknown')
    output = ROOT / 'dist' / 'web'
    output.mkdir(parents=True, exist_ok=True)
    run('wasm-bindgen', '--target', 'web', '--out-dir', str(output / 'pkg'), str(ROOT / 'target/wasm32-unknown-unknown/release/installwizard.wasm'))
    for name in ['index.html', 'bridge.js', 'worker.js']:
        shutil.copy2(ROOT / 'web' / name, output / name)
    catalog(args.templates)
    if (output / 'templates').exists(): shutil.rmtree(output / 'templates')
    shutil.copytree(args.templates, output / 'templates', ignore=shutil.ignore_patterns('README.md'))
    print(f'Static creator: {output}')

def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('command', choices=['native', 'web', 'catalog'])
    parser.add_argument('--target', choices=TARGETS)
    parser.add_argument('--templates', type=Path, default=ROOT / 'templates')
    args = parser.parse_args()
    {'native': native, 'web': web, 'catalog': lambda a: catalog(a.templates)}[args.command](args)

if __name__ == '__main__': main()
