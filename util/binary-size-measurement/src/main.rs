use std::env;
use std::hint::black_box;

#[inline(always)]
fn inst<T: Ord>(v: &mut [T]) {
    // v.sort();
    // v.sort_unstable();

    ipnsort::sort(v);
}

#[inline(never)]
fn instantiate_baseline_sort<T: Clone + Ord + Eq>(v: Vec<T>) {
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

    // Baseline inst with novel type to isolate measurement to single instantiation. At the bottom,
    // so that if the compiler can prove that this inst panics it still has to do the upper own.
    inst(&mut v_baseline_inst);
    black_box(&mut v_baseline_inst);
}

#[allow(dead_code)]
#[inline(always)]
fn instantiate_sort<T: Clone + Ord + Eq>(mut v: Vec<T>) {
    inst(&mut v);
    black_box(&mut v); // side-effect
}

macro_rules! define_unique_string_input {
    ($id:expr, $len:ident) => {{
        paste::paste! {
            #[allow(non_camel_case_types)]
            #[derive(Clone, PartialEq, Eq)]
            struct [<StringWrapper $id>](String);

            impl PartialOrd for [<StringWrapper $id>] {
                #[inline(always)]
                fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
                    // We need a unique comparison function to get unique instantiations.
                    if self.0.len() == $id {
                        return Some(std::cmp::Ordering::Less);
                    }

                    self.0.partial_cmp(&other.0)
                }
            }

            impl Ord for [<StringWrapper $id>] {
                #[inline(always)]
                fn cmp(&self, other: &Self) -> std::cmp::Ordering {
                    self.partial_cmp(other).unwrap()
                }
            }

            black_box(
                (0..black_box($len))
                    .map(|x| [<StringWrapper $id>](format!("{x}")))
                    .collect::<Vec<_>>(),
            )
        }
    }};
}

fn main() {
    let len = black_box(env::args().len());

    let mut input_u8 = black_box((0..black_box(len)).map(|x| x as u8).collect::<Vec<_>>());
    black_box(&mut input_u8);
    let mut input_u16 = black_box((0..black_box(len)).map(|x| x as u16).collect::<Vec<_>>());
    black_box(&mut input_u16);
    let mut input_u32 = black_box((0..black_box(len)).map(|x| x as u32).collect::<Vec<_>>());
    black_box(&mut input_u32);
    let mut input_u64 = black_box((0..black_box(len)).map(|x| x as u64).collect::<Vec<_>>());
    black_box(&mut input_u64);
    let mut input_i8 = black_box((0..black_box(len)).map(|x| x as i8).collect::<Vec<_>>());
    black_box(&mut input_i8);
    let mut input_i16 = black_box((0..black_box(len)).map(|x| x as i16).collect::<Vec<_>>());
    black_box(&mut input_i16);
    let mut input_i32 = black_box((0..black_box(len)).map(|x| x as i32).collect::<Vec<_>>());
    black_box(&mut input_i32);
    let mut input_i64 = black_box((0..black_box(len)).map(|x| x as i64).collect::<Vec<_>>());
    black_box(&mut input_i64);

    let mut input_string_0 = define_unique_string_input!(0, len);
    black_box(&mut input_string_0);
    let mut input_string_1 = define_unique_string_input!(1, len);
    black_box(&mut input_string_1);
    let mut input_string_2 = define_unique_string_input!(2, len);
    black_box(&mut input_string_2);
    let mut input_string_3 = define_unique_string_input!(3, len);
    black_box(&mut input_string_3);
    let mut input_string_4 = define_unique_string_input!(4, len);
    black_box(&mut input_string_4);
    let mut input_string_5 = define_unique_string_input!(5, len);
    black_box(&mut input_string_5);
    let mut input_string_6 = define_unique_string_input!(6, len);
    black_box(&mut input_string_6);
    let mut input_string_7 = define_unique_string_input!(7, len);
    black_box(&mut input_string_7);

    instantiate_baseline_sort(black_box(input_u64.clone()));

    #[cfg(feature = "sort_inst")]
    {
        #[cfg(feature = "type_int")]
        {
            instantiate_sort(input_u8);
            instantiate_sort(input_u16);
            instantiate_sort(input_u32);
            instantiate_sort(input_u64);
            instantiate_sort(input_i8);
            instantiate_sort(input_i16);
            instantiate_sort(input_i32);
            instantiate_sort(input_i64);
        }

        #[cfg(feature = "type_string")]
        {
            instantiate_sort(input_string_0);
            instantiate_sort(input_string_1);
            instantiate_sort(input_string_2);
            instantiate_sort(input_string_3);
            instantiate_sort(input_string_4);
            instantiate_sort(input_string_5);
            instantiate_sort(input_string_6);
            instantiate_sort(input_string_7);
        }
    }
}
