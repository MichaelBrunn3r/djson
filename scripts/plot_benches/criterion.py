import json
import os
from enum import StrEnum
from pathlib import Path
from .catalog import SeriesSpec

DEFAULT_BASELINE = "base"

class MeasurementCategory(StrEnum):
    DURATION = "duration"
    SIZE = "size"
    ELEMENTS = "elements"

def load_series_points(series: list[tuple[SeriesSpec, str]], measurement: MeasurementCategory) -> list[tuple[list, list]]:
    """Load `(xs, ys)` per series, with the ys in categories the base unit."""
    root = default_output_dir()
    resolved = []
    for (spec, baseline) in series:
        xs = []
        ys = []
        for x, bench_id in spec.xs.items():
            bench_dir = resolve_bench_dir(root, bench_id, baseline)
            xs.append(x)
            ys.append(load_value(measurement, bench_dir))
        resolved.append((xs, ys))
    return resolved

def load_value(category: MeasurementCategory, bench_dir: Path) -> float:
    """Load a benchmark's value for the expected measurement category.
    The value is in the category's base unit: nanoseconds, bits/s, or elements/s."""

    estimates = json.loads((bench_dir / "estimates.json").read_text())
    ns = (estimates.get("slope") or estimates["mean"])["point_estimate"]

    # Expecting a duration in ns
    if category is MeasurementCategory.DURATION:
        return ns

    # Expecting a rate in elements/s or bits/s
    raw = json.loads((bench_dir / "benchmark.json").read_text())
    ((tag, raw_count),) = raw["throughput"].items()

    count: float = 0
    try:
        match category, tag:
            case MeasurementCategory.SIZE, "Bits":
                count = float(raw_count)
            case MeasurementCategory.SIZE, "Bytes" | "BytesDecimal":
                count = float(raw_count) * 8
            case MeasurementCategory.ELEMENTS, "ElementsAndBytes":
                count = float(raw_count["elements"])
            case MeasurementCategory.ELEMENTS, "Elements":
                count = float(raw_count)
            case _:
                raise ValueError(f"{tag!r} does not count {category}")
    except (TypeError, KeyError) as error:
        raise ValueError(f"{tag!r} has no usable count: {error}") from error

    return count / (ns * 1e-9)

def resolve_bench_dir(root: Path, bench_id: str, baseline: str) -> Path:
    # Replace all '/' (except the last one) with '_'
    group, _, name = bench_id.rpartition("/")
    return root / group.replace("/", "_") / name / baseline

def default_output_dir() -> Path:
    target = os.environ.get("CARGO_TARGET_DIR") or "target"
    return Path(target) / "criterion"
