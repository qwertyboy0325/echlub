#!/usr/bin/env python3
"""Run synthetic WebRTC performance baseline and validate evidence."""
from __future__ import annotations

import argparse
import os
import subprocess
import sys
import time
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
EVIDENCE_DIR = ROOT / "evidence" / "performance-baseline"
PREVIEW_HOST = "localhost"
PREVIEW_PORT = 4173


def run(label: str, cmd: list[str], *, optional: bool = False, env: dict | None = None) -> bool:
    print(f"\n==> {label}")
    merged_env = os.environ.copy()
    if env:
        merged_env.update(env)
    result = subprocess.run(cmd, cwd=ROOT, env=merged_env)
    if result.returncode != 0:
        if optional:
            print(f"SKIPPED (optional): {label}")
            return False
        print(f"FAILED: {label}", file=sys.stderr)
        return False
    print(f"OK: {label}")
    return True


def start_preview() -> subprocess.Popen | None:
    build_ok = subprocess.run(
        ["corepack", "pnpm", "--filter", "@echlub/web", "build"],
        cwd=ROOT,
    )
    if build_ok.returncode != 0:
        return None
    proc = subprocess.Popen(
        [
            "npx",
            "vite",
            "preview",
            "--port",
            str(PREVIEW_PORT),
            "--strictPort",
        ],
        cwd=ROOT / "apps" / "web",
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
    )
    for _ in range(20):
        time.sleep(0.5)
        probe = subprocess.run(
            ["curl", "-sf", f"http://{PREVIEW_HOST}:{PREVIEW_PORT}/"],
            cwd=ROOT,
            stdout=subprocess.DEVNULL,
        )
        if probe.returncode == 0:
            return proc
    proc.terminate()
    return None


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--validate-only", action="store_true")
    args = parser.parse_args()

    if args.validate_only:
        json_files = list(EVIDENCE_DIR.glob("*.json"))
        if not json_files:
            print("No evidence artifacts to validate")
            return 1
        for f in json_files:
            if not run(f"validate {f.name}", [
                "cargo",
                "run",
                "-p",
                "echlub-performance-report",
                "--",
                "validate",
                str(f),
            ]):
                return 1
        return 0

    preview = start_preview()
    if preview is None:
        print("FAILED: web preview unavailable")
        return 1

    try:
        browser_ok = run(
            "synthetic browser harness",
            ["corepack", "pnpm", "--filter", "@echlub/performance-browser-harness", "harness"],
            optional=True,
            env={"ECHLUB_WEB_URL": f"http://{PREVIEW_HOST}:{PREVIEW_PORT}"},
        )
    finally:
        preview.terminate()
        preview.wait(timeout=5)

    json_files = list(EVIDENCE_DIR.glob("synthetic-*.json"))
    if json_files:
        latest = max(json_files, key=lambda p: p.stat().st_mtime)
        run(f"validate {latest.name}", [
            "cargo",
            "run",
            "-p",
            "echlub-performance-report",
            "--",
            "validate",
            str(latest),
        ])
        run(f"summarize {latest.name}", [
            "cargo",
            "run",
            "-p",
            "echlub-performance-report",
            "--",
            "summarize",
            str(latest),
        ])
    elif not browser_ok:
        print("\nPARTIAL: browser harness failed, no synthetic evidence recorded")
        return 2

    return 0


if __name__ == "__main__":
    sys.exit(main())
