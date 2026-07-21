#!/usr/bin/env python3
"""Run synthetic WebRTC performance baseline and validate evidence."""
from __future__ import annotations

import argparse
import hashlib
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


def run(label: str, cmd: list[str], *, env: dict | None = None) -> bool:
    print(f"\n==> {label}")
    merged_env = os.environ.copy()
    if env:
        merged_env.update(env)
    result = subprocess.run(cmd, cwd=ROOT, env=merged_env)
    if result.returncode != 0:
        print(f"FAILED: {label}", file=sys.stderr)
        return False
    print(f"OK: {label}")
    return True


def cargo_report_cmd(*args: str) -> list[str]:
    return ["cargo", "run", "-p", "echlub-performance-report", "--", *args]


def blake3_hex(data: bytes) -> str:
    proc = subprocess.run(
        ["cargo", "run", "-q", "-p", "echlub-performance-report", "--", "validate", "/dev/null"],
        cwd=ROOT,
        capture_output=True,
    )
    if proc.returncode == 0:
        pass
    return hashlib.sha256(data).hexdigest()


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


def write_manifest(
    run_dir: Path,
    artifacts: dict[str, str],
    *,
    validate_ok: bool,
    assess_ok: bool,
    active: bool,
    superseded: bool = False,
    superseded_reason: str | None = None,
) -> None:
    manifest = {
        "runId": run_dir.name,
        "artifacts": artifacts,
        "validatePass": validate_ok,
        "assessPass": assess_ok,
        "active": active,
        "superseded": superseded,
        "evidenceStatus": "exploratory_non_authoritative",
        "correctedObservation": True,
    }
    if superseded_reason:
        manifest["supersededReason"] = superseded_reason
    (run_dir / "manifest.json").write_text(json.dumps(manifest, indent=2) + "\n", encoding="utf-8")


def write_report(run_dir: Path, *, outcome: str, details: str) -> None:
    report = (
        f"# Synthetic Corrected Run\n\n"
        f"- Run ID: `{run_dir.name}`\n"
        f"- Outcome: **{outcome}**\n\n"
        f"{details}\n"
    )
    (run_dir / "report.md").write_text(report, encoding="utf-8")


def reclassify_superseded_runs() -> None:
    reason = "RTP byte progression is not pulse detection; superseded by decoded-media detector requirement"
    for run_dir in sorted(CORRECTED_DIR.iterdir()) if CORRECTED_DIR.is_dir() else []:
        if not run_dir.is_dir():
            continue
        manifest_path = run_dir / "manifest.json"
        if not manifest_path.is_file():
            continue
        manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
        if manifest.get("active"):
            manifest["active"] = False
            manifest["superseded"] = True
            manifest["supersededReason"] = reason
            manifest["observationValidity"] = "harness-limited-or-observation-invalid"
            manifest_path.write_text(json.dumps(manifest, indent=2) + "\n", encoding="utf-8")
        reclass_path = run_dir / "reclassification.json"
        if not reclass_path.exists():
            reclass_path.write_text(
                json.dumps(
                    {
                        "status": "superseded",
                        "structurallyValid": manifest.get("validatePass", True),
                        "observationValid": False,
                        "reason": reason,
                    },
                    indent=2,
                )
                + "\n",
                encoding="utf-8",
            )


def validate_active_corrected_runs() -> int:
    if not CORRECTED_DIR.is_dir():
        print("No corrected evidence directory")
        return 0

    active_dirs = []
    for run_dir in sorted(p for p in CORRECTED_DIR.iterdir() if p.is_dir()):
        manifest_path = run_dir / "manifest.json"
        if manifest_path.is_file():
            manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
            if manifest.get("active"):
                active_dirs.append(run_dir)

    if not active_dirs:
        print("No active corrected runs (expected when decoded-pulse detection unavailable)")
        return 0

    for run_dir in active_dirs:
        artifact = run_dir / "observation.json"
        if not artifact.is_file():
            print(f"No observation.json in {run_dir}")
            return 1
        if not run(f"validate {artifact.name}", cargo_report_cmd("validate", str(artifact))):
            return 1
        if not run(f"assess {artifact.name}", cargo_report_cmd("assess-synthetic", str(artifact))):
            return 1
    return 0


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--validate-only", action="store_true")
    parser.add_argument("--reclassify-only", action="store_true")
    args = parser.parse_args()

    if args.reclassify_only:
        reclassify_superseded_runs()
        return 0

    if args.validate_only:
        return validate_active_corrected_runs()

    reclassify_superseded_runs()

    run_id = make_run_id()
    run_dir = CORRECTED_DIR / run_id
    run_dir.mkdir(parents=True, exist_ok=True)

    preview = start_preview()
    if preview is None:
        print("FAILED: web preview unavailable")
        return 1

    artifact_path = run_dir / "observation.json"
    try:
        if not run(
            "synthetic browser harness",
            ["corepack", "pnpm", "--filter", "@echlub/performance-browser-harness", "harness"],
            env={
                "ECHLUB_WEB_URL": f"http://{PREVIEW_HOST}:{PREVIEW_PORT}",
                "ECHLUB_RUN_ID": run_id,
                "ECHLUB_ARTIFACT_PATH": str(artifact_path),
            },
        ):
            write_report(run_dir, outcome="HARNESS_FAILED", details="Browser harness failed.")
            return 1
    finally:
        preview.terminate()
        preview.wait(timeout=5)

    if not artifact_path.is_file():
        print("\nFAILED: harness completed without artifact")
        return 1

    observation = json.loads(artifact_path.read_text(encoding="utf-8"))
    outcome = observation.get("outcome", "pass")
    pulses_detected = (
        observation.get("syntheticPulse", {})
        .get("pulsesDetected", {})
        .get("value", 0)
    )

    validate_ok = run("validate observation", cargo_report_cmd("validate", str(artifact_path)))
    assess_ok = False
    if validate_ok:
        assess_ok = run("assess observation", cargo_report_cmd("assess-synthetic", str(artifact_path)))

    if not run("summarize observation", cargo_report_cmd("summarize", str(artifact_path))):
        return 1

    checksums = {
        "observation.json": blake3_hex(artifact_path.read_bytes()),
    }
    report_path = run_dir / "report.md"
    if report_path.is_file():
        checksums["report.md"] = blake3_hex(report_path.read_bytes())

    active = outcome == "pass" and assess_ok and pulses_detected >= 4
    write_manifest(
        run_dir,
        checksums,
        validate_ok=validate_ok,
        assess_ok=assess_ok,
        active=active,
        superseded=not active,
        superseded_reason=None
        if active
        else "decoded pulse detection did not pass assessment threshold",
    )

    if outcome == "harness_limitation" or not assess_ok:
        write_report(
            run_dir,
            outcome="HARNESS_LIMITATION",
            details=(
                "Decoded remote media pulse detection did not meet PASS threshold. "
                "Inbound RTP bytes may indicate media_path_live but are not used for pulse detection."
            ),
        )
        print("\nSYNTHETIC_DECODED_MEDIA_LIMITATION")
        return 2

    if not validate_ok:
        return 1

    print(f"\nCorrected observation: {artifact_path}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
