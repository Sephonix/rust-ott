#!/usr/bin/env python3
"""Create a distributable archive with plugin bundles, docs, and dependency licenses."""
import json
from pathlib import Path
import platform
import shutil
import subprocess
import tarfile

root = Path(__file__).resolve().parents[1]
metadata = json.loads(subprocess.check_output(["cargo", "metadata", "--locked", "--format-version", "1"], cwd=root))
target = Path(metadata["target_directory"])
output = root / "dist"
archive_root = output / "sps-ott-linux"
if archive_root.exists():
    shutil.rmtree(archive_root)
archive_root.mkdir(parents=True)
for name in ["SPS OTT.clap", "SPS OTT.vst3", "sps-ott.lv2"]:
    source = target / "bundled" / name
    if source.is_dir():
        shutil.copytree(source, archive_root / name)
    else:
        shutil.copy2(source, archive_root / name)
for name in ["LICENSE", "README.md", "THIRD_PARTY.md"]:
    shutil.copy2(root / name, archive_root / name)
shutil.copytree(root / "docs", archive_root / "docs")
# Cargo metadata locates downloaded source for each locked dependency. Preserve
# actual upstream notices; record SPDX metadata for packages without notice files.
licenses = archive_root / "licenses"
licenses.mkdir()
rows = []
for package in metadata["packages"]:
    directory = licenses / f'{package["name"]}-{package["version"]}-{package["id"].encode().hex()[-12:]}'
    directory.mkdir()
    source = Path(package["manifest_path"]).parent
    notices = []
    for parent in [source, source.parent]:
        for file in parent.iterdir():
            if file.is_file() and file.name.upper().startswith(("LICENSE", "LICENCE", "COPYING", "NOTICE")):
                if file.name not in notices:
                    shutil.copy2(file, directory / file.name)
                    notices.append(file.name)
    record = {key: package.get(key) for key in ["name", "version", "license", "license_file", "repository", "source"]}
    record["notices"] = notices
    (directory / "package.json").write_text(json.dumps(record, indent=2) + "\n")
    rows.append(record)
(licenses / "dependencies.json").write_text(json.dumps(rows, indent=2) + "\n")
revision = subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=root, stderr=subprocess.DEVNULL).decode().strip() if (root / ".git").exists() and subprocess.run(["git", "rev-parse", "--verify", "HEAD"], cwd=root, capture_output=True).returncode == 0 else "uncommitted"
(archive_root / "BUILD.json").write_text(json.dumps({"revision": revision, "source": "https://github.com/Sephonix/rust-ott", "architecture": platform.machine(), "rustc": subprocess.check_output(["rustc", "--version"]).decode().strip()}, indent=2) + "\n")
archive = output / f"sps-ott-linux-{platform.machine()}.tar.gz"
with tarfile.open(archive, "w:gz") as tar:
    tar.add(archive_root, arcname=archive_root.name)
print(archive)
