"""
Produce graph that showcases the number of comparisons done by each sort.
"""


import sys
import math

from collections import defaultdict

from bokeh import models
from bokeh.plotting import figure, ColumnDataSource, show
from bokeh.resources import CDN
from bokeh.embed import file_html
from bokeh.palettes import Colorblind
from bokeh.models import FactorRange, LabelSet

TRANSFORMS = ["i32", "u64", "string", "1k", "f128"]


def is_stable_sort(sort_name):
    return "_stable" in sort_name


def extract_groups(comp_data):
    # Result layout:
    # { type (eg. u64):
    #   { test_size (eg. 500):
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

        if is_stable_sort(sort_name):
            continue

        entry_parts = entry.split("-")

        ty = entry_parts[1]
        pattern = entry_parts[2]
        test_size = int(entry_parts[3].partition(":")[0])

        if test_size < 2:
            continue  # These don't make sense and mess up calc

        comp_count = int(entry.rpartition(":")[2].strip())

        groups[ty][test_size][pattern][sort_name] = comp_count

    return groups


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
                ("Sort", "@y"),
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


def plot_single_size(ty, test_size, values):
    sort_names = sorted(list(list(values.values())[0].keys()))
    palette = Colorblind[len(sort_names)]

    def map_sort_to_color(sort_name):
        return palette[sort_names.index(sort_name)]

    max_comp_count = max([max(val.values()) for val in values.values()])
    comp_div = test_size - 1

    y = []
    comp_counts = []
    comp_counts_full = []
    colors = []
    for pattern, val in sorted(values.items()):
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

    log_n = math.log(test_size)

    plot_name = f"comp-{ty}-{test_size}"
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


def plot_comparisons(groups):
    # Assumes all entries were tested for the same patterns.
    for ty, val1 in groups.items():
        for test_size, val2 in val1.items():
            if test_size != 19:
                continue

            init_tools()

            plot_name, plot = plot_single_size(ty, test_size, val2)

            show(plot)

            # html = file_html(plot, CDN, plot_name)
            # with open(f"{plot_name}.html", "w+") as outfile:
            #     outfile.write(html)

            raise Exception()


if __name__ == "__main__":
    with open(sys.argv[1], "r") as comp_data_file:
        comp_data = comp_data_file.read()

    groups = extract_groups(comp_data)

    plot_comparisons(groups)
