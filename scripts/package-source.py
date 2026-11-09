"""Create a source-only archive; never include local inventories, secrets or tool caches."""
from pathlib import Path
import json
from zipfile import ZipFile, ZIP_DEFLATED
root = Path(__file__).resolve().parents[1]
version = json.loads((root / 'package.json').read_text())['version']
output = root / 'artifacts' / f'DevShield-{version}-source.zip'
output.parent.mkdir(exist_ok=True)
files = [root / name for name in ('README.md', 'LICENSE', 'SECURITY.md', 'CHANGELOG.md', 'THIRD_PARTY_NOTICES.md', '.gitignore', 'package.json', 'package-lock.json', 'index.html', 'tsconfig.json', 'vite.config.ts')]
for name in ('src', 'src-tauri', 'assets', 'docs', 'scripts', '.github'):
    files.extend(f for f in (root / name).rglob('*') if f.is_file() and not {'target', 'gen', '__pycache__'}.intersection(f.relative_to(root).parts))
with ZipFile(output, 'w', ZIP_DEFLATED) as archive:
    for file in sorted(set(files)):
        archive.write(file, 'devshield/' + str(file.relative_to(root)))
print(output)
