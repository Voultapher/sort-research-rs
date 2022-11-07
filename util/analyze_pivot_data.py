# The goal is to find a good way to skew small size pivots to fit the
# sorting-network sizes.

import sys

from collections import defaultdict

from bokeh.plotting import figure, show
from bokeh.models import BoxZoomTool

TEST_SIZE = 10_000


def draw_dist_plot(p, len_dist_mean_sorted, max_small_sort, name, color):
    x = [len for len, frequency in len_dist_mean_sorted]
    y = [frequency for len, frequency in len_dist_mean_sorted]

    p.square(x, y, fill_color=None, line_color=color)
    p.line(x, y, line_color=color, legend_label=name)

    y2 = [
        frequency if len <= max_small_sort else 0
        for len, frequency in len_dist_mean_sorted
    ]
    p.varea(
        x,
        y1=0,
        y2=y2,
        alpha=0.3,
        fill_color=color,
        legend_label=f"Handled by dedicated small sort len <= {max_small_sort}",
    )


def analyze_pivot_data(text):
    len_data = list(
        filter(lambda line: line.startswith("len: "), text.split("\n"))
    )

    print(len(len_data))

    len_dists = []

    # Assumes the data is ordered such that the top size shows up, the the
    # sub-sizes and then again a new top size.
    for line in len_data:
        sort_len = int(line.partition(":")[2].partition(",")[0])
        if sort_len == TEST_SIZE:
            len_dists.append(defaultdict(lambda: 0))

        len_dists[len(len_dists) - 1][sort_len] += 1

    len_dists_sorted = [sorted(len_dist.items()) for len_dist in len_dists]

    # Ok this is more complicated than just summing everything but I could see
    # this more detailed information be helpful for future analysis.
    len_dist_aggregated = defaultdict(lambda: [])

    for len_dist in len_dists_sorted:
        for length, frequency in len_dist:
            len_dist_aggregated[length].append(frequency)

    len_dist_mean = {}
    for length, frequencies in len_dist_aggregated.items():
        len_dist_mean[length] = sum(frequencies) / len(len_dists_sorted)

    len_dist_mean_sorted = sorted(len_dist_mean.items())

    return len_dist_mean_sorted


def graph_pivot_data(text_a, text_b):
    p = figure(
        title=f"How often will recurse be called with a length when sorting {TEST_SIZE} random elements",
        x_axis_label="v.len() in function recurse (log)",
        x_axis_type="log",
        y_axis_label="Average times called",
        tools="pan,wheel_zoom,box_zoom,reset,hover",
    )

    len_dist_mean_sorted_a = analyze_pivot_data(text_a)
    len_dist_mean_sorted_b = analyze_pivot_data(text_b)

    draw_dist_plot(
        p, len_dist_mean_sorted_a, 20, "rust_std_unstable", color="green"
    )
    draw_dist_plot(
        p, len_dist_mean_sorted_b, 40, "rust_new_unstable", color="orange"
    )

    p.toolbar.active_drag = BoxZoomTool()

    show(p)


if __name__ == "__main__":
    with open(sys.argv[1], "r") as file:
        text_a = file.read()

    with open(sys.argv[2], "r") as file:
        text_b = file.read()

    graph_pivot_data(text_a, text_b)
