#!/usr/bin/env python3
"""Live two-peer observation launcher and verify helper."""
from __future__ import annotations

import argparse
import os
import signal
import subprocess
import sys
import time
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
WEB_PORT = 5173
DEFAULT_BIND = "127.0.0.1:8080"


def run_cmd(cmd: list[str], *, cwd: Path | None = None) -> int:
    return subprocess.run(cmd, cwd=cwd or ROOT).returncode


def start_host(bind: str) -> list[subprocess.Popen]:
    env = os.environ.copy()
    env["ECHLUB_BIND_ADDR"] = bind
    control = subprocess.Popen(
        ["cargo", "run", "-p", "echlub-control-plane", "--bin", "control-plane"],
        cwd=ROOT,
        env=env,
    )
    web = subprocess.Popen(
        ["corepack", "pnpm", "--filter", "@echlub/web", "dev", "--host", "127.0.0.1", "--port", str(WEB_PORT)],
        cwd=ROOT,
    )
    return [control, web]


def terminate(procs: list[subprocess.Popen]) -> None:
    for proc in procs:
        if proc.poll() is None:
            proc.terminate()
    time.sleep(0.5)
    for proc in procs:
        if proc.poll() is None:
            proc.kill()


def host_mode(bind: str) -> int:
    procs = start_host(bind)
    print("Host mode started.")
    print(f"Open http://127.0.0.1:{WEB_PORT} on this device.")
    print("Peer device uses ws://<HOST_LAN_IP>:8080/v1/signaling/ws in the UI.")
    print("Press Ctrl+C to stop.")

    def handle_sig(_signum, _frame):
        terminate(procs)
        sys.exit(0)

    signal.signal(signal.SIGINT, handle_sig)
    signal.signal(signal.SIGTERM, handle_sig)
    try:
        while True:
            time.sleep(1)
            if any(p.poll() is not None for p in procs):
                print("A child process exited unexpectedly.", file=sys.stderr)
                terminate(procs)
                return 1
    except KeyboardInterrupt:
        terminate(procs)
        return 0


def peer_mode() -> int:
    proc = subprocess.Popen(
        ["corepack", "pnpm", "--filter", "@echlub/web", "dev", "--host", "127.0.0.1", "--port", str(WEB_PORT)],
        cwd=ROOT,
    )
    print(f"Peer mode: open http://127.0.0.1:{WEB_PORT}")
    print("Enter host signaling URL in the UI (ws://<HOST_LAN_IP>:8080/v1/signaling/ws).")
    try:
        return proc.wait()
    except KeyboardInterrupt:
        proc.terminate()
        return 0


def verify_mode() -> int:
    steps = [
        (["cargo", "test", "-p", "echlub-control-plane"], "control-plane tests"),
        (["cargo", "test", "-p", "echlub-performance"], "performance tests"),
        (["corepack", "pnpm", "--filter", "@echlub/web", "typecheck"], "web typecheck"),
        (["corepack", "pnpm", "--filter", "@echlub/web", "test", "run"], "web tests"),
    ]
    for cmd, label in steps:
        print(f"\n==> {label}")
        if run_cmd(cmd) != 0:
            print(f"FAILED: {label}", file=sys.stderr)
            return 1
        print(f"OK: {label}")

    vectors = list((ROOT / "test-vectors" / "performance").glob("live-*.json"))
    for vector in vectors:
        print(f"\n==> validate {vector.name}")
        if run_cmd([
            "cargo", "run", "-p", "echlub-performance-report", "--",
            "validate-live-endpoint", str(vector),
        ]) != 0:
            return 1
    print("\nLive readiness verify passed.")
    return 0


def main() -> int:
    parser = argparse.ArgumentParser()
    sub = parser.add_subparsers(dest="mode", required=True)
    host = sub.add_parser("host")
    host.add_argument("--bind", default=DEFAULT_BIND)
    sub.add_parser("peer")
    sub.add_parser("verify")
    args = parser.parse_args()

    if args.mode == "host":
        return host_mode(args.bind)
    if args.mode == "peer":
        return peer_mode()
    return verify_mode()


if __name__ == "__main__":
    sys.exit(main())
