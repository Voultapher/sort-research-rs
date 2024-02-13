import json
import os
import sys

from collections import defaultdict

from bokeh.palettes import Colorblind


def parse_result(path):
    with open(path, "r") as file:
        return json.load(file)


def parse_skip(key):
    skip = os.environ.get(key)
    if skip is None:
        return []

    return skip.replace(" ", "").split(",")


def base_name():
    return os.path.basename(sys.argv[1]).partition(".")[0]


def plot_name_suffix():
    return os.environ.get("PLOT_NAME_SUFFIX", "")


def extract_groups(bench_result):
    # Result layout:
    # { type (eg. u64):
    #   { prediction_state (eg. hot):
    #     { test_len (eg. 500):
    #       { pattern (eg. descending):
    #         { sort_name (eg. rust_std_stable):
    #            bench_time_ns
    groups = defaultdict(
        lambda: defaultdict(
            lambda: defaultdict(lambda: defaultdict(lambda: {}))
        )
    )

    sort_name_skip = parse_skip("SORT_NAME_SKIP")
    pred_state_skip = parse_skip("PRED_STATE_SKIP")
    type_skip = parse_skip("TYPE_SKIP")
    test_len_skip = parse_skip("TEST_LEN_SKIP")
    pattern_skip = parse_skip("PATTERN_SKIP")

    for benchmark_full, value in bench_result["benchmarks"].items():
        sort_name, _, benchmark = benchmark_full.partition("-")

        entry_parts = benchmark.split("-")

        pred_state = entry_parts[0]
        ty = entry_parts[1]
        pattern = entry_parts[2]
        test_len_str = entry_parts[3]
        test_len = int(test_len_str)

        if sort_name in sort_name_skip:
            continue

        if pred_state in pred_state_skip:
            continue

        if ty in type_skip:
            continue

        if test_len_str in test_len_skip:
            continue

        if pattern in pattern_skip:
            continue

        if test_len <= 32:
            continue

        if pattern == "saws_short":
            continue

        # if sort_name not in (
        #     "lomuto_branchy",
        #     "lomuto_branchless",
        #     "lomuto_branchless_cyclic",
        #     "lomuto_branchless_cyclic_opt",
        # ):
        #     continue

        bench_time_ns = value["criterion_estimates_v1"]["median"][
            "point_estimate"
        ]

        groups[ty][pred_state][test_len][pattern][sort_name] = bench_time_ns

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


def build_implementation_meta_info():
    """
    Returns a dict with color and symbol information pinned to a specific
    implementation.

    This is used to visually identify the different implementations.
    """

    # Use color blind palette to increase accessibility.
    palette = list(Colorblind[8])

    # Make colors more consistent by pinning them to a specific sort
    # regardless of the set of tested sorts.
    # This avoids color swapping between different graphs.
    meta_info = {
        # Stable
        "c_fluxsort_stable": (palette[0], "square"),
        "cpp_std_sys_stable": (palette[1], "square"),
        "cpp_std_gnu_stable": (palette[1], "square"),
        "cpp_std_msvc_stable": (palette[1], "square"),
        "rust_std_stable": (palette[2], "square"),
        "rust_glidesort_stable": (palette[3], "square"),
        "rust_driftsort_stable": (palette[7], "square"),
        "cpp_std_libcxx_stable": (palette[4], "square"),
        "rust_ipn_stable": (palette[5], "square"),
        "cpp_powersort_stable": (palette[6], "square"),
        "cpp_powersort_4way_stable": (palette[7], "square"),
        "rust_wpwoodjr_stable": (palette[7], "square"),
        "rust_tinymergesort_stable": (palette[7], "square"),
        # Unstable
        "c_crumsort_unstable": (palette[0], "square"),
        "cpp_std_sys_unstable": (palette[1], "square"),
        "cpp_std_gnu_unstable": (palette[1], "square"),
        "cpp_std_msvc_unstable": (palette[1], "square"),
        "rust_std_unstable": (palette[2], "square"),
        "cpp_pdqsort_unstable": (palette[3], "square"),
        "cpp_std_libcxx_unstable": (palette[4], "square"),
        "rust_ipn_unstable": (palette[5], "square"),
        "rust_ipnsort_unstable": (palette[5], "square"),
        "cpp_ips4o_unstable": (palette[6], "square"),
        "cpp_blockquicksort": (palette[7], "square"),
        "rust_tinyheapsort_unstable": (palette[7], "square"),
        # There are more sorts but they don't really fit the graph or colors at
        # the same time
        "rust_radsort_radix": (palette[4], "square"),
        "cpp_vqsort": (palette[6], "square"),
        "cpp_intel_avx512": (palette[7], "square"),
        "singeli_singelisort": (palette[3], "square"),
        # For partition bench
        "hoare_branchy": (palette[1], "diamond"),
        "hoare_block": (palette[4], "plus"),
        "hoare_crumsort": (palette[3], "square_pin"),
        "lomuto_branchy": (palette[0], "square"),
        "lomuto_branchless": (palette[5], "circle"),
        "lomuto_branchless_cyclic": (palette[6], "square_cross"),
        "lomuto_branchless_cyclic_opt": (palette[7], "triangle"),
    }

    return meta_info


def build_pattern_meta_info():
    """
    Returns a dict with color and symbol information pinned to a specific
    pattern.

    This is used to visually identify the different patterns.
    """

    # Use color blind palette to increase accessibility.
    palette = list(Colorblind[8])

    meta_info = {
        "ascending": (palette[0], "diamond"),
        "descending": (palette[1], "square"),
        "random_d20": (palette[3], "square_pin"),
        "random_p5": (palette[4], "square_cross"),
        "random_s95": (palette[5], "circle"),
        "random_z1": (palette[6], "square_cross"),
        "random": (palette[7], "triangle"),
    }

    return meta_info
