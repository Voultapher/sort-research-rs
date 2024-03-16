//! Specialized on u64 and counting sort with perfect hash function.

bucket_sort!("bucket_phf");

impl BucketSort for u64 {
    fn sort(v: &mut [Self]) {
        bucket_sort(v);
    }
}

fn bucket_sort(v: &mut [u64]) {
    let len = v.len();

    if len < 2 {
        return;
    }

    let mut counts = [0; 4];

    for val in v.iter() {
        let idx = 3 - ((val + 3) % 4);
        counts[idx as usize] += 1;
    }

    let mut offset = 0;
    for (i, count) in counts.iter().enumerate() {
        v[offset..offset + count].fill(fixed_bucket_value!(i));
        offset += count;
    }
}
