import json

from collections import defaultdict

from bokeh.palettes import Colorblind


def parse_result(path):
    with open(path, "r") as file:
        return json.load(file)


def extract_groups(bench_result):
    # Result layout:
    # { type (eg. u64):
    #   { prediction_state (eg. hot):
    #     { test_size (eg. 500):
    #       { pattern (eg. descending):
    #         { sort_name (eg. rust_std_stable):
    #            bench_time_ns
    groups = defaultdict(
        lambda: defaultdict(
            lambda: defaultdict(lambda: defaultdict(lambda: {}))
        )
    )

    for benchmark_full, value in bench_result["benchmarks"].items():
        sort_name, _, benchmark = benchmark_full.partition("-")

        entry_parts = benchmark.split("-")

        pred_state = entry_parts[0]
        ty = entry_parts[1]
        pattern = entry_parts[2]
        test_size = int(entry_parts[3])

        if sort_name == "c_fluxsort_stable" and ty not in ("u64", "i32"):
            continue

        if "_stable" in sort_name:
            continue  # TODO graph all.

        # if "radix" in sort_name:
        #     continue

        bench_time_ns = value["criterion_estimates_v1"]["median"][
            "point_estimate"
        ]

        groups[ty][pred_state][test_size][pattern][sort_name] = bench_time_ns

    return groups


def type_size(type_name):
    if type_name == "i32":
        return 4
    elif type_name == "u64":
        return 8
    elif type_name == "string":
        return 24
    elif type_name == "f128":
        return 16
    elif type_name == "1k":
        return 1_000

    raise Exception(f"Unknown type: {type_name}")


def build_color_palette():
    # Use color blind palette to increase accessibility.
    palette = list(Colorblind[8])

    # Make colors more consistent by pinning them to a specific sort
    # regardless of the set of tested sorts.
    # This avoids color swapping between different graphs.
    pinned_colors = {
        # Stable
        "c_fluxsort_stable": palette[0],
        "cpp_std_sys_stable": palette[1],
        "cpp_std_msvc_stable": palette[1],
        "rust_std_stable": palette[2],
        "rust_glidesort_stable": palette[3],
        "cpp_std_libcxx_stable": palette[4],
        "rust_ipn_stable": palette[5],
        "cpp_powersort_stable": palette[6],
        "cpp_powersort_4way_stable": palette[7],
        "rust_wpwoodjr_stable": palette[7],
        # Unstable
        "c_crumsort_unstable": palette[0],
        "cpp_std_sys_unstable": palette[1],
        "cpp_std_msvc_unstable": palette[1],
        "rust_std_unstable": palette[2],
        "cpp_pdqsort_unstable": palette[3],
        "cpp_std_libcxx_unstable": palette[4],
        "rust_ipn_unstable": palette[5],
        "cpp_ips4o_unstable": palette[6],
        "cpp_blockquicksort": palette[7],
        # There are more sorts but they don't really fit the graph or colors at
        # the same time
        "rust_radsort_radix": palette[4],
        "cpp_vqsort": palette[6],
        "cpp_intel_avx512": palette[7],
    }

    return pinned_colors
