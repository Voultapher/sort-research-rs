use sort_test_tools::instantiate_sort_tests;

type TestSort = sort_research_rs::unstable::c_llvm_libc::SortImpl;

instantiate_sort_tests!(TestSort);
