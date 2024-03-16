//! Specialized on u64 and match the values.

bucket_sort!("bucket_match");

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
        match val {
            fixed_bucket_value!(0) => counts[0] += 1,
            fixed_bucket_value!(1) => counts[1] += 1,
            fixed_bucket_value!(2) => counts[2] += 1,
            fixed_bucket_value!(3) => counts[3] += 1,
            _ => unreachable!("{val}"),
        }
    }

    let mut offset = 0;
    for (i, count) in counts.iter().enumerate() {
        v[offset..offset + count].fill(fixed_bucket_value!(i));
        offset += count;
    }
}
