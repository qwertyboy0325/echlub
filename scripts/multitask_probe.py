#!/usr/bin/env python3
"""Deterministic read-only helper for Cursor Multitask smoke probes."""

from __future__ import annotations

import argparse
import json
import re
import sys
import time
from datetime import datetime, timezone
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
CARGO_TOML = ROOT / "Cargo.toml"
PACKAGE_JSON = ROOT / "package.json"
AGENTS_MD = ROOT / "AGENTS.md"
FOUNDATION_AUDIT = ROOT / "docs" / "quality" / "foundation-audit-report.md"
PERFORMANCE_AUDIT = ROOT / "docs" / "quality" / "performance-audit-report.md"

VALID_MODES = frozenset({"grok", "cargo", "package"})
MIN_SLEEP = 0
MAX_SLEEP = 30


def utc_now_iso() -> str:
    return datetime.now(timezone.utc).isoformat(timespec="microseconds")


def git_head_sha() -> str:
    import subprocess

    result = subprocess.run(
        ["git", "rev-parse", "HEAD"],
        cwd=ROOT,
        check=True,
        capture_output=True,
        text=True,
    )
    return result.stdout.strip()


def bounded_sleep(seconds: float) -> None:
    if seconds < MIN_SLEEP or seconds > MAX_SLEEP:
        raise ValueError(f"sleep seconds must be between {MIN_SLEEP} and {MAX_SLEEP}")
    if seconds > 0:
        time.sleep(seconds)


def parse_workspace_members(text: str) -> list[str]:
    members: list[str] = []
    in_members = False
    for line in text.splitlines():
        stripped = line.strip()
        if stripped.startswith("members"):
            in_members = True
            bracket = stripped.find("[")
            if bracket != -1 and "]" in stripped[bracket:]:
                inner = stripped[bracket + 1 : stripped.index("]", bracket)]
                for item in inner.split(","):
                    cleaned = item.strip().strip('"').strip("'")
                    if cleaned:
                        members.append(cleaned)
                in_members = False
            continue
        if in_members:
            if stripped.startswith("]"):
                in_members = False
                continue
            match = re.match(r'^"([^"]+)"', stripped)
            if match:
                members.append(match.group(1))
    return members


def parse_root_scripts(text: str) -> list[str]:
    scripts: list[str] = []
    in_scripts = False
    for line in text.splitlines():
        stripped = line.strip()
        if stripped.startswith('"scripts"'):
            in_scripts = True
            continue
        if in_scripts:
            if stripped.startswith("}"):
                break
            match = re.match(r'^"([^"]+)"\s*:', stripped)
            if match:
                scripts.append(match.group(1))
    return sorted(scripts)


def run_grok_probe(
    start_timestamp_utc: str,
    end_timestamp_utc: str,
    start_monotonic_ns: int,
    end_monotonic_ns: int,
    head_sha: str,
) -> dict:
    files_inspected: list[str] = []
    for path in (AGENTS_MD, FOUNDATION_AUDIT, PERFORMANCE_AUDIT):
        if path.exists():
            files_inspected.append(str(path.relative_to(ROOT)))
    return {
        "probe": "grok-auditor",
        "start_timestamp_utc": start_timestamp_utc,
        "end_timestamp_utc": end_timestamp_utc,
        "start_monotonic_ns": start_monotonic_ns,
        "end_monotonic_ns": end_monotonic_ns,
        "head_sha": head_sha,
        "files_inspected": files_inspected,
        "result": "PASS",
    }


def run_cargo_probe(
    start_timestamp_utc: str,
    end_timestamp_utc: str,
    start_monotonic_ns: int,
    end_monotonic_ns: int,
    head_sha: str,
) -> dict:
    if not CARGO_TOML.exists():
        raise FileNotFoundError(f"missing {CARGO_TOML}")
    text = CARGO_TOML.read_text(encoding="utf-8")
    members = parse_workspace_members(text)
    return {
        "probe": "cargo-reader",
        "start_timestamp_utc": start_timestamp_utc,
        "end_timestamp_utc": end_timestamp_utc,
        "start_monotonic_ns": start_monotonic_ns,
        "end_monotonic_ns": end_monotonic_ns,
        "head_sha": head_sha,
        "workspace_member_count": len(members),
        "workspace_members": members,
        "files_inspected": [str(CARGO_TOML.relative_to(ROOT))],
        "result": "PASS",
    }


def run_package_probe(
    start_timestamp_utc: str,
    end_timestamp_utc: str,
    start_monotonic_ns: int,
    end_monotonic_ns: int,
    head_sha: str,
) -> dict:
    if not PACKAGE_JSON.exists():
        raise FileNotFoundError(f"missing {PACKAGE_JSON}")
    text = PACKAGE_JSON.read_text(encoding="utf-8")
    scripts = parse_root_scripts(text)
    return {
        "probe": "package-reader",
        "start_timestamp_utc": start_timestamp_utc,
        "end_timestamp_utc": end_timestamp_utc,
        "start_monotonic_ns": start_monotonic_ns,
        "end_monotonic_ns": end_monotonic_ns,
        "head_sha": head_sha,
        "root_script_names": scripts,
        "files_inspected": [str(PACKAGE_JSON.relative_to(ROOT))],
        "result": "PASS",
    }


def main() -> int:
    parser = argparse.ArgumentParser(description="Deterministic Cursor Multitask smoke probe helper")
    parser.add_argument("--mode", required=True, choices=sorted(VALID_MODES))
    parser.add_argument("--sleep-seconds", type=float, default=0)
    args = parser.parse_args()

    try:
        bounded_sleep(args.sleep_seconds)
    except ValueError as exc:
        print(str(exc), file=sys.stderr)
        return 2

    start_timestamp_utc = utc_now_iso()
    start_monotonic_ns = time.monotonic_ns()
    head_sha = git_head_sha()
    if args.sleep_seconds > 0:
        time.sleep(args.sleep_seconds)
    end_monotonic_ns = time.monotonic_ns()
    end_timestamp_utc = utc_now_iso()

    if args.mode == "grok":
        payload = run_grok_probe(
            start_timestamp_utc,
            end_timestamp_utc,
            start_monotonic_ns,
            end_monotonic_ns,
            head_sha,
        )
    elif args.mode == "cargo":
        payload = run_cargo_probe(
            start_timestamp_utc,
            end_timestamp_utc,
            start_monotonic_ns,
            end_monotonic_ns,
            head_sha,
        )
    else:
        payload = run_package_probe(
            start_timestamp_utc,
            end_timestamp_utc,
            start_monotonic_ns,
            end_monotonic_ns,
            head_sha,
        )

    print(json.dumps(payload, indent=2, sort_keys=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
