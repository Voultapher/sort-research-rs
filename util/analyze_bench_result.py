import json
import statistics
import sys
import math


class BenchEntry:
    def __init__(self, time, size):
        self.time_ns = time
        self.size = size


def parse_result(path):
    with open(path, "r") as file:
        return json.load(file)


def extract_groups(bench_result):
    groups = {}

    for benchmark, value in bench_result["benchmarks"].items():
        entry_parts = benchmark.split("-")
        test_size = int(entry_parts[3].partition(":")[0])
        # if test_size < 20:
        #     size_range = "-20-sub"
        # else:
        #     size_range = "-20-plus"

        ty = "-".join(entry_parts[:3])
        bench_time = value["criterion_estimates_v1"]["median"]["point_estimate"]

        groups.setdefault(ty, {})[benchmark] = BenchEntry(bench_time, test_size)

    return groups


def calc_a_percent_larger_than_b(a, b):
    assert a >= b
    return ((a - b) / b) * 100.0


def calc_part(list_full, list_partial):
    if len(list_partial) == 0:
        return 0.0

    return round((len(list_partial) / len(list_full)) * 100, 2)


def calc_elem_per_us(group):
    # / max(1.0, math.log(max(1, entry.size)))
    return round(
        statistics.median(
            [entry.size / (entry.time_ns / 1000.0) for entry in group.values()]
        )
    )


def p_per(number):
    """Print percent justified"""
    return f"{number}%".ljust(7)


def analyze_group_pair(group_a, group_b):
    # Threshold, anything below this is considered noise and not relevant.
    threshold = 1.02

    bench_times = [
        (entry_a.time_ns, group_b[bench_name].time_ns)
        for bench_name, entry_a in group_a.items()
    ]

    slowdowns = [
        calc_a_percent_larger_than_b(time_a, time_b)
        for (time_a, time_b) in bench_times
        if time_a > (time_b * threshold)
    ]

    speedups = [
        calc_a_percent_larger_than_b(time_b, time_a)
        for (time_a, time_b) in bench_times
        if time_b > (time_a * threshold)
    ]

    if len(speedups) > 0:
        speedup_percent = round(calc_part(bench_times, speedups))
        speedup_val = round(statistics.median(speedups))

        speedup_val_str = f"({speedup_val}%)".ljust(5)
        speedup = f"{p_per(speedup_percent)}  {speedup_val_str}"
    else:
        speedup = "no speedup".ljust(14)

    if len(slowdowns) > 0:
        slowdown_percent = round(calc_part(bench_times, slowdowns))
        slowdown_val = round(statistics.median(slowdowns))

        slowdown_val_str = f"({slowdown_val}%)".ljust(10)
        slowdown = f"{p_per(slowdown_percent)}   {slowdown_val_str}"
    else:
        slowdown = "no slowdown".ljust(20)

    a_elem_per_us = calc_elem_per_us(group_a)
    b_elem_per_us = calc_elem_per_us(group_b)

    return (
        f"{speedup}  |     {slowdown} {str(a_elem_per_us).rjust(3)} vs {b_elem_per_us}"
    )


def analyze_bench_results(result_a, result_b):
    name_a = result_a["name"]
    name_b = result_b["name"]

    groups_a = extract_groups(result_a)
    groups_b = extract_groups(result_b)

    name_a_title = f"[{name_a}]".ljust(15)
    name_b_title = f"[{name_b}]".ljust(15)
    print(f"Comparing                          {name_a_title} |     {name_b_title}")
    print(
        f"[Benchmark]                        speedup% (avg.) |     slowdown% (avg.)  avg. elem/us a vs b"
    )

    for name, group_a in sorted(groups_a.items()):
        group_b = groups_b[name]

        a_vs_b = analyze_group_pair(group_a, group_b)

        name_padded = f"[{name}]:".ljust(35)
        print(f"{name_padded}{a_vs_b}")


if __name__ == "__main__":
    result_a = parse_result(sys.argv[1])
    result_b = parse_result(sys.argv[2])

    analyze_bench_results(result_a, result_b)
