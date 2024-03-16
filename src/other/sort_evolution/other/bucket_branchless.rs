//! Specialized on u64 and match the values branchless.

bucket_sort!("bucket_branchless");

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
        counts[0] += (*val == fixed_bucket_value!(0)) as usize;
        counts[1] += (*val == fixed_bucket_value!(1)) as usize;
        counts[2] += (*val == fixed_bucket_value!(2)) as usize;
        counts[3] += (*val == fixed_bucket_value!(3)) as usize;
    }

    let mut offset = 0;
    for (i, count) in counts.iter().enumerate() {
        v[offset..offset + count].fill(fixed_bucket_value!(i));
        offset += count;
    }
}
