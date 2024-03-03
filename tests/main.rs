use sort_test_tools::instantiate_sort_tests;

type TestSort = sort_research_rs::stable::rust_driftsort::SortImpl;

instantiate_sort_tests!(TestSort);
