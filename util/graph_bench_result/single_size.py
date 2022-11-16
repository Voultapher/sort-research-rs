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
from bokeh.transform import dodge
from bokeh.palettes import Colorblind

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

        if not is_stable_sort(sort_name):
            continue  # TODO graph all.

        # TODO Data is botched:
        if "cpp_std_libcxx" in sort_name:
            continue

        if pattern == "random_random_size":
            continue  # TODO I'm not too sure about this one.

        bench_time_ns = value["criterion_estimates_v1"]["median"][
            "point_estimate"
        ]

        groups[ty][pred_state][test_size][pattern][sort_name] = bench_time_ns

    return groups


# Needs to be shared instance :/
TOOLS = None


def init_tools_overview():
    global TOOLS
    TOOLS = [
        models.WheelZoomTool(),
        models.BoxZoomTool(),
        models.PanTool(),
        models.HoverTool(
            tooltips=[  # TODO
                ("Name", "@name"),
                ("Test Size", "@x"),
                ("Relative speedup", "@y%"),
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


def plot_single_size(ty, prediction_state, test_size, values):
    patterns = list(values.keys())
    sort_names = sorted(list(list(values.values())[0].keys()))

    max_time_ns = max([max(val.values()) for val in values.values()])
    time_div, time_unit = find_time_scale(max_time_ns)
    max_time = max_time_ns / time_div

    data = {"patterns": patterns}
    for pattern, val in sorted(values.items()):
        for sort_name, bench_times_ns in sorted(val.items()):
            data.setdefault(sort_name, []).append(bench_times_ns / time_div)

    source = ColumnDataSource(data)

    plot_name = f"{prediction_state}-{ty}-{test_size}"
    plot = figure(
        x_range=patterns,
        x_axis_label="Pattern",
        y_axis_label=f"Time ({time_unit})",
        y_range=(0, max_time * 1.15),
        title=plot_name,
        tools="",
        plot_width=1400,
    )

    add_tools_to_plot(plot)

    sn_len = len(sort_names)
    step_size = 0.8 / sn_len

    def offsets():
        offset = -(int(sn_len / 2) * step_size)

        while True:
            yield offset
            offset += step_size

    colors = Colorblind[len(sort_names)]

    for sort_name, offset, color in zip(sort_names, offsets(), colors):
        plot.vbar(
            x=dodge("patterns", offset, range=plot.x_range),
            top=sort_name,
            source=source,
            width=step_size * 0.8,
            color=color,
            legend_label=sort_name,
        )

    plot.xgrid.grid_line_color = None
    plot.legend.location = "top_left"
    plot.legend.orientation = "horizontal"

    return plot_name, plot


def plot_sizes(groups):
    init_tools_overview()

    # Assumes all entries were tested for the same patterns.
    for ty, val1 in groups.items():
        for prediction_state, val2 in val1.items():
            for test_size, val3 in val2.items():
                plot_name, plot = plot_single_size(
                    ty, prediction_state, test_size, val3
                )

                show(plot)

                # html = file_html(plot, CDN, plot_name)
                # with open(f"{plot_name}.html", "w+") as outfile:
                #     outfile.write(html)

                raise Exception()


if __name__ == "__main__":
    combined_result = parse_result(sys.argv[1])

    groups = extract_groups(combined_result)

    plot_sizes(groups)
