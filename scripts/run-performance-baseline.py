#!/usr/bin/env python3
"""Run synthetic WebRTC performance baseline and validate evidence."""
from __future__ import annotations

import argparse
import json
import os
import subprocess
import sys
import time
from datetime import datetime, timezone
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
EVIDENCE_DIR = ROOT / "evidence" / "performance-baseline"
CORRECTED_DIR = EVIDENCE_DIR / "corrected"
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


def cargo_report_cmd(*args: str) -> list[str]:
    return ["cargo", "run", "-p", "echlub-performance-report", "--", *args]


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


def make_run_id() -> str:
    return datetime.now(timezone.utc).strftime("synthetic-%Y-%m-%dT%H-%M-%S-%fZ")


def write_manifest(run_dir: Path, artifact_name: str, *, validate_ok: bool, assess_ok: bool) -> None:
    manifest = {
        "runId": run_dir.name,
        "artifact": artifact_name,
        "validatePass": validate_ok,
        "assessPass": assess_ok,
        "evidenceStatus": "exploratory_non_authoritative",
        "correctedObservation": True,
    }
    (run_dir / "manifest.json").write_text(json.dumps(manifest, indent=2) + "\n", encoding="utf-8")


def validate_corrected_runs() -> int:
    if not CORRECTED_DIR.is_dir():
        print("No corrected evidence directory")
        return 1

    run_dirs = sorted(p for p in CORRECTED_DIR.iterdir() if p.is_dir())
    if not run_dirs:
        print("No corrected run directories to validate")
        return 1

    for run_dir in run_dirs:
        artifacts = list(run_dir.glob("*.json"))
        artifacts = [p for p in artifacts if p.name != "manifest.json"]
        if not artifacts:
            print(f"No artifact in {run_dir}")
            return 1
        artifact = artifacts[0]
        if not run(f"validate {artifact.name}", cargo_report_cmd("validate", str(artifact))):
            return 1
        if not run(f"assess {artifact.name}", cargo_report_cmd("assess-synthetic", str(artifact))):
            return 1
    return 0


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--validate-only", action="store_true")
    args = parser.parse_args()

    if args.validate_only:
        return validate_corrected_runs()

    run_id = make_run_id()
    run_dir = CORRECTED_DIR / run_id
    run_dir.mkdir(parents=True, exist_ok=True)

    preview = start_preview()
    if preview is None:
        print("FAILED: web preview unavailable")
        return 1

    artifact_path = run_dir / "observation.json"
    try:
        browser_ok = run(
            "synthetic browser harness",
            ["corepack", "pnpm", "--filter", "@echlub/performance-browser-harness", "harness"],
            optional=True,
            env={
                "ECHLUB_WEB_URL": f"http://{PREVIEW_HOST}:{PREVIEW_PORT}",
                "ECHLUB_RUN_ID": run_id,
                "ECHLUB_ARTIFACT_PATH": str(artifact_path),
            },
        )
    finally:
        preview.terminate()
        preview.wait(timeout=5)

    if not artifact_path.is_file():
        if not browser_ok:
            print("\nPARTIAL: browser harness failed, no synthetic evidence recorded")
            return 2
        print("\nFAILED: harness completed without artifact")
        return 2

    validate_ok = run(
        "validate observation",
        cargo_report_cmd("validate", str(artifact_path)),
    )
    assess_ok = run(
        "assess observation",
        cargo_report_cmd("assess-synthetic", str(artifact_path)),
    )
    run(
        "summarize observation",
        cargo_report_cmd("summarize", str(artifact_path)),
        optional=True,
    )
    write_manifest(run_dir, artifact_path.name, validate_ok=validate_ok, assess_ok=assess_ok)

    if not validate_ok or not assess_ok:
        print("\nSYNTHETIC_OBSERVATION_RERUN_REQUIRED: validate or assess failed")
        return 1

    print(f"\nCorrected observation: {artifact_path}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
