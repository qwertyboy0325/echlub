#!/usr/bin/env python3
"""Fetch shallow reference repositories (stdlib only)."""
from __future__ import annotations

import json
import subprocess
import sys
from datetime import datetime, timezone
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
REFERENCE_DIR = ROOT / ".reference"
LOCK_FILE = ROOT / "docs" / "reference" / "source-lock.json"

REPOS = [
    {
        "repository": "qwertyboy0325/echlub-front",
        "path": "echlub-front",
        "remote_url": "https://github.com/qwertyboy0325/echlub-front.git",
    },
    {
        "repository": "qwertyboy0325/echlub_backend",
        "path": "echlub_backend",
        "remote_url": "https://github.com/qwertyboy0325/echlub_backend.git",
    },
    {
        "repository": "qwertyboy0325/vox-proof",
        "path": "vox-proof",
        "remote_url": "https://github.com/qwertyboy0325/vox-proof.git",
    },
]


def run(cmd: list[str], cwd: Path | None = None) -> str:
    result = subprocess.run(cmd, cwd=cwd, capture_output=True, text=True, check=False)
    if result.returncode != 0:
        raise RuntimeError(f"Command failed: {' '.join(cmd)}\n{result.stderr}")
    return result.stdout.strip()


def clone_or_update(repo: dict) -> dict:
    dest = REFERENCE_DIR / repo["path"]
    REFERENCE_DIR.mkdir(parents=True, exist_ok=True)
    if dest.exists():
        run(["git", "fetch", "origin", "--depth", "1"], cwd=dest)
        run(["git", "checkout", "main"], cwd=dest)
        run(["git", "reset", "--hard", "origin/main"], cwd=dest)
    else:
        gh = run(["which", "gh"], cwd=ROOT) if subprocess.run(["which", "gh"], capture_output=True).returncode == 0 else None
        if gh:
            run(["gh", "repo", "clone", repo["repository"], str(dest), "--", "--depth", "1"], cwd=ROOT)
        else:
            run(["git", "clone", "--depth", "1", repo["remote_url"], str(dest)], cwd=ROOT)

    sha = run(["git", "rev-parse", "HEAD"], cwd=dest)
    branch = run(["git", "rev-parse", "--abbrev-ref", "HEAD"], cwd=dest)
    clean = run(["git", "status", "--porcelain"], cwd=dest) == ""
    return {
        "repository": repo["repository"],
        "remote_url": repo["remote_url"],
        "path": f".reference/{repo['path']}",
        "default_branch": branch,
        "sha": sha,
        "fetched_at": datetime.now(timezone.utc).isoformat(),
        "read_only": True,
        "clean": clean,
    }


def main() -> int:
    entries = [clone_or_update(r) for r in REPOS]
    LOCK_FILE.parent.mkdir(parents=True, exist_ok=True)
    LOCK_FILE.write_text(json.dumps({"references": entries}, indent=2) + "\n", encoding="utf-8")
    print(f"Recorded {len(entries)} references in {LOCK_FILE}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
