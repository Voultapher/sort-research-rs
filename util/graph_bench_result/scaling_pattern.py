"""
Produce graphs that show the scaling nature of sort implementations.
Special cases, that don't scale with size as parameter but with pattern.
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
from cpu_info import get_cpu_info

CPU_BOOST_GHZ = None
CPU_ARCH = None

# Adjust for pattern
X_AXIS_LABEL = """% of input that is random, rest is zero"""
# X_AXIS_LABEL = "Number of distintinct values"
# X_AXIS_LABEL = "Zipf distribution exponent"

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
                (X_AXIS_LABEL, "@x"),
                ("Elements per cycle", "@y"),
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


def extract_line(sort_name, test_size, prefix, values):
    def extract_property(pattern):
        return float(pattern.partition(prefix)[2].replace("_", "."))

    x = []
    y = []

    for test_size_x, val in values.items():
        if test_size_x != test_size:
            continue

        for pattern, val2 in sorted(
            val.items(), key=lambda x: extract_property(x[0])
        ):
            for sort_name_x, bench_time_ns in val2.items():
                if sort_name_x != sort_name:
                    continue

                x.append(extract_property(pattern))
                elem_per_ns = test_size / bench_time_ns
                elem_per_cycle = elem_per_ns / CPU_BOOST_GHZ
                y.append(elem_per_cycle)

    return x, y


def plot_scaling(ty, prediction_state, prefix, test_size, values):
    plot_name = f"{prediction_state}-{ty}-{test_size}-scaling-{prefix}"
    plot = figure(
        title=plot_name,
        x_axis_label=f"{X_AXIS_LABEL} (log)",
        x_axis_type="log",
        y_axis_label=f"Elements per cycle (log) | Higher is better | {CPU_ARCH}@{CPU_BOOST_GHZ}GHz",
        y_axis_type="log",
        plot_width=800,
        plot_height=600,
        tools="",
    )
    add_tools_to_plot(plot)

    plot.add_layout(models.Legend(), "right")

    sort_names = sorted(list(list(values.values())[0].values())[0].keys())

    for sort_name in sort_names:
        x, y = extract_line(sort_name, test_size, prefix, values)
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


def plot_patterns(name, groups):
    test_sizes = sorted(list(list(groups.values())[0].values())[0].keys())
    patterns = sorted(
        list(list(list(groups.values())[0].values())[0].values())[0].keys()
    )
    prefix = os.path.commonprefix(patterns)

    for ty, val1 in groups.items():
        for prediction_state, val2 in val1.items():
            for test_size in test_sizes:
                init_tools()

                plot_name, plot = plot_scaling(
                    ty, prediction_state, prefix, test_size, val2
                )

                html = file_html(plot, CDN, plot_name)
                with open(f"{name}-{plot_name}.html", "w+") as outfile:
                    outfile.write(html)


if __name__ == "__main__":
    combined_result = parse_result(sys.argv[1])

    groups = extract_groups(combined_result)

    name = os.path.basename(sys.argv[1]).partition(".")[0]
    CPU_BOOST_GHZ, CPU_ARCH = get_cpu_info(name)
    plot_patterns(name, groups)
