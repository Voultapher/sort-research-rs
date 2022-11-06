# The goal is to find a good way to skew small size pivots to fit the
# sorting-network sizes.

import sys

from collections import defaultdict

from bokeh.plotting import figure, show


TEST_SIZE = 10_000
MAX_SMALL_SORT = 40


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

    x = [len for len, frequency in len_dist_mean_sorted]
    y = [frequency for len, frequency in len_dist_mean_sorted]

    p = figure(
        title=f"How often will recurse be called with this length when sorting {TEST_SIZE} elements",
        x_axis_label="v.len() in recurse (log)",
        x_axis_type="log",
        y_axis_label="Average times called",
        tools="pan,wheel_zoom,box_zoom,reset,hover",
    )

    color = "green"
    p.square(x, y, fill_color=None, line_color=color)
    p.line(x, y, line_color=color)

    y2 = [
        frequency if len <= MAX_SMALL_SORT else 0
        for len, frequency in len_dist_mean_sorted
    ]
    p.varea(
        x,
        y1=0,
        y2=y2,
        alpha=0.3,
        fill_color=color,
        legend_label=f"Handled by dedicated small sort len <= {MAX_SMALL_SORT}",
    )

    show(p)


if __name__ == "__main__":
    file_path = sys.argv[1]

    with open(file_path, "r") as file:
        text = file.read()

    analyze_pivot_data(text)
