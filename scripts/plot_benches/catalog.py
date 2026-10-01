import json
from pathlib import Path
from dataclasses import dataclass
from typing import TypeVar

@dataclass
class Catalog:
    series: dict[str, SeriesSpec]

    @classmethod
    def load(cls) -> Catalog:
        path = Path(__file__).with_name("catalog.json")
        raw_data = json.loads(path.read_text())

        if not isinstance(raw_data, dict):
            raise CatalogError(
                f"{path.name}: expected an object at the top level, got {type(raw_data).__name__}"
            )

        series = {}
        for name in raw_data:
            series[name] = SeriesSpec.parse(name, require_entry(raw_data, name, dict, path.name))

        return Catalog(series)

    def overview(self) -> str:
        result = ""
        width = max(len(name) for name in self.series)
        for name, spec in self.series.items():
            spec_info = f"{len(spec.xs)} x {spec.x_unit}: {[x for x in spec.xs.keys()]}"
            result += f"{name:<{width}}: {spec_info}\n"
        return result

@dataclass
class SeriesSpec:
    name: str
    x_unit: str
    xs: dict[int, str]

    @classmethod
    def parse(cls, name: str, raw_data: dict) -> SeriesSpec:
        where = f"series {name!r}"
        x_unit = require_entry(raw_data, "x_unit", str, where)
        raw_xs = require_entry(raw_data, "xs", dict, where)

        xs: dict[int, str] = {}
        seen: set[str] = set()
        for x, entry in raw_xs.items():
            try:
                key = int(x)
            except ValueError:
                raise CatalogError(f"{where}: 'xs' key {x!r} is not an integer") from None
            if entry in seen:
                raise CatalogError(f"{where}: duplicate entry {entry!r}")
            seen.add(entry)
            xs[key] = entry

        return SeriesSpec(name, x_unit, xs)

class CatalogError(ValueError):
    """catalog.json is missing a field or has the wrong shape."""

T = TypeVar("T")
def require_entry(data: dict, key: str, _type: type[T], where: str) -> T:
    """Fetch `data[field]`, requiring a value of type `typ`. `where` labels the data."""
    try:
        value = data[key]
    except KeyError:
        raise CatalogError(f"{where}: missing required entry {key!r}") from None
    if not isinstance(value, _type):
        raise CatalogError(
            f"{where}: {key!r} must be '{_type.__name__}', got {value}"
        )
    return value
