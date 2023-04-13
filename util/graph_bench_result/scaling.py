"""
Produce graphs that show the scaling nature of sort implementations.
"""

import sys
import os


from bokeh import models
from bokeh.plotting import figure, ColumnDataSource
from bokeh.resources import CDN
from bokeh.embed import file_html

from cpu_info import get_cpu_info
from util import parse_result, extract_groups, build_color_palette, type_size

CPU_INFO = None

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
                ("Throughput", "@y"),
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


def extract_line(ty, sort_name, pattern, values):
    x = []
    y = []

    # type_size_bytes = type_size(ty)

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
                input_size_mil = (test_size) / 1e6
                bench_time_s = bench_time_ns / 1e9
                million_elem_per_s = input_size_mil / bench_time_s
                y.append(million_elem_per_s)

    return x, y


def plot_scaling(ty, prediction_state, pattern, values):
    plot_name = f"{prediction_state}-{ty}-scaling-{pattern}"
    plot = figure(
        title=plot_name,
        x_axis_label="Input Size (log)",
        x_axis_type="log",
        y_axis_label=f"Million elements per second | Higher is better | {CPU_INFO}",
        plot_width=1000,
        plot_height=600,
        tools="",
    )
    add_tools_to_plot(plot)

    plot.add_layout(models.Legend(), "right")

    sort_names = sorted(list(list(values.values())[0].values())[0].keys())

    max_y_val = 0
    for sort_name in sort_names:
        x, y = extract_line(ty, sort_name, pattern, values)
        color = COLOR_PALETTE[sort_name]

        max_y_val = max(max_y_val, max(y))

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

    if max_y_val < 400:
        step_size = 10 if max_y_val <= 200 else 20
        top_line = 500
        plot.yaxis.ticker = list(range(0, top_line, step_size))

    return plot_name, plot


def plot_patterns(name, groups):
    patterns = sorted(
        list(list(list(groups.values())[0].values())[0].values())[0].keys()
    )

    for ty, val1 in groups.items():
        for prediction_state, val2 in val1.items():
            for pattern in patterns:
                init_tools()

                plot_name, plot = plot_scaling(
                    ty, prediction_state, pattern, val2
                )

                html = file_html(plot, CDN, plot_name)
                with open(f"{name}-{plot_name}.html", "w+") as outfile:
                    outfile.write(html)


if __name__ == "__main__":
    combined_result = parse_result(sys.argv[1])

    groups = extract_groups(combined_result)

    name = os.path.basename(sys.argv[1]).partition(".")[0]
    CPU_INFO = get_cpu_info(name)
    plot_patterns(name, groups)
