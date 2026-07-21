#!/usr/bin/env python3
"""Run full EchLub foundation verification sequence."""
from __future__ import annotations

import argparse
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent


def run(label: str, cmd: list[str]) -> bool:
    print(f"\n==> {label}")
    result = subprocess.run(cmd, cwd=ROOT)
    if result.returncode != 0:
        print(f"FAILED: {label}", file=sys.stderr)
        return False
    print(f"OK: {label}")
    return True


def run_expect_fail(label: str, cmd: list[str]) -> bool:
    print(f"\n==> {label} (expect fail)")
    result = subprocess.run(cmd, cwd=ROOT)
    if result.returncode == 0:
        print(f"FAILED: {label} should have failed", file=sys.stderr)
        return False
    print(f"OK: {label} failed as expected")
    return True


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--skip-references", action="store_true")
    args = parser.parse_args()

    steps: list[tuple[str, list[str]]] = []
    if not args.skip_references:
        steps.append(("reference cleanliness (start)", ["python3", "scripts/check-references.py"]))

    steps.extend(
        [
            ("rust fmt", ["cargo", "fmt", "--all", "--", "--check"]),
            (
                "rust clippy",
                [
                    "cargo",
                    "clippy",
                    "--workspace",
                    "--all-targets",
                    "--all-features",
                    "--",
                    "-D",
                    "warnings",
                ],
            ),
            ("rust tests", ["cargo", "test", "--workspace", "--all-features"]),
            (
                "wasm build",
                ["wasm-pack", "build", "crates/echlub-web", "--target", "web", "--out-dir", "pkg"],
            ),
            ("wasm tests", ["wasm-pack", "test", "--node", "crates/echlub-web"]),
            ("web typecheck", ["corepack", "pnpm", "--filter", "@echlub/web", "typecheck"]),
            ("web tests", ["corepack", "pnpm", "--filter", "@echlub/web", "test", "run"]),
            ("web build", ["corepack", "pnpm", "--filter", "@echlub/web", "build"]),
            ("protocol lab", ["cargo", "run", "-p", "echlub-protocol-lab", "--bin", "protocol-lab"]),
            ("control plane tests", ["cargo", "test", "-p", "echlub-control-plane"]),
            ("performance crate tests", ["cargo", "test", "-p", "echlub-performance"]),
            (
                "performance test vectors",
                [
                    "cargo",
                    "run",
                    "-p",
                    "echlub-performance-report",
                    "--",
                    "validate",
                    "test-vectors/performance/harness-validation-v1.json",
                ],
            ),
            (
                "performance assess vector",
                [
                    "cargo",
                    "run",
                    "-p",
                    "echlub-performance-report",
                    "--",
                    "assess-synthetic",
                    "test-vectors/performance/synthetic-media-path-v1.json",
                ],
            ),
            (
                "performance zero-detection negative fixture",
                [
                    "cargo",
                    "run",
                    "-p",
                    "echlub-performance-report",
                    "--",
                    "validate",
                    "test-vectors/performance/synthetic-zero-detection-v1.json",
                ],
            ),
            (
                "live endpoint test vector",
                [
                    "cargo",
                    "run",
                    "-p",
                    "echlub-performance-report",
                    "--",
                    "validate-live-endpoint",
                    "test-vectors/performance/live-endpoint-peer-a-v1.json",
                ],
            ),
        ]
    )

    if not args.skip_references:
        steps.append(("reference cleanliness (end)", ["python3", "scripts/check-references.py"]))

    failed = []
    for label, cmd in steps:
        if label == "performance zero-detection negative fixture":
            ok = run_expect_fail(label, cmd)
        else:
            ok = run(label, cmd)
        if not ok:
            failed.append(label)
    print("\n=== Verification summary ===")
    if failed:
        print("FAILED steps:")
        for step in failed:
            print(f"  - {step}")
        return 1
    print("All verification steps passed.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
