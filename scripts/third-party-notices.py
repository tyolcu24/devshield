"""Generate notices from the locked target graph and installed production npm packages."""
import argparse
import json
import subprocess
from pathlib import Path
root = Path(__file__).resolve().parents[1]
parser = argparse.ArgumentParser()
parser.add_argument('--target', choices=['aarch64-apple-darwin', 'x86_64-apple-darwin', 'universal-apple-darwin', 'x86_64-pc-windows-msvc'], required=True)
args = parser.parse_args()
targets = ['aarch64-apple-darwin', 'x86_64-apple-darwin'] if args.target == 'universal-apple-darwin' else [args.target]
packages = {}
for target in targets:
    metadata = json.loads(subprocess.check_output(['cargo', 'metadata', '--locked', '--format-version', '1', '--filter-platform', target, '--manifest-path', str(root / 'src-tauri/Cargo.toml')], cwd=root))
    selected = {node['id'] for node in metadata['resolve']['nodes']}
    packages.update({item['id']: item for item in metadata['packages'] if item['id'] in selected and item['name'] != 'devshield'})
lines = ['# Third-party notices', '', f'Target: {args.target}. Generated from the locked build graph and frontend production dependencies. Project code is MIT licensed; dependency code retains its own licenses.', '']
def append_files(folder):
    for file in sorted(folder.iterdir()):
        if file.is_file() and any(file.name.upper().startswith(prefix) for prefix in ('LICENSE', 'COPYING', 'NOTICE', 'COPYRIGHT')):
            try:
                text = file.read_text(encoding='utf-8')
            except (UnicodeError, OSError):
                continue
            lines.extend([f'### {file.name}', '', '```text', text.strip(), '```', ''])
for item in sorted(packages.values(), key=lambda x: (x['name'], x['version'])):
    lines.extend([f"## Rust: {item['name']} {item['version']}", '', f"License expression: {item.get('license') or 'See upstream license files'}", f"Source: https://crates.io/crates/{item['name']}/{item['version']}", ''])
    append_files(Path(item['manifest_path']).parent)
lock = json.loads((root / 'package-lock.json').read_text(encoding='utf-8'))
for name, info in sorted(lock['packages'].items()):
    if not name or info.get('dev'):
        continue
    folder = root / name
    actual = json.loads((folder / 'package.json').read_text(encoding='utf-8'))
    lines.extend([f"## Frontend: {actual['name']} {info['version']}", '', f"License: {actual.get('license', 'See upstream')}", f"Source: https://www.npmjs.com/package/{actual['name']}/v/{info['version']}", ''])
    append_files(folder)
(root / 'THIRD_PARTY_NOTICES.md').write_text('\n'.join(lines), encoding='utf-8')
print(f'Generated target notices: {args.target}; {len(packages)} Rust dependencies')
