use sort_test_tools::instantiate_sort_tests;

type TestSort = sort_research_rs::unstable::rust_ipnsort::SortImpl;

instantiate_sort_tests!(TestSort);
