import statistics
import sys

from pprint import pprint

from collections import defaultdict


def load_results(path):
    with open(path, "r") as file:
        results = []
        for line in file:
            if " " not in line:
                continue

            l, _, r = line.partition(" ")

            try:
                # len = int(l, base=8)
                # ref_cycles = int(r, base=8)
                len = int(l)
                ref_cycles = int(r)

                results.append((len, ref_cycles))
            except Exception:
                pass

    return results


if __name__ == "__main__":
    results = load_results(sys.argv[1])
    print(f"total results: {len(results)}")

    total_ref_cycles = sum([ref_cycles for len, ref_cycles in results], 0)
    print(f"total ref_cycles: {total_ref_cycles}")

    size_dist = defaultdict(lambda: 0)
    for len, ref_cyles in results:
        size_dist[len] += 1

    print(f"size dist: {sorted(size_dist.items())}")

    total_insertion = sum([count for size, count in size_dist.items() if size <= 20], 0)
    print(f"total results size <= 20: {total_insertion}")

    total_ref_cycles_non_insertion = sum([ref_cycles for len, ref_cycles in results if len > 20], 0)
    print(f"total ref_cycles len >= 20: {total_ref_cycles_non_insertion}")

    run_dists = defaultdict(list)
    for len, ref_cycles in results:
        run_dists[len].append(ref_cycles)

    run_dist_medians = {}
    for len, run_dist in run_dists.items():
        run_dist_medians[len] = (statistics.median(run_dist), sum(run_dist, 0))

    pprint(run_dist_medians)

