// Vectorized approach by Orson Peters

bucket_sort!("bucket_vectorized");

impl BucketSort for u64 {
    fn sort(v: &mut [Self]) {
        bucket_sort(v);
    }
}

fn bucket_sort(v: &mut [u64]) {
    let mut buckets = [0; 4];
    for chunk in v.chunks(1 << 15) {
        let mut sum = 0;
        for x in chunk {
            let idx = *x % 4;
            sum += 1 << (idx * 16);
        }
        for i in 0..4 {
            buckets[i] += sum & 0xffff;
            sum >>= 16;
        }
    }

    let mut offset = 0;
    for (i, count) in buckets.iter().enumerate() {
        v[offset..offset + count].fill(i as u64); // lookup table.
        offset += count;
    }
}
