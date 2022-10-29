# The goal is to find a good way to skew small size pivots to fit the
# sorting-network sizes.

import sys

from collections import defaultdict

from bokeh.plotting import figure, show


def analyze_pivot_data(text):
    len_data = list(
        filter(lambda line: line.startswith("len: "), text.split("\n"))
    )

    print(len(len_data))

    len_dist = defaultdict(lambda: 0)

    for line in len_data:
        sort_len = int(line.partition(":")[2].partition(",")[0])
        len_dist[sort_len] += 1

    len_dist = sorted(len_dist.items())

    x = [len for len, frequency in len_dist]
    y = [frequency for len, frequency in len_dist]

    p = figure(
        title="Simple line example",
        x_axis_label="x",
        # x_axis_type="log",
        y_axis_label="y",
        tools="pan,wheel_zoom,box_zoom,reset,hover",
    )

    color = "green"
    p.square(x, y, fill_color=None, line_color=color)
    p.line(x, y, line_color=color)

    show(p)


if __name__ == "__main__":
    file_path = sys.argv[1]

    with open(file_path, "r") as file:
        text = file.read()

    analyze_pivot_data(text)
