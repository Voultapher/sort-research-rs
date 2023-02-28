import subprocess
import sys
import os
import shutil

from graph_bench_result.cpu_info import get_cpu_info


def check_for_critcmp():
    subprocess.run(["critcmp", "--version"], check=True, capture_output=True)


def check_for_correct_dir():
    cur_dir = os.path.abspath(os.getcwd())
    cargo_lock_path = os.path.join(cur_dir, "Cargo.lock")
    build_rs_path = os.path.join(cur_dir, "build.rs")

    if not (os.path.exists(cargo_lock_path) and os.path.exists(build_rs_path)):
        print(
            "Please make sure to run this program with the repo root dir as the current working directory."
        )
        sys.exit(1)


def run_benchmarks(test_name):
    # Clean target/criterion a messy one can cause issues when exporting with critcmp.
    # We made sure we are in the current dir earlier.
    cur_dir = os.path.abspath(os.getcwd())
    crit_dir = os.path.join(cur_dir, "target", "criterion")
    if os.path.exists(crit_dir):
        shutil.rmtree(crit_dir)

    if "CUSTOM_BENCH_REGEX" not in os.environ:
        user_val = input(
            "Are you sure you want to run all the benchmarks without a custom filter? This may take days to complete. [y/N]"
        )
        if user_val.lower().strip() != "y":
            print(
                "\nSpecify a custom filter by setting the enviroment variable CUSTOM_BENCH_REGEX. See the README.md for more info."
            )
            sys.exit(1)

    subprocess.run(
        [
            "cargo",
            "bench",
            "--features",
            "cold_benchmarks",
            "--bench",
            "bench",
            "--",
            "--warm-up-time",
            "2",
            "--measurement-time",
            "4",
            "--save-baseline",
            test_name,
        ],
        check=True,
    )

    critcmp_result = subprocess.run(
        ["critcmp", "--export", test_name], check=True, capture_output=True
    )

    bench_results = critcmp_result.stdout.decode("utf-8")

    out_file_name = f"{test_name}.json"
    with open(out_file_name, "w+") as result_file:
        result_file.write(bench_results)

    print(f"\nWrote results to {out_file_name}")


if __name__ == "__main__":
    check_for_critcmp()
    check_for_correct_dir()

    if len(sys.argv) < 2:
        print(
            "Please specify a name including the cpu arch as command line parameter, eg. my_test_zen3"
        )
        sys.exit(1)

    test_name = sys.argv[1]
    try:
        _ = get_cpu_info(test_name)
    except Exception:
        print(
            "Unknown cpu arch, please adapt util/graph_bench_result/cpu_info.py"
        )
        sys.exit(1)

    run_benchmarks(test_name)
