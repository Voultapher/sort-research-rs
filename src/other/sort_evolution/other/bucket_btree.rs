//! Basic BTreeMap based approach.

use std::collections::BTreeMap;
use std::mem::SizedTypeProperties;

bucket_sort!("bucket_btree");

impl<T: Clone + Ord> BucketSort for T {
    fn sort(v: &mut [Self]) {
        bucket_sort(v);
    }
}

fn bucket_sort<T: Clone + Ord>(v: &mut [T]) {
    let len = v.len();

    if len < 2 || T::IS_ZST {
        return;
    }

    let mut buckets = BTreeMap::<T, usize>::new();

    for elem in v.iter() {
        if let Some(entry) = buckets.get_mut(elem) {
            *entry += 1;
        } else {
            buckets.insert(elem.clone(), 1);
        }
    }

    let mut offset = 0;
    for (elem, count) in buckets {
        v[offset..offset + count].fill(elem);
        offset += count;
    }
}
