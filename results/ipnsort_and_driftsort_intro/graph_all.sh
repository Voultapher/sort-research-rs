#!/usr/bin/env bash

SRC_PATH=/home/dev/projects/rust/sort-research-rs

source $SRC_PATH/util/graph_bench_result/venv/bin/activate

set -euxo pipefail

rm -rf comp_info_analysis
mkdir comp_info_analysis

# python $SRC_PATH/util/graph_bench_result/comp_count.py comp_info/unstable.txt
# mv unstable.html comp_info_analysis/
# python $SRC_PATH/util/graph_bench_result/comp_count.py comp_info/pdqsort.txt
# mv pdqsort.html comp_info_analysis/

# python $SRC_PATH/util/graph_bench_result/comp_count.py comp_info/stable.txt
# mv stable.html comp_info_analysis/
# python $SRC_PATH/util/graph_bench_result/comp_count.py comp_info/glidesort.txt
# mv glidesort.html comp_info_analysis/

# python $SRC_PATH/util/graph_bench_result/graph_all.py zen3_unstable.json
# python $SRC_PATH/util/graph_bench_result/graph_all.py haswell_unstable.json
# python $SRC_PATH/util/graph_bench_result/graph_all.py firestorm_unstable.json

export CLIP_PATTERN_OVERRIDE="random_z1"
python $SRC_PATH/util/graph_bench_result/graph_all.py zen3_stable.json
# python $SRC_PATH/util/graph_bench_result/graph_all.py haswell_stable.json
# python $SRC_PATH/util/graph_bench_result/graph_all.py firestorm_stable.json


