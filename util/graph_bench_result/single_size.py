"""
Produce bar graph that compares N implementations for a single size.
"""

import sys


from bokeh import models
from bokeh.plotting import figure, ColumnDataSource
from bokeh.resources import CDN
from bokeh.embed import file_html
from bokeh.models import FactorRange, LabelSet

from natsort import natsorted

from cpu_info import get_cpu_info
from util import (
    parse_bench_results,
    build_implementation_meta_info,
    base_name,
)

CPU_INFO = None


def find_time_scale(max_time_ns):
    if max_time_ns < 10_000:
        return 1, "ns"

    if max_time_ns < 10_000_000:
        return 1000, "us"

    if max_time_ns < 10_000_000_000:
        return 1_000_000, "ms"

    if max_time_ns < 10_000_000_000_000:
        return 1_000_000_000, "s"

    raise Exception("time scale not supported")


def format_time(time_val):
    if time_val < 10.0:
        return f"{time_val:.2f}"

    return f"{time_val:.1f}"


IMPL_META_INFO = build_implementation_meta_info()


def plot_single_size(ty, prediction_state, test_len, values):
    max_time_ns = max([max(val.values()) for val in values.values()])
    time_div, time_unit = find_time_scale(max_time_ns)
    max_time = max_time_ns / time_div

    y = []
    bench_times = []
    colors = []
    hatch_pattern = []
    fill_alpha = []
    sort_names = set()

    for pattern, val in reversed(natsorted(values.items())):
        for sort_name, bench_times_ns in sorted(
            val.items(), key=lambda x: x[1], reverse=True
        ):
            sort_names.add(sort_name)

            is_new_sort = sort_name.endswith("_new")
            effective_sort_name = (
                sort_name.partition("_new")[0] if is_new_sort else sort_name
            )
            hatch_pattern.append("/" if is_new_sort else None)
            fill_alpha.append(0.8 if is_new_sort else 1.0)

            y.append((pattern, sort_name))
            bench_times.append(bench_times_ns / time_div)
            color, _shape = IMPL_META_INFO[effective_sort_name]
            colors.append(color)

    bench_times_text = [format_time(x) for x in bench_times]

    source = ColumnDataSource(
        data={
            "y": y,
            "bench_times": bench_times,
            "bench_times_text": bench_times_text,
            "colors": colors,
            "hatch_pattern": hatch_pattern,
            "fill_alpha": fill_alpha,
        }
    )

    # Dependent on the number of sort implementations.
    plot_height_extra = min(max(0, len(sort_names) - 2), 3) * 100

    plot_name = f"{prediction_state}-{ty}-{test_len}"
    plot = figure(
        x_axis_label=f"Time ({time_unit}) | Lower is better | {CPU_INFO}",
        x_range=(0, max_time * 1.1),
        y_range=FactorRange(*y),
        y_axis_label="Pattern",
        title=plot_name,
        tools="",
        width=800,
        height=600 + plot_height_extra,
    )

    plot.hbar(
        y="y",
        right="bench_times",
        height=0.8,
        source=source,
        fill_color="colors",
        line_color="black",
        hatch_pattern="hatch_pattern",
        fill_alpha="fill_alpha",
    )

    labels = LabelSet(
        x="bench_times",
        y="y",
        text="bench_times_text",
        x_offset=5,
        y_offset=-5,
        source=source,
        # renderers="canvas",
        text_font_size="10pt",
    )
    plot.add_layout(labels)

    plot.toolbar.logo = None

    plot.x_range.start = 0
    plot.ygrid.grid_line_color = None
    plot.y_range.range_padding = 0.02

    return plot_name, plot


def plot_sizes(groups):
    # Assumes all entries were tested for the same patterns.
    for ty, val1 in groups.items():
        for prediction_state, val2 in val1.items():
            for test_len, val3 in val2.items():
                plot_name, plot = plot_single_size(ty, prediction_state, test_len, val3)

                html = file_html(plot, CDN, plot_name)
                with open(f"{plot_name}.html", "w+") as outfile:
                    outfile.write(html)


if __name__ == "__main__":
    groups = parse_bench_results(sys.argv[1:])

    name = base_name()
    CPU_INFO = get_cpu_info(name)
    plot_sizes(groups)
