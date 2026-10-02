#!/usr/bin/env python3
"""Regenerate the real-time jam simulation outputs.

Runs every Taiwan preset across endpoint profiles and topologies, the
latency-tolerance sweeps, and (optionally) a localhost UDP bot run, writing
JSON + Markdown under evidence/realtime-jam-simulation/<label>/.

All inputs are ASSUMED planning profiles. Outputs are simulations, not
measurements, and support no latency claim about real networks or people.
"""
from __future__ import annotations

import argparse
import datetime as dt
import json
import shutil
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
PRESETS = [
    "taiwan-duo",
    "taiwan-trio",
    "taiwan-rock-4",
    "taiwan-rock-4-wired",
    "taiwan-rock-4-mobile",
]
ENDPOINTS = [
    "native-interface",
    "native-builtin",
    "browser-worklet-tuned",
    "browser-webrtc-default",
]
TOPOLOGIES = ["mix", "forward"]


def lab(*args: str) -> None:
    cmd = ["cargo", "run", "--quiet", "--release", "-p", "echlub-jam-lab", "--bin", "jam-lab", "--", *args]
    result = subprocess.run(cmd, cwd=ROOT, stdout=subprocess.DEVNULL)
    if result.returncode != 0:
        sys.exit(f"FAILED: {' '.join(args)}")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--label", default=dt.date.today().isoformat())
    parser.add_argument("--skip-udp", action="store_true", help="skip localhost UDP bot runs")
    args = parser.parse_args()

    out = ROOT / "evidence" / "realtime-jam-simulation" / args.label
    if out.exists():
        shutil.rmtree(out)
    out.mkdir(parents=True)

    rows = [
        "| Scenario | Endpoint | Topology | Mouth-to-ear p50 range (ms) | Mean tempo drift | Tight | Playable+ |",
        "| --- | --- | --- | --- | --- | --- | --- |",
    ]
    for preset in PRESETS:
        for endpoint in ENDPOINTS:
            for topology in TOPOLOGIES:
                d = out / "scenarios" / f"{preset}--{endpoint}--{topology}"
                lab("scenario", "--preset", preset, "--endpoint", endpoint, "--topology", topology, "--out", str(d))
                r = json.loads((d / "scenario.json").read_text())
                lat = [x for row in r["latency_matrix_ms"] for x in row if x > 0]
                rows.append(
                    f"| {preset} | {endpoint} | {topology} | {min(lat):.1f}–{max(lat):.1f} "
                    f"| {r['mean_tempo_drift_pct']:+.2f}% | {r['tight_fraction'] * 100:.0f}% "
                    f"| {r['playable_or_better_fraction'] * 100:.0f}% |"
                )
    banner = (
        "> **SIMULATION.** All network, device, and musician parameters are assumed "
        "planning values, not measurements. No latency claim about real networks or people "
        "follows from this table.\n"
    )
    (out / "matrix.md").write_text(
        "# Taiwan jam scenario matrix\n\n" + banner + "\n" + "\n".join(rows) + "\n"
    )

    lab("sweep", "--max", "80", "--step", "5", "--runs", "30", "--out", str(out / "sweep-naive"))
    lab(
        "sweep", "--max", "80", "--step", "5", "--runs", "30", "--compensation", "0.7",
        "--out", str(out / "sweep-compensation-0.7"),
    )
    for role in ["vocals", "drums"]:
        lab("mobile-study", "--role", role, "--seeds", "10", "--out", str(out / f"mobile-study-{role}"))
    if not args.skip_udp:
        for topology in TOPOLOGIES:
            lab(
                "bots", "--count", "4", "--mode", "probe", "--seconds", "8", "--impair",
                "--topology", topology, "--out", str(out / f"udp-bots-probe-{topology}"),
            )
    print(f"wrote {out.relative_to(ROOT)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
