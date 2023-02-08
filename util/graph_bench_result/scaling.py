"""
Produce graphs that show the scaling nature of sort implementations.
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

from single_size import parse_result, extract_groups, build_color_palette

TRANSFORMS = ["i32", "u64", "string", "1k", "f128"]

# Adjust for machine
CPU_BOOST_GHZ = 4.9
CPU_ARCH = "Zen3"


# Needs to be shared instance :/
TOOLS = None
COLOR_PALETTE = build_color_palette()


def init_tools():
    global TOOLS
    TOOLS = [
        models.WheelZoomTool(),
        models.BoxZoomTool(),
        models.PanTool(),
        models.HoverTool(
            tooltips=[
                ("Input Size", "@x"),
                ("elements per cycle", "@y"),
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


def extract_line(sort_name, pattern, values):
    x = []
    y = []
    for test_size, val in sorted(values.items(), key=lambda x: x[0]):
        if test_size < 1:
            continue

        for pattern_x, val2 in val.items():
            if pattern_x != pattern:
                continue

            for sort_name_x, bench_time_ns in val2.items():
                if sort_name_x != sort_name:
                    continue

                x.append(test_size)
                elem_per_ns = test_size / bench_time_ns
                elem_per_cycle = elem_per_ns / CPU_BOOST_GHZ
                y.append(elem_per_cycle)

    return x, y


def plot_scaling(ty, prediction_state, pattern, values):
    plot_name = f"{prediction_state}-{ty}-scaling-{pattern}"
    plot = figure(
        title=plot_name,
        x_axis_label="Input Size (log)",
        x_axis_type="log",
        y_axis_label=f"elements per cycle (log) | Higher is better | {CPU_ARCH}@{CPU_BOOST_GHZ}GHz",
        y_axis_type="log",
        plot_width=800,
        plot_height=600,
        tools="",
    )
    add_tools_to_plot(plot)

    plot.add_layout(models.Legend(), "right")

    sort_names = sorted(list(list(values.values())[0].values())[0].keys())

    for sort_name in sort_names:
        x, y = extract_line(sort_name, pattern, values)
        color = COLOR_PALETTE[sort_name]

        data = {"x": x, "y": y, "name": [sort_name] * len(x)}
        source = ColumnDataSource(data=data)

        plot.line(
            source=source,
            line_width=1.5,
            color=color,
            legend_label=sort_name,
        )

        plot.square(
            source=source,
            size=5,
            fill_color=None,
            line_color=color,
        )

    return plot_name, plot


def plog_patterns(name, groups):
    patterns = sorted(
        list(list(list(groups.values())[0].values())[0].values())[0].keys()
    )

    for ty, val1 in groups.items():
        for prediction_state, val2 in val1.items():
            for pattern in patterns:
                init_tools()

                plot_name, plot = plot_scaling(ty, prediction_state, pattern, val2)


                html = file_html(plot, CDN, plot_name)
                with open(f"{name}-{plot_name}.html", "w+") as outfile:
                    outfile.write(html)


if __name__ == "__main__":
    combined_result = parse_result(sys.argv[1])

    groups = extract_groups(combined_result)

    name = os.path.basename(sys.argv[1]).partition(".")[0]
    plog_patterns(name, groups)
