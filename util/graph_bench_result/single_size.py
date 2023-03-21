"""
Produce bar graph that compares N implementations for a single size.
"""

import sys
import os


from bokeh import models
from bokeh.plotting import figure, ColumnDataSource, show
from bokeh.resources import CDN
from bokeh.embed import file_html
from bokeh.models import FactorRange, LabelSet

from cpu_info import get_cpu_info
from util import parse_result, extract_groups, build_color_palette

CPU_BOOST_GHZ = None
CPU_ARCH = None


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


COLOR_PALETTE = build_color_palette()


def plot_single_size(ty, prediction_state, test_size, values):
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
            colors.append(COLOR_PALETTE[sort_name])

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
        x_axis_label=f"Time ({time_unit}) | Lower is better | {CPU_ARCH} max {CPU_BOOST_GHZ}GHz",
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


def plot_sizes(name, groups):
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
                with open(f"{name}-{plot_name}.html", "w+") as outfile:
                    outfile.write(html)

                # raise Exception()


if __name__ == "__main__":
    combined_result = parse_result(sys.argv[1])

    groups = extract_groups(combined_result)

    name = os.path.basename(sys.argv[1]).partition(".")[0]
    CPU_BOOST_GHZ, CPU_ARCH = get_cpu_info(name)
    plot_sizes(name, groups)
