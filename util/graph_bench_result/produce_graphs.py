import json
import sys
import statistics

from bokeh import models

# from bokeh.io import export_png
from bokeh.plotting import figure, gridplot, ColumnDataSource
from bokeh.resources import CDN
from bokeh.embed import file_html

TRANSFORMS = ["i32", "u64", "string", "1k", "f128"]


class BenchEntry:
    def __init__(self, time, size):
        self.time_ns = time
        self.size = size

    def __repr__(self):
        return f"BenchEntry(time_ns: {round(self.time_ns)}, size: {self.size})"


def parse_result(path):
    with open(path, "r") as file:
        return json.load(file)


def extract_groups(bench_result):
    groups = {}

    for benchmark, value in bench_result["benchmarks"].items():
        entry_parts = benchmark.split("-")
        test_size = int(entry_parts[3].partition(":")[0])

        ty = "-".join(entry_parts[:3])
        bench_time = value["criterion_estimates_v1"]["median"][
            "point_estimate"
        ]

        groups.setdefault(ty, {})[benchmark] = BenchEntry(
            bench_time, test_size
        )

    return groups


# Needs to be shared instance :/
TOOLS = None


def init_tools_detailed():
    global TOOLS
    TOOLS = [
        models.WheelZoomTool(),
        models.BoxZoomTool(),
        models.PanTool(),
        models.HoverTool(
            tooltips=[
                ("Name", "@name"),
                ("Test Size", "@x"),
                ("Time in ns", "@y"),
            ],
        ),
        models.ResetTool(),
    ]


def init_tools_overview():
    global TOOLS
    TOOLS = [
        models.WheelZoomTool(),
        models.BoxZoomTool(),
        models.PanTool(),
        models.HoverTool(
            tooltips=[
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


def add_plot_line(p, filter_fn, name, group, color):
    entries = list(
        sorted(
            filter(filter_fn, group.values()),
            key=lambda entry: entry.size,
        )
    )

    x = [entry.size for entry in entries]
    y = [entry.time_ns for entry in entries]
    data = {"x": x, "y": y, "name": [name] * len(x)}

    source = ColumnDataSource(data=data)

    p.square(
        source=source, legend_label=name, fill_color=None, line_color=color
    )
    p.line(source=source, legend_label=name, line_color=color)
    p.legend.location = "top_left"


def make_detail_plot(bench_name, name_a, group_a, name_b, group_b):
    def produce_plot(p, filter_fn):
        for name, group, color in [
            (name_a, group_a, "green"),  # I hope these are readable
            (name_b, group_b, "orange"),
        ]:
            add_plot_line(p, filter_fn, name, group, color)

        return p

    cutoff = 20

    p1 = figure(
        title=f"{bench_name} < {cutoff}",
        x_axis_label="Input Size (log)",
        x_axis_type="log",
        y_axis_label="Time in ns",
        tools="",
    )
    add_tools_to_plot(p1)

    p2 = figure(
        title=f"{bench_name} >= {cutoff}",
        x_axis_label="Input Size (log)",
        x_axis_type="log",
        y_axis_label="Time in ns",
        y_axis_type="log",
        tools="",
    )
    add_tools_to_plot(p2)

    produce_plot(p1, lambda entry: entry.size < cutoff)
    produce_plot(p2, lambda entry: entry.size >= cutoff)

    return [p1, p2]


def plot_detailed(name_a, groups_a, name_b, groups_b):
    for transform in TRANSFORMS:
        for temp in ["hot", "cold"]:
            filter_name = f"{temp}-{transform}-"
            init_tools_detailed()
            detail_plots = [
                make_detail_plot(
                    bench_name,
                    name_a,
                    group_a,
                    name_b,
                    groups_b[bench_name],
                )
                for bench_name, group_a in groups_a.items()
                if bench_name.startswith(filter_name)
            ]

            grid_plot = gridplot(detail_plots)
            html = file_html(grid_plot, CDN, transform)
            with open(f"{temp}-{transform}.html", "w+") as outfile:
                outfile.write(html)


# If time_a is faster than time_b -> % faster than time_b
# If time_b is faster than time_a -> % faster than time_a as negative number
# 100 == time_a 2x faster than time_b
# -100 == time_b 2x faster than time_a
def relative_speedup(time_a, time_b):
    if time_a <= time_b:
        # time_a is faster.
        return ((time_b / time_a) - 1) * 100
    else:
        # time_b is faster
        return -(((time_a / time_b) - 1) * 100)


def plot_distribution(temp, transform, name_a, groups_a, name_b, groups_b):
    plot = figure(
        title=f"{name_a} vs {name_b} ({temp}-{transform})",
        x_axis_label="Input Size (log)",
        x_axis_type="log",
        y_axis_label=f"Relative speedup in %. {name_a} > 0 > {name_b}. 100% == 2x",
        tools="",
    )
    add_tools_to_plot(plot)

    test_sizes = []
    speedups = []
    names = []

    filter_name = f"{temp}-{transform}-"
    for bench_name, group_a in groups_a.items():
        if filter_name not in bench_name:
            continue

        group_b = groups_b[bench_name]

        for entry_name, entry_a in group_a.items():
            entry_b = group_b[entry_name]
            assert entry_a.size == entry_b.size

            test_sizes.append(entry_a.size)
            speedups.append(relative_speedup(entry_a.time_ns, entry_b.time_ns))
            names.append(entry_name)

    data = {"x": test_sizes, "y": speedups, "name": names}
    source = ColumnDataSource(data=data)

    plot.scatter(
        source=source,
        line_alpha=0.35,
        fill_alpha=0.18,
        line_color="royalblue",
        fill_color="royalblue",
        size=10,
    )

    test_sizes_unique = list(set(test_sizes))
    test_sizes_unique.sort()

    # This is really inefficient, but its ok for producing graphs.
    speedups_random_median = [
        [
            s
            for t, s, n in zip(test_sizes, speedups, names)
            if t == test_size and "-random-" in n
        ][0]
        for test_size in test_sizes_unique
    ]

    plot.line(
        x=test_sizes_unique,
        y=speedups_random_median,
        color="orange",
        legend_label="random pattern",
    )

    plot.legend.location = "top_left"

    return plot


def plot_overview(name_a, groups_a, name_b, groups_b):
    from bokeh.plotting import show

    for temp in ["hot", "cold"]:
        init_tools_overview()
        distribution_plots = [
            plot_distribution(
                temp, transform, name_a, groups_a, name_b, groups_b
            )
            for transform in TRANSFORMS
        ]

        grid_plot = gridplot(distribution_plots, ncols=3)
        show(grid_plot)


if __name__ == "__main__":
    result_a = parse_result(sys.argv[1])
    result_b = parse_result(sys.argv[2])

    name_a = result_a["name"]
    name_b = result_b["name"]

    groups_a = extract_groups(result_a)
    groups_b = extract_groups(result_b)

    assert len(groups_a) == len(groups_b)
    for bench_name, group_a in groups_a.items():
        assert len(group_a) == len(groups_b[bench_name])

    # plot_detailed(name_a, groups_a, name_b, groups_b)
    plot_overview(name_a, groups_a, name_b, groups_b)
