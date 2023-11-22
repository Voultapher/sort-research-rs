import subprocess
import sys
import os
import shutil
import argparse
import json

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


def run_benchmarks(test_name, bench_name_overwrite):
    # Clean target/criterion a messy one can cause issues when exporting with critcmp.
    # We made sure we are in the current dir earlier.
    cur_dir = os.path.abspath(os.getcwd())
    crit_dir = os.path.join(cur_dir, "target", "criterion")
    if os.path.exists(crit_dir):
        shutil.rmtree(crit_dir)

    if "BENCH_REGEX" not in os.environ:
        user_val = input(
            "Are you sure you want to run all the benchmarks without a custom filter? This may take days to complete. [y/N]"
        )
        if user_val.lower().strip() != "y":
            print(
                "\nSpecify a custom filter by setting the enviroment variable BENCH_REGEX. See the README.md for more info."
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
            "--noplot",
            "--save-baseline",
            test_name,
        ],
        check=True,
        env=os.environ,
    )

    critcmp_result = subprocess.run(
        ["critcmp", "--export", test_name], capture_output=True
    )

    if critcmp_result.returncode != 0:
        critcmp_result_stderr = critcmp_result.stderr.decode("utf-8")
        sys.stderr.write(f"\n[Error] Failed to export results with critcmp: {critcmp_result_stderr}")
        sys.exit(critcmp_result.returncode)

    bench_results = critcmp_result.stdout.decode("utf-8")

    out_file_name = f"{test_name}.json"
    with open(out_file_name, "w+") as result_file:
        result_file.write(bench_results)

    print(f"\nWrote results to {out_file_name}")
    return out_file_name


def run_benchmarks_variant(test_name, variant):
    variant_name = variant["name"]
    setup_cmd = variant["setup_cmd"]
    bench_name_overwrite = variant["BENCH_NAME_OVERWRITE"]

    if bench_name_overwrite != "":
        os.environ["BENCH_NAME_OVERWRITE"] = bench_name_overwrite

    if setup_cmd != "":
        print(f"Running setup_cmd: {setup_cmd}")
        subprocess.run(setup_cmd, shell=True, check=True)

    full_test_name = test_name
    if variant_name != "":
        full_test_name = f"{test_name}_{variant_name}"

    print(f"Running test: {full_test_name}")

    return run_benchmarks(full_test_name, bench_name_overwrite)


def combine_out_files(test_name, out_file_names):
    out_name = f"{test_name}.json"

    if len(out_file_names) == 1 and out_name == out_file_names[0]:
        return

    combined_result = json.loads(
        open(out_file_names[0], "r", encoding="utf-8").read()
    )

    for out_file_name in out_file_names[1:]:
        parsed_result = json.loads(
            open(out_file_name, "r", encoding="utf-8").read()
        )
        combined_result["benchmarks"] |= parsed_result["benchmarks"]

    with open(out_name, "w+", encoding="utf-8") as out_file:
        out_file.write(json.dumps(combined_result, indent=2))
        out_file.flush()

    print(f"Wrote combined results to {out_name}")

    for out_file_name in out_file_names:
        os.remove(out_file_name)


if __name__ == "__main__":
    check_for_critcmp()
    check_for_correct_dir()

    variants_help = """"List of variants of the code that should be tested.
E.g. `--variants path/to/variants.json`
JSON format:
{
    "test_name": "my_test_zen3",
    "variants": [
        {
            "name": "a",
            "setup_cmd": "",
            "BENCH_NAME_OVERWRITE": ""
        },
        {
            "name": "b",
            "setup_cmd": "git checkout xxx",
            "BENCH_NAME_OVERWRITE": "rust_ipnsort_unstable:rust_ipnsort_new_unstable"
        }
    ]
}
"""

    parser = argparse.ArgumentParser(
        description="Tool for running and collecting benchmark results"
    )
    parser.add_argument("--variants", dest="variants_file", help=variants_help)
    parser.add_argument(
        "test_name",
        nargs="?",
        help="Test name including CPU arch name, e.g. my_test_zen3",
    )
    args = parser.parse_args()

    default_variants = {
        "test_name": args.test_name,
        "variants": [
            {"name": "", "setup_cmd": "", "BENCH_NAME_OVERWRITE": ""}
        ],
    }
    variants = (
        json.loads(open(args.variants_file, "r", encoding="utf-8").read())
        if args.variants_file
        else default_variants
    )

    variant_names = [v["name"] for v in variants["variants"]]
    assert len(variants["variants"]) == len(
        set(variant_names)
    ), f"You need to specify unique variant names, got: {variant_names}"

    if len(variants["variants"]) > 1:
        print("Testing variant setup commands")
        for variant in variants["variants"]:
            subprocess.run(variant["setup_cmd"], shell=True, check=True)

    test_name = variants["test_name"]
    out_file_names = []
    for variant in variants["variants"]:
        out_file_names.append(run_benchmarks_variant(test_name, variant))

    if len(out_file_names) > 1:
        combine_out_files(test_name, out_file_names)
