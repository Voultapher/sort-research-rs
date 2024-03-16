//! Replaces BTreeMap with HashMap.

use std::hash::Hash;
use std::mem::SizedTypeProperties;

use fxhash::FxHashMap;

bucket_sort!("bucket_hash");

impl<T: Clone + Ord + Hash> BucketSort for T {
    fn sort(v: &mut [Self]) {
        bucket_sort(v);
    }
}

fn bucket_sort<T: Clone + Ord + Hash>(v: &mut [T]) {
    let len = v.len();

    if len < 2 || T::IS_ZST {
        return;
    }

    let mut buckets = FxHashMap::<T, usize>::default();

    for elem in v.iter() {
        if let Some(entry) = buckets.get_mut(elem) {
            *entry += 1;
        } else {
            buckets.insert(elem.clone(), 1);
        }
    }

    let mut buckets_sorted = buckets.into_iter().collect::<Vec<_>>();
    buckets_sorted.sort_unstable_by(|a, b| a.0.cmp(&b.0));

    let mut offset = 0;
    for (elem, count) in buckets_sorted {
        v[offset..offset + count].fill(elem);
        offset += count;
    }
}
