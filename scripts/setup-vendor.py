#!/usr/bin/env python3
import hashlib
import subprocess
import sys
import tomllib
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent


def git(*args, cwd=ROOT, capture=False):
    return subprocess.run(
        ["git", *args], cwd=cwd, check=True,
        stdout=subprocess.PIPE if capture else None,
        text=True, encoding="utf-8",
    ).stdout


def setup(source):
    destination = ROOT / source["path"]
    patch = ROOT / source["patch"]
    patch_bytes = patch.read_bytes()
    if hashlib.sha256(patch_bytes).hexdigest() != source["patch_sha256"]:
        raise RuntimeError(f"patch checksum mismatch: {patch}")
    if not destination.exists():
        destination.mkdir(parents=True)
        git("init", str(destination))
        git("remote", "add", "upstream", source["url"], cwd=destination)
        git("fetch", "--depth", "1", "upstream", source["rev"], cwd=destination)
        git("checkout", "--detach", source["rev"], cwd=destination)
        git("update-ref", "refs/remotes/upstream/main", source["rev"], cwd=destination)
        git("branch", "our/main", source["rev"], cwd=destination)
        git("checkout", "-b", source["branch"], cwd=destination)
    revision = git("rev-parse", "HEAD", cwd=destination, capture=True).strip()
    if revision != source["rev"]:
        raise RuntimeError(f"unexpected revision in {destination}: {revision}")
    existing = git("diff", "HEAD", "--binary", cwd=destination, capture=True).encode()
    untracked = git("ls-files", "--others", "--exclude-standard", cwd=destination, capture=True)
    if untracked:
        raise RuntimeError(f"untracked files in {destination}; preserve or move them before setup")
    if existing == patch_bytes:
        print(f"Verified {source['path']} @ {revision[:12]} + pinned patch")
        return
    if existing:
        raise RuntimeError(f"local changes in {destination}; setup never overwrites them")
    git("apply", "--check", str(patch), cwd=destination)
    git("apply", "--index", str(patch), cwd=destination)
    print(f"Prepared {source['path']} @ {revision[:12]}")


def main():
    sources = tomllib.loads((ROOT / "vendor/sources.toml").read_text())
    for source in sources["source"]:
        setup(source)


if __name__ == "__main__":
    try:
        main()
    except (RuntimeError, subprocess.CalledProcessError) as error:
        print(str(error), file=sys.stderr)
        sys.exit(1)
