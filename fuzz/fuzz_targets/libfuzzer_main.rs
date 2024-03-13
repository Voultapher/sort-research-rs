#![no_main]

use libfuzzer_sys::fuzz_target;

use sort_research_rs::unstable::rust_ipnsort as test_sort;

fuzz_target!(|data: &[u8]| {
    let mut v = data.to_vec();
    test_sort::sort(&mut v);
});
