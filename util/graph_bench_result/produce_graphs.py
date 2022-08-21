import json
import sys

from bokeh import models

# from bokeh.io import export_png
from bokeh.plotting import figure, gridplot, ColumnDataSource
from bokeh.resources import CDN
from bokeh.embed import file_html

TRANSFORMS = ["i32"]  # , "u64", "string", "1k", "f128"]


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


def init_tools():
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
            init_tools()
            grid_plot = gridplot(
                [
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
            )

            html = file_html(grid_plot, CDN, transform)
            with open(f"{temp}-{transform}.html", "w+") as outfile:
                outfile.write(html)


if __name__ == "__main__":
    result_a = parse_result(sys.argv[1])
    result_b = parse_result(sys.argv[2])

    name_a = result_a["name"]
    name_b = result_b["name"]

    groups_a = extract_groups(result_a)
    groups_b = extract_groups(result_b)

    plot_detailed(name_a, groups_a, name_b, groups_b)
