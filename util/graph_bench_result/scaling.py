"""
Produce graphs that show the scaling nature of sort implementations.
"""

import sys
import math


from bokeh import models
from bokeh.plotting import figure, ColumnDataSource
from bokeh.resources import CDN
from bokeh.embed import file_html

from cpu_info import get_cpu_info
from util import (
    parse_result,
    extract_groups,
    build_implementation_meta_info,
    type_size,
    base_name,
    plot_name_suffix,
)

CPU_INFO = None

# Needs to be shared instance :/
TOOLS = None
IMPL_META_INFO = build_implementation_meta_info()


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

    for test_len, val in sorted(values.items(), key=lambda x: x[0]):
        if test_len < 1:
            continue

        for pattern_x, val2 in val.items():
            if pattern_x != pattern:
                continue

            for sort_name_x, bench_time_ns in val2.items():
                if sort_name_x != sort_name:
                    continue

                x.append(test_len)
                input_size_mil = (test_len) / 1e6
                bench_time_s = bench_time_ns / 1e9
                million_elem_per_s = input_size_mil / bench_time_s
                y.append(million_elem_per_s)

    return x, y


def plot_scaling(ty, prediction_state, pattern, values):
    plot_name = (
        f"{prediction_state}-{ty}-scaling-{pattern}{plot_name_suffix()}"
    )
    plot = figure(
        title=plot_name,
        x_axis_label="Input length (log)",
        x_axis_type="log",
        y_axis_label=f"Million elements per second | Higher is better | {CPU_INFO}",
        plot_width=1000,
        plot_height=600,
        tools="",
    )
    add_tools_to_plot(plot)

    plot.add_layout(models.Legend(), "right")

    sort_names = sorted(list(list(values.values())[0].values())[0].keys())

    y_max = 0
    for sort_name in sort_names:
        is_new_sort = sort_name.endswith("_new")

        effective_sort_name = (
            sort_name.partition("_new")[0] if is_new_sort else sort_name
        )
        line_dash = "dashed" if is_new_sort else "solid"

        x, y = extract_line(ty, sort_name, pattern, values)
        color, symbol = IMPL_META_INFO[effective_sort_name]

        y_max = max(y_max, max(y))

        data = {"x": x, "y": y, "name": [sort_name] * len(x)}
        source = ColumnDataSource(data=data)

        plot.line(
            source=source,
            line_width=1.5,
            color=color,
            line_dash=line_dash,
            legend_label=sort_name,
        )

        getattr(plot, symbol)(
            source=source,
            size=6,
            fill_color=None,
            line_color=color,
            legend_label=sort_name,
        )

    y_step_size = max(round(y_max / 15.0, 0), 1.0)
    y_range = math.ceil((y_max * 1.03) / y_step_size) * y_step_size

    # There has to be a better way to do this.
    y_ticker = []
    y_ticker_cur = -y_range
    while y_ticker_cur <= y_range:
        y_ticker.append(y_ticker_cur)
        y_ticker_cur += y_step_size

    plot.yaxis.ticker = y_ticker

    plot.y_range = models.Range1d(start=0, end=y_range)

    return plot_name, plot


def plot_patterns(groups):
    for ty, val1 in groups.items():
        for prediction_state, val2 in val1.items():
            patterns_all = [
                [pattern for pattern in val3.keys()]
                for _test_len, val3 in val2.items()
            ]
            patterns = patterns_all[0]
            # assert all(
            #     p == patterns for p in patterns_all
            # ), f"Expected all patterns for one type-prediction_state {ty}-{prediction_state} combination to be the same, but got: {patterns_all}"

            for pattern in patterns:
                init_tools()

                plot_name, plot = plot_scaling(
                    ty, prediction_state, pattern, val2
                )

                html = file_html(plot, CDN, plot_name)
                with open(f"{plot_name}.html", "w+") as outfile:
                    outfile.write(html)


if __name__ == "__main__":
    combined_result = parse_result(sys.argv[1])

    groups = extract_groups(combined_result)

    name = base_name()
    CPU_INFO = get_cpu_info(name)
    plot_patterns(groups)
