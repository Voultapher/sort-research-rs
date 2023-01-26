#[macro_use]
extern crate afl;

use sort_comp::unstable::rust_new as test_sort;

fn main() {
    fuzz!(|data: &[u8]| {
        let mut v = data.to_vec();
        test_sort::sort(&mut v);
    });
}
