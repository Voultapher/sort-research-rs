"""
Produce graph that showcases the number of comparisons done by each sort.
"""


import sys
import math
import os

from collections import defaultdict

from bokeh import models
from bokeh.plotting import figure, ColumnDataSource, show, gridplot
from bokeh.resources import CDN
from bokeh.embed import file_html
from bokeh.palettes import Colorblind
from bokeh.models import FactorRange, LabelSet

from natsort import natsorted

from util import base_name, build_pattern_meta_info


PATTERN_META_INFO = build_pattern_meta_info()


def extract_groups(comp_data):
    # Result layout:
    # { type (eg. u64):
    #   { test_len (eg. 500):
    #     { pattern (eg. descending):
    #       { sort_name (eg. rust_std_stable):
    #          comp_count
    groups = defaultdict(
        lambda: defaultdict(
            lambda: defaultdict(lambda: defaultdict(lambda: {}))
        )
    )

    for line in comp_data.splitlines():
        if ":" not in line:
            continue

        sort_name, _, entry = line.partition("-")

        entry_parts = entry.split("-")

        ty = entry_parts[1]
        pattern = entry_parts[2]
        test_len = int(entry_parts[3].partition(":")[0])

        if test_len < 2:
            continue  # These don't make sense and mess up calc

        comp_count = int(entry.rpartition(":")[2].strip())

        groups[ty][test_len][pattern][sort_name] = comp_count

    return groups


# Needs to be shared instance :/
TOOLS = None


def init_evolution_tools():
    global TOOLS
    TOOLS = [
        models.WheelZoomTool(),
        models.BoxZoomTool(),
        models.PanTool(),
        models.HoverTool(
            tooltips=[
                ("Pattern", "@patterns"),
                ("Test Size", "@test_sizes"),
                ("Comparisons / (N - 1)", "@comp_counts"),
                ("Comparisons", "@comp_counts_full"),
            ],
        ),
        models.ResetTool(),
    ]


def add_tools_to_plot(plot):
    plot.add_tools(*TOOLS)

    plot.toolbar.active_scroll = None
    plot.toolbar.active_tap = None
    plot.toolbar.active_drag = TOOLS[1]


def plot_comparison_evolution_single(sort_names, groups, sort_name):
    plot = figure(
        title=f"{sort_name}-comp-evolution",
        x_axis_label="Input length (log)",
        x_axis_type="log",
        y_axis_label="Mean comparisons performed / (N - 1) | Lower is better",
        y_range=(0, 30),  # Chosen to make comparing sorts easier.
        tools="",
    )
    add_tools_to_plot(plot)

    # Only works for a single type for now.
    values = list(groups.values())[0]

    patterns = list(
        sorted(list(values.values())[len(values.values()) - 1].keys())
    )

    pattern_comp_counts = {}
    for test_len, val1 in sorted(values.items()):
        for pattern, val2 in val1.items():
            for sort_name_x, comp_count in val2.items():
                if sort_name_x != sort_name:
                    continue

                pattern_comp_counts.setdefault(pattern, {}).setdefault(
                    "test_sizes", []
                ).append(test_len)

                pattern_comp_counts[pattern].setdefault(
                    "comp_counts", []
                ).append(comp_count / (test_len - 1))

                pattern_comp_counts[pattern].setdefault(
                    "comp_counts_full", []
                ).append(comp_count)

                pattern_comp_counts[pattern].setdefault("patterns", []).append(
                    pattern
                )

    for pattern, data in natsorted(pattern_comp_counts.items()):
        source = ColumnDataSource(data=data)
        color, symbol = PATTERN_META_INFO[pattern]

        plot.line(
            x="test_sizes",
            y="comp_counts",
            source=source,
            line_width=1.5,
            color=color,
            legend_label=pattern,
        )

        getattr(plot, symbol)(
            x="test_sizes",
            y="comp_counts",
            source=source,
            size=6,
            fill_color=None,
            line_color=color,
            legend_label=pattern,
        )

    plot.legend.location = "top_left"

    return plot


def plot_comparison_evolution(groups):
    # Assumes all entries were tested for the same patterns.
    sort_names = sorted(
        list(
            list(list(list(groups.values())[0].values())[0].values())[0].keys()
        )
    )

    init_evolution_tools()

    plots = [
        plot_comparison_evolution_single(sort_names, groups, sort_name)
        for sort_name in sort_names
    ]

    grid_plot = gridplot(plots, ncols=2)
    name = base_name()
    html = file_html(grid_plot, CDN)
    with open(f"{name}.html", "w+") as outfile:
        outfile.write(html)


def plot_comp(comp_data):
    groups = extract_groups(comp_data)
    plot_comparison_evolution(groups)


if __name__ == "__main__":
    with open(sys.argv[1], "r") as comp_data_file:
        comp_data = comp_data_file.read()

    plot_comp(comp_data)
