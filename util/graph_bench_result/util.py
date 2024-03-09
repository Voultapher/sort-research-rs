import json
import os
import sys

from collections import defaultdict
from itertools import chain

from bokeh.palettes import Colorblind


def parse_bench_results(paths):
    """Parse a list of benchmark result files, returning a unified auto-spliced
    representation, in the groups format."""
    if len(paths) == 0:
        raise Exception("paths must not be non empty list.")

    def parse_bench_result(path):
        with open(path, "r") as file:
            file_content = json.load(file)

        source_format = (
            "critcmp"
            if file_content.get("benchmarks") is not None
            else "rustc-sort-bench"
        )

        is_new_path = "_new" in path and len(paths) > 1 and path == paths[0]

        if source_format == "critcmp":

            def bench_result_iter():
                for key, value in file_content["benchmarks"].items():
                    benchmark_key = BenchmarkKey(key)
                    if is_new_path:
                        benchmark_key.sort_name += "_new"
                    bench_time_ns = value["criterion_estimates_v1"]["median"][
                        "point_estimate"
                    ]

                    yield (benchmark_key, bench_time_ns)

        elif source_format == "rustc-sort-bench":

            def bench_result_iter():
                for key, time_opaque in file_content["results"].items():
                    benchmark_key = BenchmarkKey(key)
                    if is_new_path:
                        benchmark_key.sort_name += "_new"

                    # FIXME this is needs to be converted to ns.
                    bench_time_ns = time_opaque

                    yield (benchmark_key, bench_time_ns)

        return list(bench_result_iter())

    parsed_bench_results = [parse_bench_result(path) for path in paths]

    # If there are multiple results, the first one will be the basis for the
    # others. So the most limited set of results should be the first one passed
    # to the tools.
    baseline = parsed_bench_results[0]
    splice_filter_fn = build_auto_splice_filter(baseline)

    return extract_groups(chain.from_iterable(parsed_bench_results), splice_filter_fn)


def base_name():
    return os.path.basename(sys.argv[1]).partition(".")[0]


def plot_name_suffix():
    return os.environ.get("PLOT_NAME_SUFFIX", "")


class BenchmarkKey:
    def __init__(self, key):
        self.sort_name, _, benchmark = key.partition("-")

        entry_parts = benchmark.split("-")

        self.pred_state = entry_parts[0]
        self.ty = entry_parts[1]
        self.pattern = entry_parts[2]
        self.test_len_str = entry_parts[3]
        self.test_len = int(self.test_len_str)


def build_auto_splice_filter(bench_result_iter):
    """Returns filter function based on baseline"""

    # sort_names = set()
    pred_states = set()
    types = set()
    patterns = set()
    test_lens = set()

    for benchmark_key, bench_time_ns in bench_result_iter:
        pred_states.add(benchmark_key.pred_state)
        types.add(benchmark_key.ty)
        patterns.add(benchmark_key.pattern)
        test_lens.add(benchmark_key.test_len)

    def filter_fn(benchmark_key):
        if benchmark_key.pred_state not in pred_states:
            return True

        if benchmark_key.ty not in types:
            return True

        if benchmark_key.pattern not in patterns:
            return True

        if benchmark_key.test_len not in test_lens:
            return True

        return False

    return filter_fn


def parse_skip(key):
    skip = os.environ.get(key)
    if skip is None:
        return []

    return skip.replace(" ", "").split(",")


def build_env_filter():
    """Returns filter function based on environment variable filter settings"""

    sort_name_skip = parse_skip("SORT_NAME_SKIP")
    pred_state_skip = parse_skip("PRED_STATE_SKIP")
    type_skip = parse_skip("TYPE_SKIP")
    test_len_skip = parse_skip("TEST_LEN_SKIP")
    pattern_skip = parse_skip("PATTERN_SKIP")

    def filter_fn(benchmark_key):
        if benchmark_key.sort_name in sort_name_skip:
            return True

        if benchmark_key.pred_state in pred_state_skip:
            return True

        if benchmark_key.ty in type_skip:
            return True

        if benchmark_key.test_len_str in test_len_skip:
            return True

        if benchmark_key.pattern in pattern_skip:
            return True

        return False

    return filter_fn


def extract_groups(bench_result_iter, splice_filter_fn):
    # Result layout:
    # { type (eg. u64):
    #   { prediction_state (eg. hot):
    #     { test_len (eg. 500):
    #       { pattern (eg. descending):
    #         { sort_name (eg. rust_std_stable):
    #            bench_time_ns
    groups = defaultdict(
        lambda: defaultdict(lambda: defaultdict(lambda: defaultdict(lambda: {})))
    )

    env_filter_fn = build_env_filter()

    for benchmark_key, bench_time_ns in bench_result_iter:
        if env_filter_fn(benchmark_key):
            continue

        if splice_filter_fn is not None and splice_filter_fn(benchmark_key):
            continue

        groups[benchmark_key.ty][benchmark_key.pred_state][benchmark_key.test_len][
            benchmark_key.pattern
        ][benchmark_key.sort_name] = bench_time_ns

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
        "rust_std_stable": (palette[4], "square"),
        "rust_glidesort_stable": (palette[3], "triangle"),
        "rust_driftsort_stable": (palette[7], "square"),
        "cpp_std_libcxx_stable": (palette[4], "square"),
        "rust_ipn_stable": (palette[5], "square"),
        "cpp_powersort_stable": (palette[6], "square"),
        "cpp_powersort_4way_stable": (palette[7], "square"),
        "rust_wpwoodjr_stable": (palette[7], "square"),
        "rust_tinymergesort_stable": (palette[7], "square"),
        "rust_driftsort_stable": (palette[6], "circle"),
        # Unstable
        "c_crumsort_unstable": (palette[0], "square"),
        "cpp_std_sys_unstable": (palette[1], "square"),
        "cpp_std_gnu_unstable": (palette[1], "square"),
        "cpp_std_msvc_unstable": (palette[1], "square"),
        "rust_std_unstable": (palette[4], "square"),
        "cpp_pdqsort_unstable": (palette[3], "square"),
        "cpp_std_libcxx_unstable": (palette[4], "square"),
        "rust_ipn_unstable": (palette[5], "square"),
        "rust_ipnsort_unstable": (palette[5], "circle"),
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
        "random_z1": (palette[6], "inverted_triangle"),
        "random": (palette[7], "triangle"),
    }

    return meta_info
