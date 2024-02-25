import json
import pathlib
from collections import defaultdict

import argparse

parser = argparse.ArgumentParser(description='Parse cargo bloat json for size differences')
parser.add_argument('baseline', type=pathlib.Path)
parser.add_argument('target', type=pathlib.Path)
parser.add_argument('-v', '--verbose', action='store_true')
args = parser.parse_args()

baseline = json.load(open(args.baseline))
target = json.load(open(args.target))

delta_sizes = defaultdict(int)
counts = defaultdict(int)
for f in baseline["functions"]:
    name = f.get("name") or "unknown"
    delta_sizes[name] -= f["size"]
    counts[name] -= 1

for f in target["functions"]:
    name = f.get("name") or "unknown"
    delta_sizes[name] += f["size"]
    counts[name] += 1

nonzero_deltas = {name: delta for name, delta in delta_sizes.items() if delta != 0 }

by_delta = defaultdict(list)
for name, delta in nonzero_deltas.items():
    by_delta[delta].append(name)
    
likely_renames = []
likely_renamed = set()
for name, delta in nonzero_deltas.items():
    if delta > 0 and len(by_delta[delta]) == 1 and len(by_delta[-delta]) == 1:
        old_name = by_delta[-delta][0]
        likely_renames.append((old_name, name))
        likely_renamed.add(old_name)
        likely_renamed.add(name)

total_codesize = sum(nonzero_deltas.values())
total_count = sum(counts.values())

if args.verbose:
    print(f"{'size':>8} count name")
    for name, delta in sorted(nonzero_deltas.items(), key=lambda t: t[1], reverse=True):
        if name not in likely_renamed:
            net_count = counts[name]
            print(f"{delta:>8} {net_count:>5} {name}")
    print("-" * 20)
    print(f"{total_codesize:>8} total net codesize increase")
    print(f"{total_count:>8} total net functions added")

    print("\nnot included were the following likely renames / reinlines, as they had exactly the same size:")
    for old, new in likely_renames:
        print(f"    {old} --> {new}")
else:
    print(total_codesize)