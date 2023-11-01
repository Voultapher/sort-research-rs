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

TRANSFORMS = ["i32", "u64", "string", "1k", "f128"]


def is_stable_sort(sort_name):
    return "_stable" in sort_name


def extract_groups(comp_data, stable):
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

        if is_stable_sort(sort_name) ^ stable:
            continue

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


def init_bar_tools():
    global TOOLS
    TOOLS = [
        models.WheelZoomTool(),
        models.BoxZoomTool(),
        models.PanTool(),
        models.HoverTool(
            tooltips=[
                ("Sort", "@y"),
                ("Comparisons", "@comp_counts_full"),
            ],
        ),
        models.ResetTool(),
    ]


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


def make_map_val_to_color(values):
    palette = Colorblind[len(values)]

    def map_sort_to_color(value):
        return palette[values.index(value)]

    return map_sort_to_color


def plot_single_size(ty, test_len, values):
    sort_names = sorted(list(list(values.values())[0].keys()))
    map_sort_to_color = make_map_val_to_color(sort_names)

    max_comp_count = max([max(val.values()) for val in values.values()])
    comp_div = test_len - 1

    y = []
    comp_counts = []
    comp_counts_full = []
    colors = []
    for pattern, val in natsorted(values.items()):
        for sort_name, comp_count in sorted(
            val.items(), key=lambda x: x[1], reverse=True
        ):
            y.append((pattern, sort_name))
            comp_counts.append(comp_count / comp_div)
            comp_counts_full.append(comp_count)
            colors.append(map_sort_to_color(sort_name))

    comp_counts_text = [f"{x:.1f}" for x in comp_counts]

    source = ColumnDataSource(
        data={
            "y": y,
            "comp_counts": comp_counts,
            "comp_counts_text": comp_counts_text,
            "comp_counts_full": comp_counts_full,
            "colors": colors,
        }
    )

    log_n = math.log(test_len)

    plot_name = f"comp-{ty}-{test_len}"
    plot = figure(
        x_axis_label=f"Comparisons performed / (N - 1), log(N) == {log_n:.1f} | Lower is better",
        x_range=(0, max_comp_count / comp_div * 1.1),
        y_range=FactorRange(*y),
        y_axis_label="Pattern",
        title=plot_name,
        tools="",
        plot_width=800,
        plot_height=1000,
    )

    add_tools_to_plot(plot)

    plot.hbar(
        y="y",
        right="comp_counts",
        height=0.8,
        source=source,
        fill_color="colors",
        line_color="black",
    )

    labels = LabelSet(
        x="comp_counts",
        y="y",
        text="comp_counts_text",
        x_offset=5,
        y_offset=-5,
        source=source,
        render_mode="canvas",
        text_font_size="10pt",
    )
    plot.add_layout(labels)

    plot.x_range.start = 0
    plot.ygrid.grid_line_color = None
    plot.y_range.range_padding = 0.02

    return plot_name, plot


def plot_comparisons(groups, stable_name):
    # Assumes all entries were tested for the same patterns.
    for ty, val1 in groups.items():
        for test_len, val2 in val1.items():
            if test_len != 19:
                continue

            init_bar_tools()

            plot_name, plot = plot_single_size(ty, test_len, val2)

            show(plot)

            # html = file_html(plot, CDN, plot_name)
            # with open(f"{plot_name}.html", "w+") as outfile:
            #     outfile.write(html)

            raise Exception()


def plot_comparison_evolution_single(sort_names, groups, sort_name):
    plot = figure(
        title=f"{sort_name}-comp-evolution",
        x_axis_label="Input length (log)",
        x_axis_type="log",
        y_axis_label="Comparisons performed / (N - 1) | Lower is better",
        y_range=(0, 30),  # Chosen to make comparing sorts easier.
        tools="",
    )
    add_tools_to_plot(plot)

    # Only works for a single type for now.
    values = list(groups.values())[0]

    patterns = list(
        sorted(list(values.values())[len(values.values()) - 1].keys())
    )
    map_pattern_to_color = make_map_val_to_color(patterns)

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
        color = map_pattern_to_color(pattern)

        plot.line(
            x="test_sizes",
            y="comp_counts",
            source=source,
            line_width=1.5,
            color=color,
            legend_label=pattern,
        )

        plot.square(
            x="test_sizes",
            y="comp_counts",
            source=source,
            size=5,
            fill_color=None,
            line_color=color,
        )

    plot.legend.location = "top_left"

    return plot


def plot_comparison_evolution(groups, stable_name):
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
    name = os.path.basename(sys.argv[1]).partition(".")[0]
    html = file_html(grid_plot, CDN, stable_name)
    with open(f"{name}-{stable_name}.html", "w+") as outfile:
        outfile.write(html)


def plot_comp(comp_data):
    stable_name = "stable"
    stable = True

    groups = extract_groups(comp_data, stable)
    # plot_comparisons(groups, stable_name)
    plot_comparison_evolution(groups, stable_name)

    stable_name = "unstable"
    stable = False

    groups = extract_groups(comp_data, stable)
    # plot_comparisons(groups, stable_name)
    plot_comparison_evolution(groups, stable_name)


if __name__ == "__main__":
    with open(sys.argv[1], "r") as comp_data_file:
        comp_data = comp_data_file.read()

    plot_comp(comp_data)
