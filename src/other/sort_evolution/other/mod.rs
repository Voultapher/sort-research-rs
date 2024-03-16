macro_rules! bucket_sort {
    ($name:expr) => {
        sort_impl!($name);

        #[inline]
        pub fn sort<T: Ord>(v: &mut [T]) {
            T::sort(v);
        }

        #[inline]
        pub fn sort_by<T, F: FnMut(&T, &T) -> std::cmp::Ordering>(_v: &mut [T], mut _compare: F) {
            panic!("not supported");
        }

        trait BucketSort: Sized {
            fn sort(v: &mut [Self]);
        }

        impl<T> BucketSort for T {
            default fn sort(_v: &mut [Self]) {
                panic!("not supported");
            }
        }
    };
}

macro_rules! fixed_bucket_value {
    (0) => {
        4611686016279904256
    };
    (1) => {
        4611686018427387903
    };
    (2) => {
        4611686020574871550
    };
    (3) => {
        4611686022722355197
    };
    ($idx:expr) => {
        match $idx {
            0 => fixed_bucket_value!(0),
            1 => fixed_bucket_value!(1),
            2 => fixed_bucket_value!(2),
            3 => fixed_bucket_value!(3),
            _ => unreachable!(),
        }
    };
}

pub mod bucket_branchless;
pub mod bucket_btree;
pub mod bucket_hash;
pub mod bucket_match;
pub mod bucket_phf;
