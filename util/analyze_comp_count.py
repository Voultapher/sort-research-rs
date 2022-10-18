import collections
import statistics
import sys


class Entry:
    def __init__(self, pattern, comp_count):
        self.pattern = pattern
        self.comp_count = comp_count

    def __repr__(self):
        return f"Entry(pattern: {self.pattern}, comp_count: {self.comp_count})"


def bucketize_comp_data(name: str):
    result = collections.defaultdict(list)

    for line in comp_data.splitlines():
        if line.startswith(name):
            entry = line.partition("-comp-")[2]

            entry_parts = entry.split("-")
            test_size = int(entry_parts[2].partition(":")[0])
            if test_size < 20:
                size_range = "-20-sub"
            else:
                size_range = "-20-plus"

            ty = "-".join(entry_parts[:2]) + size_range

            pattern, _, rest = entry.partition(":")
            comp_count = int(rest.partition(":")[2].strip())

            result[ty].append(Entry(pattern, comp_count))

    return result


# If val_a is larger than val_b -> % larger than val_b
# If val_b is larger than val_a -> % larger than val_a as negative number
# 100 == val_a 2x larger than val_b
# -100 == val_b 2x larger than val_a
def relative_speedup(val_a, val_b):
    if val_a <= val_b:
        # val_a is larger.
        return ((val_b / val_a) - 1) * 100
    else:
        # val_b is larger
        return -(((val_a / val_b) - 1) * 100)


def analyze_buckets(bucket_a, bucket_b):
    # Assumes both buckets have the same layout.

    def non_zero(zip_tuple):
        entry_a = zip_tuple[0]
        entry_b = zip_tuple[1]

        assert entry_a.pattern == entry_b.pattern

        # These just pollute the statistics and are only
        # relevant for empty slice sorts.
        return entry_a.comp_count != 0 and entry_b.comp_count != 0

    def diff_percent(zip_tuple):
        entry_a = zip_tuple[0]
        entry_b = zip_tuple[1]

        return relative_speedup(entry_a.comp_count, entry_b.comp_count)

    for key in sorted(bucket_a.keys()):
        filterd_comps = filter(non_zero, zip(bucket_a[key], bucket_b[key]))
        comp_diffs = list(map(diff_percent, filterd_comps))
        median_diff = f"median: {round(statistics.median(comp_diffs))}%"
        min_diff = f"min: {round(min(comp_diffs))}%"
        max_diff = f"max: {round(max(comp_diffs))}%"

        key_pad = " " * (35 - len(key))
        median_pad = " " * (14 - len(median_diff))
        min_pad = " " * (11 - len(min_diff))
        print(
            f"[{key}]:{key_pad}{median_diff}{median_pad}{min_diff}{min_pad}{max_diff}"
        )


def analyze(name_a, name_b):
    print(f"Percent comparisons done more by {name_b} than {name_a}")
    analyze_buckets(
        bucketize_comp_data(name_a),
        bucketize_comp_data(name_b),
    )


if __name__ == "__main__":
    with open(sys.argv[1], "r") as comp_data_file:
        comp_data = comp_data_file.read()

    analyze("std_stable", "new_stable")
