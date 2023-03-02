use sort_test_tools::instantiate_sort_tests;
use sort_test_tools::Sort;

struct SortImpl {}

impl Sort for SortImpl {
    fn name() -> String {
        "rust_std_stable".into()
    }

    fn sort<T>(arr: &mut [T])
    where
        T: Ord,
    {
        arr.sort();
    }

    fn sort_by<T, F>(arr: &mut [T], compare: F)
    where
        F: FnMut(&T, &T) -> std::cmp::Ordering,
    {
        arr.sort_by(compare);
    }
}

instantiate_sort_tests!(SortImpl);
