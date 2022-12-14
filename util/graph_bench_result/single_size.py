"""
Produce bar graph that compares N implementations for a single size.
"""

import json
import sys

from collections import defaultdict

from bokeh import models
from bokeh.plotting import figure, ColumnDataSource, show
from bokeh.resources import CDN
from bokeh.embed import file_html
from bokeh.palettes import Colorblind
from bokeh.models import FactorRange, LabelSet

TRANSFORMS = ["i32", "u64", "string", "1k", "f128"]


def parse_result(path):
    with open(path, "r") as file:
        return json.load(file)


def is_stable_sort(sort_name):
    return "_stable" in sort_name


def extract_groups(bench_result):
    # Result layout:
    # { type (eg. u64):
    #   { prediction_state (eg. hot):
    #     { test_size (eg. 500):
    #       { pattern (eg. descending):
    #         { sort_name (eg. rust_std_stable):
    #            bench_time_ns
    groups = defaultdict(
        lambda: defaultdict(
            lambda: defaultdict(lambda: defaultdict(lambda: {}))
        )
    )

    for benchmark_full, value in bench_result["benchmarks"].items():
        sort_name, _, benchmark = benchmark_full.partition("-")

        entry_parts = benchmark.split("-")

        pred_state = entry_parts[0]
        ty = entry_parts[1]
        pattern = entry_parts[2]
        test_size = int(entry_parts[3])

        if is_stable_sort(sort_name):
            continue  # TODO graph all.

        # TODO Data is botched:
        # if "cpp_std_libcxx" in sort_name:
        #     continue

        if pattern == "random_random_size":
            continue  # TODO I'm not too sure about this one.

        # TODO new patterns
        if pattern == "ascending_saw_5" or pattern == "descending_saw_5":
            continue

        if pattern == "ascending_saw_20":
            pattern = "ascending_saw"

        if pattern == "descending_saw_20":
            pattern = "descending_saw"

        # if "radix" in sort_name:
        #     continue

        bench_time_ns = value["criterion_estimates_v1"]["median"][
            "point_estimate"
        ]

        groups[ty][pred_state][test_size][pattern][sort_name] = bench_time_ns

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
                ("Runtime", "@bench_times"),
            ],
        ),
        models.ResetTool(),
    ]


def add_tools_to_plot(plot):
    plot.add_tools(*TOOLS)

    plot.toolbar.active_scroll = None
    plot.toolbar.active_tap = None
    plot.toolbar.active_drag = TOOLS[1]


def find_time_scale(max_time_ns):
    if max_time_ns < 1_000:
        return 1, "ns"

    if max_time_ns < 1_000_000:
        return 1000, "us"

    if max_time_ns < 1_000_000_000:
        return 1_000_000, "ms"

    raise Exception("time scale not supported")


def format_time(time_val):
    if time_val < 10.0:
        return f"{time_val:.2f}"

    return f"{time_val:.1f}"


def plot_single_size(ty, prediction_state, test_size, values):
    sort_names = sorted(list(list(values.values())[0].keys()))
    pallet_len = max(3, len(sort_names))
    palette = Colorblind[pallet_len]

    def map_sort_to_color(sort_name):
        return palette[sort_names.index(sort_name)]

    max_time_ns = max([max(val.values()) for val in values.values()])
    time_div, time_unit = find_time_scale(max_time_ns)
    max_time = max_time_ns / time_div

    y = []
    bench_times = []
    colors = []
    for pattern, val in reversed(sorted(values.items())):
        for sort_name, bench_times_ns in sorted(
            val.items(), key=lambda x: x[1], reverse=True
        ):
            y.append((pattern, sort_name))
            bench_times.append(bench_times_ns / time_div)
            colors.append(map_sort_to_color(sort_name))

    bench_times_text = [format_time(x) for x in bench_times]

    source = ColumnDataSource(
        data={
            "y": y,
            "bench_times": bench_times,
            "bench_times_text": bench_times_text,
            "colors": colors,
        }
    )

    plot_name = f"{prediction_state}-{ty}-{test_size}"
    plot = figure(
        x_axis_label=f"Time ({time_unit}) | Lower is better",
        x_range=(0, max_time * 1.1),
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
        right="bench_times",
        height=0.8,
        source=source,
        fill_color="colors",
        line_color="black",
    )

    labels = LabelSet(
        x="bench_times",
        y="y",
        text="bench_times_text",
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


def plot_sizes(groups):
    # Assumes all entries were tested for the same patterns.
    for ty, val1 in groups.items():
        for prediction_state, val2 in val1.items():
            for test_size, val3 in val2.items():
                init_tools()

                plot_name, plot = plot_single_size(
                    ty, prediction_state, test_size, val3
                )

                # show(plot)

                html = file_html(plot, CDN, plot_name)
                with open(f"{plot_name}.html", "w+") as outfile:
                    outfile.write(html)

                # raise Exception()


if __name__ == "__main__":
    combined_result = parse_result(sys.argv[1])

    groups = extract_groups(combined_result)

    plot_sizes(groups)
