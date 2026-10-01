#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.14"
# dependencies = [
#     "plotly",
#     "pydantic",
#     "tyro",
# ]
# ///

# Add './scripts' as a directory where python searches for modules so we can use 'import plot_benches'
if __package__ in (None, ""):
    import sys
    from pathlib import Path
    sys.path.insert(0, str(Path(__file__).resolve().parent.parent))

import tyro
from pathlib import Path
from dataclasses import dataclass, field
from enum import StrEnum
from typing import ClassVar, Annotated

from plot_benches.catalog import Catalog, SeriesSpec
from plot_benches.criterion import DEFAULT_BASELINE, MeasurementCategory, load_series_points
from plot_benches.figure import MAX_MEMBERS_PER_GROUP, MAX_GROUPS, build_figure

@dataclass
class Unit:
    name: str
    factor: float
    """Factor multiplied with the base unit to get this unit"""
    category: MeasurementCategory

    DEFAULT = "us"
    DURATION_UNITS: ClassVar[dict[str, float]] = {
        "ps": 1e3,
        "ns": 1.0,
        "us": 1e-3,
        "µs": 1e-3,
        "ms": 1e-6,
        "s": 1e-9,
    }
    RATE_UNITS: ClassVar[dict[str, tuple[MeasurementCategory, float]]] = {
        "b/s": (MeasurementCategory.SIZE, 1.0), # base size unit
        "B/s": (MeasurementCategory.SIZE, 1/8),
        "elem/s": (MeasurementCategory.ELEMENTS, 1.0)
    }
    PREFIXES: ClassVar[dict[str, float]] = {
        "k": 1e3,
        "M": 1e6,
        "G": 1e9,
        "T": 1e12,
        "Ki": 2**10,
        "Mi": 2**20,
        "Gi": 2**30,
        "Ti": 2**40
    }

    @classmethod
    def parse(cls, raw_unit: str) -> Unit:
        if raw_unit in cls.DURATION_UNITS:
            return Unit(raw_unit, cls.DURATION_UNITS[raw_unit], MeasurementCategory.DURATION)

        suffix = next((ru for ru in cls.RATE_UNITS if raw_unit.endswith(ru)), None)
        if suffix is None:
            raise ValueError(f"unknown unit: {raw_unit!r}")

        category, factor = cls.RATE_UNITS[suffix]
        prefix = raw_unit[: -len(suffix)]
        if prefix:
            if prefix not in cls.PREFIXES:
                raise ValueError(f"unknown prefix: {prefix!r}")
            factor /= cls.PREFIXES[prefix]
        return Unit(raw_unit, factor, category)

@dataclass
class Args:
    series: Annotated[list[str], tyro.conf.Positional] = field(default_factory=list)
    groups: str | None = None
    """Group series to get similar line styles. E.g. '1,2;3,4'"""
    y: str = Unit.DEFAULT
    """Unit of the y-axis. s, b/s, B/s, elem/s, ms, us, MB/s, MiB/s, ..."""
    no_open: bool = False
    """Don't open the plot in a browser"""

def main(args: Args):
    y_unit = Unit.parse(args.y)
    catalog = Catalog.load()

    if not args.series:
        print("Available series:")
        print(catalog.overview())
        return

    selected_series = parse_selected_series(args.series, catalog)
    groups = parse_groups(args.groups, len(selected_series)) if args.groups else None
    series_points = load_series_points(selected_series, y_unit.category)

    # Scale base unit to y_unit
    for _, ys in series_points:
        ys[:] = [y * y_unit.factor for y in ys]

    series_labels = [
        spec.name if baseline == DEFAULT_BASELINE else f"{spec.name}@{baseline}"
        for spec, baseline in selected_series
    ]
    x_unit = selected_series[0][0].x_unit

    for label, (xs, ys) in zip(series_labels, series_points):
        print(f"{label}:, xs: {xs}, ys: {ys}")

    if not args.no_open:
        figure = build_figure(
            series_points, series_labels, y_unit.name, y_unit.category, x_unit, groups
        )
        figure.show()

def parse_groups(groups_str: str, series_count: int) -> list[list[int]]:
    groups: list[list[int]] = []
    for group_str in groups_str.split(";"):
        group = []
        for member_str in group_str.split(","):
            try:
                index = int(member_str)
            except ValueError:
                raise ValueError(
                    f"{member_str!r} is an invalid group member"
                ) from None
            if not 1 <= index <= series_count:
                raise ValueError(
                    f"group member {index} is out of range 1..{series_count}"
                )
            group.append(index - 1)
        if not group:
            raise ValueError(f"empty group: {groups_str!r}")
        groups.append(group)

    if len(groups) > MAX_GROUPS:
        raise ValueError(f"too many groups: {len(groups)} > {MAX_GROUPS}")

    seen_members: set[int] = set()
    for group in groups:
        if len(group) > MAX_MEMBERS_PER_GROUP:
            raise ValueError(
                f"too many group members: len({group}) = {len(group)} > {MAX_MEMBERS_PER_GROUP}"
            )
        for member in group:
            if member in seen_members:
                raise ValueError(
                    f"series {member + 1} is in more than one group"
                )
            seen_members.add(member)

    missing = [str(index + 1) for index in range(series_count) if index not in seen_members]
    if missing:
        raise ValueError(f"series {', '.join(missing)} in no group")

    return groups

def parse_selected_series(raw_selection: list[str], catalog: Catalog) -> list[tuple[SeriesSpec, str]]:
    selection = []
    for s in raw_selection:
        name, _, baseline = s.partition("@")
        baseline = baseline or DEFAULT_BASELINE
        series = catalog.series.get(name)

        if series is None:
            available = ", ".join(catalog.series)
            raise ValueError(f"unknown series: {s!r} (available: {available})")

        selection.append((series, baseline))

    x_units = {series.x_unit for series, _ in selection}
    if len(x_units) > 1:
        raise ValueError(f"series have mismatched x units: {', '.join(sorted(x_units))}")

    return selection

if __name__ == "__main__":
    try:
        main(tyro.cli(Args))
    except (LookupError, TypeError, ValueError) as error:
        sys.exit(f"error: {error}")
