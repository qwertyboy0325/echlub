#!/usr/bin/env python3
"""Verify reference repositories are clean and match source-lock.json."""
from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
LOCK_FILE = ROOT / "docs" / "reference" / "source-lock.json"


def run(cmd: list[str], cwd: Path) -> tuple[int, str]:
    result = subprocess.run(cmd, cwd=cwd, capture_output=True, text=True, check=False)
    return result.returncode, (result.stdout + result.stderr).strip()


def main() -> int:
    if not LOCK_FILE.exists():
        print(f"Missing lock file: {LOCK_FILE}", file=sys.stderr)
        return 1

    data = json.loads(LOCK_FILE.read_text(encoding="utf-8"))
    ok = True
    for ref in data.get("references", []):
        path = ROOT / ref["path"]
        if not path.exists():
            print(f"Missing reference clone: {path}", file=sys.stderr)
            ok = False
            continue
        code, porcelain = run(["git", "status", "--porcelain"], path)
        if code != 0 or porcelain:
            print(f"Reference not clean: {path}\n{porcelain}", file=sys.stderr)
            ok = False
        code, sha = run(["git", "rev-parse", "HEAD"], path)
        if code != 0 or sha.strip() != ref["sha"]:
            print(
                f"SHA mismatch for {path}: expected {ref['sha']}, got {sha.strip()}",
                file=sys.stderr,
            )
            ok = False
    if ok:
        print("All references clean and match source-lock.json")
    return 0 if ok else 1


if __name__ == "__main__":
    sys.exit(main())
