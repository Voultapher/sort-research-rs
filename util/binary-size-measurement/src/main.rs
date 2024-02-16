use std::env;
use std::hint::black_box;

#[inline(always)]
fn inst<T: Ord>(v: &mut [T]) {
    // v.sort();
    // v.sort_unstable();

    ipnsort::sort(v);
}

#[allow(dead_code, unused_mut)]
#[inline(never)]
fn instantiate_sort<T: Clone + Ord + Eq>(mut v: Vec<T>) {
    #[derive(PartialOrd, Ord, PartialEq, Eq)]
    struct NovelType {
        val: i128, // Explicitly a different type than `T`
    }

    let mut v_baseline_inst = v
        .iter()
        .cloned()
        .map(|val| NovelType {
            val: (&val as *const T) as i128,
        })
        .collect::<Vec<NovelType>>();

    // Avoid falsifying the calc by always constructing `v_baseline_inst` even if inst is proven to
    // panic.
    black_box(&mut v_baseline_inst);

    #[cfg(feature = "sort_inst")]
    {
        inst(&mut v);
    }

    std::hint::black_box(&mut v.as_mut_ptr());

    // side-effect
    if v[0] == v[v.len() - 1] {
        panic!();
    }

    // Baseline inst with novel type to isolate measurement to single instantiation. At the bottom,
    // so that if the compiler can prove that this inst panics it still has to do the upper own.
    inst(&mut v_baseline_inst);
}

#[allow(unused)]
fn main() {
    let len = black_box(env::args().len());

    #[cfg(feature = "type_u64")]
    {
        instantiate_sort::<u64>(black_box((0..len).map(|x| x as u64).collect()));
    }

    #[cfg(feature = "type_string")]
    {
        instantiate_sort::<String>(black_box((0..len).map(|x| format!("{x}")).collect()));
    }
}
