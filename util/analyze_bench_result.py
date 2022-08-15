import json
import statistics
import sys


def parse_result(path):
    with open(path, "r") as file:
        return json.load(file)


def extract_groups(bench_result):
    groups = {}

    for benchmark, value in bench_result["benchmarks"].items():
        entry_parts = benchmark.split("-")
        test_size = int(entry_parts[3].partition(":")[0])
        if test_size < 20:
            size_range = "-20-sub"
        else:
            size_range = "-20-plus"

        ty = "-".join(entry_parts[:3]) + size_range
        bench_time = value["criterion_estimates_v1"]["median"]["point_estimate"]

        groups.setdefault(ty, {})[benchmark] = bench_time

    return groups


def analyze_bench_results(result_a, result_b):
    name_a = result_a["name"]
    name_b = result_b["name"]

    groups_a = extract_groups(result_a)
    groups_b = extract_groups(result_b)

    def diff_percent(time_a, time_b):
        return ((time_b - time_a) / abs(time_a)) * 100

    print(f"Percent runtime more by {name_b} than {name_a}")

    # print(groups_a["cold-i32-20-plus"])
    # print(groups_b["cold-i32-20-plus"])

    for name, group_a in sorted(groups_a.items()):
        group_b = groups_b[name]
        runtime_diffs = [
            diff_percent(bench_time, group_b[bench_name])
            for bench_name, bench_time in group_a.items()
        ]

        median_diff = f"median: {round(statistics.median(runtime_diffs))}%"
        min_diff = f"min: {round(min(runtime_diffs))}%"
        max_diff = f"max: {round(max(runtime_diffs))}%"

        key_pad = " " * (40 - len(name))
        median_pad = " " * (14 - len(median_diff))
        min_pad = " " * (11 - len(min_diff))
        print(
            f"[{name}]:{key_pad}{median_diff}{median_pad}{min_diff}{min_pad}{max_diff}"
        )


if __name__ == "__main__":
    result_a = parse_result(sys.argv[1])
    result_b = parse_result(sys.argv[2])

    analyze_bench_results(result_a, result_b)
