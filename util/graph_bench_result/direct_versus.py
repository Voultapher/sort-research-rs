"""
Produce graphs that show the relative speedup and slowdown between two implementations.
"""

import sys
import os
import itertools
import statistics
import math


from bokeh import models
from bokeh.plotting import figure, ColumnDataSource
from bokeh.resources import CDN
from bokeh.embed import file_html
from bokeh.palettes import Colorblind

from natsort import natsorted

from cpu_info import get_cpu_info
from util import parse_result, extract_groups, build_pattern_meta_info

CPU_INFO = None
PATTERN_META_INFO = build_pattern_meta_info()

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
                ("Relative speedup", "@y_adjusted"),
                ("Median speedup", "@y_median"),
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
        return (time_b / time_a) - 1
    else:
        # time_b is faster
        return -(((time_a / time_b) - 1))


def relative_speedup_to_adjusted(rel_speedup):
    return rel_speedup + (1.0 if rel_speedup >= 0.0 else -1.0)


def extract_line(sort_name_a, sort_name_b, pattern, values):
    x = []
    y = []
    for test_len, val in sorted(values.items(), key=lambda x: x[0]):
        if test_len < 1:
            continue

        for pattern_x, val2 in val.items():
            if pattern_x != pattern:
                continue

            bench_time_ns_a = val2[sort_name_a]
            bench_time_ns_b = val2[sort_name_b]
            rel_speedup = relative_speedup(bench_time_ns_a, bench_time_ns_b)

            x.append(test_len)
            y.append(rel_speedup)

    return x, y


def plot_versus(sort_name_a, sort_name_b, ty, prediction_state, values):
    patterns = natsorted(list(values.values())[0].keys())
    min_test_size = min(values.keys())
    max_test_size = max(values.keys())

    plot_name = f"{sort_name_a}-vs-{sort_name_b}-{prediction_state}-{ty}"
    plot = figure(
        title=plot_name,
        x_axis_label="Input length (log)",
        x_axis_type="log",
        y_axis_label=f"Relative symmetric speedup | > 0, a x b | < 0, b x a | {CPU_INFO}",
        y_range=(-2.0, 2.0),
        plot_width=1000,
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

    y_max = 0.0

    for pattern in patterns:
        x, y = extract_line(sort_name_a, sort_name_b, pattern, values)

        y_adjusted = list(map(relative_speedup_to_adjusted, y))
        y_median = round(relative_speedup_to_adjusted(statistics.median(y)), 2)
        if y_median == -1.0:
            y_median = 1.0

        y_max = max(max(map(abs, y)), y_max)

        legend_label = pattern

        color, symbol = PATTERN_META_INFO[pattern]

        data = {
            "x": x,
            "y": y,
            "y_adjusted": y_adjusted,
            "y_median": [y_median] * len(x),
            "name": [pattern] * len(x),
        }
        source = ColumnDataSource(data=data)

        plot.line(
            source=source,
            line_width=1.5,
            color=color,
            legend_label=legend_label,
        )

        getattr(plot, symbol)(
            source=source,
            size=6,
            fill_color=None,
            line_color=color,
            legend_label=legend_label,
        )

    # Anything above 5x is too much.

    y_step_size = max(round(y_max / 10.0, 1), 0.1)
    y_range = math.ceil(y_max / y_step_size) * y_step_size

    # There has to be a better way to do this.
    y_ticker = []
    y_ticker_cur = -y_range
    while y_ticker_cur <= y_range:
        y_ticker.append(y_ticker_cur)
        y_ticker_cur += y_step_size

    plot.yaxis.ticker = y_ticker
    plot.y_range = models.Range1d(start=-y_range, end=y_range)

    format_code_js = """
        const tick_int = parseFloat(tick);
        const adjusted_tick_val = tick_int >= 0.0
            ? tick_int + 1.0
            : tick_int - 1.0;
        return `${adjusted_tick_val.toFixed(1)}x`;
    """
    plot.yaxis.formatter = models.FuncTickFormatter(code=format_code_js)

    return plot_name, plot


def plot_types(sort_name_a, sort_name_b, groups):
    for ty, val1 in groups.items():
        for prediction_state, val2 in val1.items():
            init_tools()

            plot_name, plot = plot_versus(
                sort_name_a, sort_name_b, ty, prediction_state, val2
            )

            html = file_html(plot, CDN, plot_name)
            with open(f"{plot_name}.html", "w+") as outfile:
                outfile.write(html)


if __name__ == "__main__":
    combined_result = parse_result(sys.argv[1])

    groups = extract_groups(combined_result)

    sort_names = list(
        list(
            list(list(list(groups.values())[0].values())[0].values())[
                0
            ].values()
        )[0].keys()
    )

    for sort_name_a, sort_name_b in itertools.product(
        *[sort_names, sort_names]
    ):
        if sort_name_a == sort_name_b:
            continue

        name = os.path.basename(sys.argv[1]).partition(".")[0]
        CPU_INFO = get_cpu_info(name)
        plot_types(sort_name_a, sort_name_b, groups)
