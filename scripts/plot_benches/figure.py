import plotly.graph_objects as go
from dataclasses import dataclass
from .criterion import MeasurementCategory

def build_figure(
    series_points: list[tuple[list, list]],
    series_labels: list[str],
    unit_name: str,
    category: MeasurementCategory,
    x_unit: str,
    groups: list[list[int]] | None = None,
) -> go.Figure:
    y_title = (
        f"duration per iteration ({unit_name})"
        if category is MeasurementCategory.DURATION
        else f"throughput ({unit_name})"
    )

    styles = group_styles(groups, len(series_labels)) if groups is not None else None

    figure = go.Figure()
    for index, (label, (xs, ys)) in enumerate(zip(series_labels, series_points)):
        trace = go.Scatter(
            x=xs,
            y=ys,
            name=label,
            mode="lines+markers",
            marker=dict(size=10),
            hovertemplate=(
                f"<b>{label}</b><br>"
                "%{x} " + x_unit + "<br>"
                "%{y:.4g} " + unit_name + "<extra></extra>"
            ),
        )
        if styles is not None:
            style = styles[index]
            trace.line.color = style.color
            trace.line.dash = style.dash
            trace.marker.color = style.color
            trace.marker.symbol = style.symbol
        figure.add_trace(trace)

    figure.update_layout(
        xaxis_title=x_unit,
        yaxis_title=y_title,
        xaxis_minor_ticks="outside",
        yaxis_minor_ticks="outside",
        hovermode="closest",
        legend=dict(itemwidth=45),
    )
    return figure


# (marker, base color, lighter color)
GROUP_STYLES: tuple[tuple[str, str, str], ...] = (
    ("circle", "#0066cc", "#00a0a0"),
    ("square", "#cc0000", "#ff8000"),
    ("triangle-up", "#000000", "#999999"),
)
COLORS_PER_GROUP = 2
DASH_STYLES: tuple[str, ...] = ("solid", "dash")
MAX_GROUPS = len(GROUP_STYLES)
MAX_MEMBERS_PER_GROUP = COLORS_PER_GROUP * len(DASH_STYLES)

@dataclass(frozen=True)
class Style:
    symbol: str
    color: str
    dash: str

def group_styles(groups: list[list[int]], series_count: int) -> list[Style]:
    """Create a list of styles, one for each series."""
    styles: dict[int, Style] = {}
    for group, (symbol, base_color, sub_color) in zip(groups, GROUP_STYLES):
        for member_pos, member in enumerate(group):
            styles[member] = Style(
                symbol=symbol,
                color=sub_color if member_pos >= COLORS_PER_GROUP else base_color,
                dash=DASH_STYLES[member_pos % len(DASH_STYLES)],
            )
    return [styles[index] for index in range(series_count)]
