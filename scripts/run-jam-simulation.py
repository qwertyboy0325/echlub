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


def phone_terminals(out: Path) -> None:
    """Phones as terminals: one phone in an otherwise wired band, and all-phone bands."""
    w = "fiber-wired"
    band = f"drums@taipei/{w},bass@taichung/{w},guitar@tainan/{w}"
    cases = [("baseline-all-laptop-interface", f"{band},vocals@kaohsiung/{w}")]
    for e in ["phone-interface", "phone-ios-app", "phone-android-low-latency", "phone-android-generic", "phone-browser"]:
        for label, access in [("wired", w), ("wifi", "wifi"), ("5g-sa", "mobile-5g-sa")]:
            cases.append((f"vocals-{e}-{label}", f"{band},vocals@kaohsiung/{access}/{e}"))
    for e in ["phone-ios-app", "phone-android-low-latency", "phone-android-generic"]:
        for label, access in [("wired", w), ("wifi", "wifi")]:
            players = ",".join(
                f"{role}@{site}/{access}/{e}"
                for role, site in [("drums", "taipei"), ("bass", "taichung"), ("guitar", "tainan"), ("vocals", "kaohsiung")]
            )
            cases.append((f"all4-{e}-{label}", players))
    root = out / "phone-terminals"
    rows = [
        "| Case | Players | Mouth-to-ear p50 range (ms) | Tempo drift | Playable+ | Worst audible dropouts/min |",
        "| --- | --- | --- | --- | --- | --- |",
    ]
    for name, players in cases:
        d = root / name
        lab("scenario", "--players", players, "--topology", "forward", "--redundancy", "2",
            "--coverage", "0.95", "--out", str(d))
        r = json.loads((d / "scenario.json").read_text())
        lat = [x for row in r["latency_matrix_ms"] for x in row if x > 0]
        rows.append(
            f"| {name} | `{players}` | {min(lat):.1f}–{max(lat):.1f} | {r['mean_tempo_drift_pct']:+.2f}% "
            f"| {r['playable_or_better_fraction'] * 100:.0f}% | {r['worst_audible_dropouts_per_min']:.1f} |"
        )
    note = (
        "> **SIMULATION.** Assumed phone audio and network profiles. `wired` for a phone means a "
        "USB-C Ethernet adapter. Forward topology, 2 copies per frame, 95% jitter coverage.\n"
    )
    (root / "matrix.md").write_text("# Phones as terminals\n\n" + note + "\n" + "\n".join(rows) + "\n")


def iphone_target(out: Path) -> None:
    """ADR-0007 target: all iPhone, wired monitoring, network as the variable."""
    e = "fiber-wired"

    def band(access: list[str], device: str = "phone-ios-app") -> str:
        seats = [("drums", "taipei"), ("bass", "taichung"), ("guitar", "tainan"), ("vocals", "kaohsiung")]
        return ",".join(f"{r}@{s}/{a}/{device}" for (r, s), a in zip(seats, access))

    cases = [
        ("duo-same-city-ethernet", f"drums@taichung/{e}/phone-ios-app,bass@taichung/{e}/phone-ios-app"),
        ("duo-north-south-ethernet", f"drums@taipei/{e}/phone-ios-app,bass@kaohsiung/{e}/phone-ios-app"),
        ("duo-north-south-wifi", "drums@taipei/wifi/phone-ios-app,bass@kaohsiung/wifi/phone-ios-app"),
        ("rock4-ethernet-256", band([e] * 4)),
        ("rock4-ethernet-128", band([e] * 4, "phone-ios-app-128")),
        ("rock4-3eth-vocals-wifi", band([e, e, e, "wifi"])),
        ("rock4-3eth-vocals-5g-sa", band([e, e, e, "mobile-5g-sa"])),
        ("rock4-3eth-drums-wifi", band(["wifi", e, e, e])),
        ("rock4-2eth-2wifi", band([e, e, "wifi", "wifi"])),
        ("rock4-all-wifi", band(["wifi"] * 4)),
        ("rock4-all-wifi-tuned-256", band(["wifi-tuned"] * 4)),
        ("rock4-all-wifi-tuned-128", band(["wifi-tuned"] * 4, "phone-ios-app-128")),
        ("duo-north-south-wifi-tuned-128",
         "drums@taipei/wifi-tuned/phone-ios-app-128,bass@kaohsiung/wifi-tuned/phone-ios-app-128"),
        ("rock4-all-wifi-128", band(["wifi"] * 4, "phone-ios-app-128")),
        ("rock4-all-5g-sa", band(["mobile-5g-sa"] * 4)),
        ("rock4-all-5g", band(["mobile-5g"] * 4)),
        ("rock4-all-4g", band(["mobile-4g"] * 4)),
        ("rock4-ethernet-bluetooth", band([e] * 4, "phone-ios-bluetooth")),
    ]
    root = out / "iphone-target"
    rows = [
        "| Case | Mouth-to-ear p50 range (ms) | Tempo drift | Tight | Playable+ | Worst audible dropouts/min |",
        "| --- | --- | --- | --- | --- | --- |",
    ]
    for name, players in cases:
        d = root / name
        lab("scenario", "--players", players, "--topology", "forward", "--redundancy", "2",
            "--coverage", "0.95", "--out", str(d))
        r = json.loads((d / "scenario.json").read_text())
        lat = [x for row in r["latency_matrix_ms"] for x in row if x > 0]
        rows.append(
            f"| {name} | {min(lat):.1f}–{max(lat):.1f} | {r['mean_tempo_drift_pct']:+.2f}% "
            f"| {r['tight_fraction'] * 100:.0f}% | {r['playable_or_better_fraction'] * 100:.0f}% "
            f"| {r['worst_audible_dropouts_per_min']:.1f} |"
        )
    note = (
        "> **SIMULATION.** ADR-0007 condition: all iPhone, native app, wired monitoring "
        "(except the Bluetooth contrast). Assumed profiles; `fiber-wired` = USB-C Ethernet "
        "adapter on a fibre line. Forward topology, 2 copies per frame, 95% coverage.\n"
    )
    (root / "matrix.md").write_text("# All-iPhone target\n\n" + note + "\n" + "\n".join(rows) + "\n")


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
    phone_terminals(out)
    iphone_target(out)
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
