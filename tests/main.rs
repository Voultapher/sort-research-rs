use sort_test_tools::instantiate_sort_tests;

type TestSort = sort_comp::unstable::rust_ipnsort_lomuto_nanosort::SortImpl;

instantiate_sort_tests!(TestSort);
