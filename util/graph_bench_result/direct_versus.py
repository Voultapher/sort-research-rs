"""
Produce graphs that show the relative speedup and slowdown between two implementations.
"""

import json
import sys
import os
import math

from collections import defaultdict

from bokeh import models
from bokeh.plotting import figure, ColumnDataSource, show
from bokeh.resources import CDN
from bokeh.embed import file_html
from bokeh.palettes import Colorblind
from bokeh.models import FactorRange, LabelSet

from single_size import parse_result, extract_groups
from cpu_info import CPU_BOOST_GHZ, CPU_ARCH

TRANSFORMS = ["i32", "u64", "string", "1k", "f128"]


def map_pattern_to_color(patterns, pattern):
    assert len(patterns) <= 8

    palette = Colorblind[8]

    if pattern == "random":
        # Give random extra visibility.
        return palette[7]

    patterns_filterd = [x for x in patterns if x != "random"]
    return palette[patterns_filterd.index(pattern)]


# Needs to be shared instance :/
TOOLS = None


def init_tools():
    global TOOLS
    TOOLS = [
        models.WheelZoomTool(),
        models.BoxZoomTool(),
        models.PanTool(),
        models.HoverTool(
            tooltips=[
                ("Input Size", "@x"),
                ("Relative speedup", "@y"),
                ("Name", "@name"),
            ],
        ),
        models.ResetTool(),
    ]


def add_tools_to_plot(plot):
    plot.add_tools(*TOOLS)

    plot.toolbar.active_scroll = None
    plot.toolbar.active_tap = None
    plot.toolbar.active_drag = TOOLS[1]


# If time_a is faster than time_b -> % faster than time_b
# If time_b is faster than time_a -> % faster than time_a as negative number
# 100 == time_a 2x faster than time_b
# -100 == time_b 2x faster than time_a
def relative_speedup(time_a, time_b):
    if time_a <= time_b:
        # time_a is faster.
        return ((time_b / time_a) - 1) * 100
    else:
        # time_b is faster
        return -(((time_a / time_b) - 1) * 100)


def extract_line(sort_name_a, sort_name_b, pattern, values):
    x = []
    y = []
    for test_size, val in sorted(values.items(), key=lambda x: x[0]):
        if test_size < 1:
            continue

        for pattern_x, val2 in val.items():
            if pattern_x != pattern:
                continue

            bench_time_ns_a = val2[sort_name_a]
            bench_time_ns_b = val2[sort_name_b]
            x.append(test_size)
            y.append(relative_speedup(bench_time_ns_a, bench_time_ns_b))

    return x, y


def plot_versus(sort_name_a, sort_name_b, ty, prediction_state, values):
    patterns = sorted(list(values.values())[0].keys())
    min_test_size = min(values.keys())
    max_test_size = max(values.keys())

    plot_name = f"{sort_name_a}-vs-{sort_name_b}-{prediction_state}-{ty}"
    plot = figure(
        title=plot_name,
        x_axis_label="Input Size (log)",
        x_axis_type="log",
        y_axis_label=f"a % faster than b | 100% == a 2x b, -100% == b 2x a | {CPU_ARCH}@{CPU_BOOST_GHZ}GHz",
        y_range=(-200.0, 200.0),
        plot_width=800,
        plot_height=600,
        tools="",
    )
    add_tools_to_plot(plot)

    plot.line(
        x=[max(min_test_size, 1), max_test_size],
        y=[0, 0],
        color="black",
        line_alpha=0.4,
    )

    plot.add_layout(models.Legend(), "right")

    for pattern in patterns:
        x, y = extract_line(sort_name_a, sort_name_b, pattern, values)
        color = map_pattern_to_color(patterns, pattern)

        data = {"x": x, "y": y, "name": [pattern] * len(x)}
        source = ColumnDataSource(data=data)

        plot.line(
            source=source,
            line_width=1.5,
            color=color,
            legend_label=pattern,
        )

        plot.square(
            source=source,
            size=5,
            fill_color=None,
            line_color=color,
        )

    return plot_name, plot


def plot_types(sort_name_a, sort_name_b, name, groups):
    for ty, val1 in groups.items():
        for prediction_state, val2 in val1.items():
            init_tools()

            plot_name, plot = plot_versus(
                sort_name_a, sort_name_b, ty, prediction_state, val2
            )

            html = file_html(plot, CDN, plot_name)
            with open(f"{name}-{plot_name}.html", "w+") as outfile:
                outfile.write(html)


if __name__ == "__main__":
    combined_result = parse_result(sys.argv[1])

    groups = extract_groups(combined_result)

    sort_name_a = "c_fluxsort_stable"
    sort_name_b = "rust_glidesort_stable"

    name = os.path.basename(sys.argv[1]).partition(".")[0]
    plot_types(sort_name_a, sort_name_b, name, groups)
