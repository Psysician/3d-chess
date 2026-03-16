#!/usr/bin/env python3
"""Parse cargo-llvm-cov JSON into workspace and per-crate coverage summaries."""

from __future__ import annotations

import json
import os
import sys
from pathlib import Path

CRATES = ("workspace", "chess_core", "chess_persistence", "engine_uci", "game_app")
THRESHOLD_VARS = {
    "workspace": "COVERAGE_WORKSPACE_THRESHOLD",
    "chess_core": "COVERAGE_CHESS_CORE_THRESHOLD",
    "chess_persistence": "COVERAGE_CHESS_PERSISTENCE_THRESHOLD",
    "engine_uci": "COVERAGE_ENGINE_UCI_THRESHOLD",
    "game_app": "COVERAGE_GAME_APP_THRESHOLD",
}


def load_report(report_path: Path) -> dict:
    with report_path.open(encoding="utf-8") as handle:
        payload = json.load(handle)
    data = payload.get("data")
    if isinstance(data, list) and data:
        return data[0]
    return payload


def line_counts(bucket: dict) -> tuple[int, int]:
    lines = bucket.get("lines", {})
    return int(lines.get("covered", 0)), int(lines.get("count", 0))


def crate_for_filename(filename: str) -> str | None:
    normalized = filename.replace("\\", "/")
    for crate in CRATES[1:]:
        marker = f"/crates/{crate}/"
        if marker in normalized or normalized.startswith(f"crates/{crate}/"):
            return crate
    return None


def parse_threshold(name: str) -> float | None:
    raw = os.environ.get(THRESHOLD_VARS[name], "").strip()
    if not raw:
        return None
    try:
        return float(raw)
    except ValueError as exc:
        raise SystemExit(f"invalid threshold for {name}: {raw}") from exc


def percent(covered: int, count: int) -> float:
    if count == 0:
        return 0.0
    return round((covered / count) * 100, 2)


def main() -> None:
    if len(sys.argv) != 2:
        raise SystemExit("usage: parse_coverage.py <cargo-llvm-cov-report.json>")

    report_path = Path(sys.argv[1]).resolve()
    report = load_report(report_path)

    workspace_covered, workspace_count = line_counts(report.get("totals", {}))
    counts: dict[str, list[int]] = {
        "workspace": [workspace_covered, workspace_count],
        "chess_core": [0, 0],
        "chess_persistence": [0, 0],
        "engine_uci": [0, 0],
        "game_app": [0, 0],
    }

    for entry in report.get("files", []):
        crate = crate_for_filename(entry.get("filename", ""))
        if crate is None:
            continue
        covered, count = line_counts(entry.get("summary", {}))
        counts[crate][0] += covered
        counts[crate][1] += count

    mode = os.environ.get("COVERAGE_MODE", "baseline").strip() or "baseline"
    if mode not in {"baseline", "non-regression", "hard-gate"}:
        raise SystemExit(f"unsupported COVERAGE_MODE: {mode}")

    thresholds = {name: parse_threshold(name) for name in CRATES}
    if mode != "baseline":
        missing = [name for name, value in thresholds.items() if value is None]
        if missing:
            names = ", ".join(missing)
            raise SystemExit(
                "missing thresholds for enforced coverage mode: "
                f"{names}. Set the COVERAGE_*_THRESHOLD variables first."
            )

    summary = {}
    violations = []
    for name in CRATES:
        covered, count = counts[name]
        current = percent(covered, count)
        threshold = thresholds[name]
        summary[name] = {
            "covered": covered,
            "count": count,
            "percent": current,
            "threshold": threshold,
        }
        if threshold is not None and current < threshold:
            violations.append(f"{name} {current:.2f}% < {threshold:.2f}%")

    output_dir = report_path.parent
    status = "pass"
    if violations and mode != "baseline":
        status = "fail"

    with (output_dir / "coverage-summary.json").open("w", encoding="utf-8") as handle:
        json.dump(
            {
                "mode": mode,
                "status": status,
                "violations": violations,
                "summary": summary,
            },
            handle,
            indent=2,
        )
        handle.write("\n")

    with (output_dir / "summary.txt").open("w", encoding="utf-8") as handle:
        handle.write(f"mode: {mode}\n")
        for name in CRATES:
            threshold = summary[name]["threshold"]
            threshold_text = "n/a" if threshold is None else f"{threshold:.2f}%"
            handle.write(
                f"{name}: {summary[name]['percent']:.2f}% "
                f"({summary[name]['covered']}/{summary[name]['count']}), "
                f"threshold={threshold_text}\n"
            )
        if violations:
            handle.write("violations:\n")
            for violation in violations:
                handle.write(f"- {violation}\n")

    if violations and mode != "baseline":
        raise SystemExit("coverage threshold failures: " + "; ".join(violations))


if __name__ == "__main__":
    main()
