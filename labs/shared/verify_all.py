#!/usr/bin/env python3
"""Smoke-grade all academic labs via sim_cli.

Usage (from repo root):
  python3 labs/shared/verify_all.py
"""

from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
LABS = [
    "01-utxo-and-transfers",
    "02-signatures-and-integrity",
    "03-mempool-and-fees",
    "04-proof-of-work",
    "05-forks-and-reorgs",
    "06-attacks-and-defenses",
]


def run_lab(lab_id: str) -> dict:
    cmd = [
        "cargo",
        "run",
        "--quiet",
        "--bin",
        "sim_cli",
        "--",
        "--profile",
        "classroom",
        "--json",
        "lab",
        "run",
        lab_id,
    ]
    proc = subprocess.run(
        cmd,
        cwd=ROOT,
        capture_output=True,
        text=True,
        check=False,
    )
    if proc.returncode != 0:
        raise RuntimeError(
            f"{lab_id} failed\nstdout:\n{proc.stdout}\nstderr:\n{proc.stderr}"
        )
    # JSON is printed to stdout; ignore cargo noise by finding the last object.
    text = proc.stdout.strip()
    start = text.rfind("{")
    if start < 0:
        raise RuntimeError(f"No JSON object in output for {lab_id}: {text!r}")
    return json.loads(text[start:])


def main() -> int:
    failures = []
    for lab_id in LABS:
        print(f"Running {lab_id} ...", flush=True)
        report = run_lab(lab_id)
        passed = bool(report.get("passed"))
        print(f"  passed={passed}")
        if not passed:
            failures.append(lab_id)

    if failures:
        print("FAILED:", ", ".join(failures))
        return 1

    print("All academic labs passed.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
