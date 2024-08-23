mod unstable {
    type TestSort = sort_research_rs::unstable::rust_ipnsort::SortImpl;

    sort_test_tools::instantiate_sort_tests!(TestSort);
}

mod stable {
    type TestSort = sort_research_rs::stable::rust_driftsort::SortImpl;

    sort_test_tools::instantiate_sort_tests!(TestSort);
}
