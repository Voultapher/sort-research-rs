import os
import sys
import subprocess
import shutil

PLOTS = ["scaling", "single_size", "direct_versus"]

from util import parse_skip

if __name__ == "__main__":
    this_dir = os.path.dirname(os.path.abspath(__file__))
    current_dir = os.path.abspath(os.getcwd())
    name = os.path.basename(sys.argv[1]).partition(".")[0]
    analysis_dir = os.path.join(current_dir, f"analysis_{name}")

    if os.path.exists(analysis_dir):
        shutil.rmtree(analysis_dir)

    os.mkdir(analysis_dir)

    plot_skip = parse_skip("PLOT_SKIP")

    bench_result_paths = []

    for path in sys.argv[1:]:
        if not os.path.exists(path):
            print(f"{path} not found, skipping.")
            continue

        bench_result_paths.append(os.path.abspath(path))

    for plot in PLOTS:
        if plot in plot_skip:
            continue

        cwd = os.path.join(analysis_dir, plot)
        if not os.path.exists(cwd):
            os.mkdir(cwd)

        args = [
            sys.executable,
            os.path.join(this_dir, f"{plot}.py"),
        ] + bench_result_paths
        subprocess.run(args, cwd=cwd, check=True)
