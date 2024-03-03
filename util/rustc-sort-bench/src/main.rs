mod benchmark;
mod evaluate;
mod measure;
mod patterns;

use std::env;
use std::path::PathBuf;

use crate::evaluate::{compare_sort, Sort};

struct StdStable {}

impl Sort for StdStable {
    fn name() -> String {
        "rust_std_stable".into()
    }

    fn sort<T: Ord>(v: &mut [T]) {
        v.sort();
    }
}

struct StdUnstable {}

impl Sort for StdUnstable {
    fn name() -> String {
        "rust_std_unstable".into()
    }

    fn sort<T: Ord>(v: &mut [T]) {
        v.sort_unstable();
    }
}

struct IpnsortUnstable {}

impl Sort for IpnsortUnstable {
    fn name() -> String {
        "rust_ipnsort_unstable".into()
    }

    fn sort<T: Ord>(v: &mut [T]) {
        ipnsort::sort(v);
    }
}

struct DriftsortStable {}

impl Sort for DriftsortStable {
    fn name() -> String {
        "rust_driftsort_stable".into()
    }

    fn sort<T: Ord>(v: &mut [T]) {
        driftsort::sort(v);
    }
}

fn main() {
    let args = env::args().collect::<Vec<_>>();
    let base_line_path = PathBuf::from(args.get(1).expect(
        "Please provide a base_line_path, that will either be created or compared against.",
    ));

    compare_sort::<DriftsortStable>(&base_line_path);
}
