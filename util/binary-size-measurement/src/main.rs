use std::env;
use std::hint::black_box;

#[allow(dead_code)]
#[inline(never)]
fn instantiate_sort<T: Ord>(v: &mut [T]) {
    // v.sort();
    // v.sort_unstable();

    ipnsort::sort(v);
}

fn main() {
    let len = black_box(env::args().len());

    let mut input_u64 = (0..len).map(|x| x as u64).collect::<Vec<_>>();
    black_box(&mut input_u64);

    let mut input_string = (0..len).map(|x| format!("{x}")).collect::<Vec<_>>();
    black_box(&mut input_string);

    // Always instantiate string comparison.
    black_box(input_string[0].cmp(&input_string[1]));

    #[cfg(feature = "sort_inst")]
    {
        #[cfg(feature = "type_u64")]
        {
            instantiate_sort(&mut input_u64);
        }

        #[cfg(feature = "type_string")]
        {
            instantiate_sort(&mut input_string);
        }
    }

    black_box(input_u64); // side-effect
    black_box(input_string); // side-effect
}
