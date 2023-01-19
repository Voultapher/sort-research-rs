import sys

# import re
import statistics


if __name__ == "__main__":
    with open(sys.argv[1], "r") as file:
        input = file.read()

    bucket_a_min = 50
    bucket_b_min = 512

    bucket_a = []
    bucket_b = []
    bucket_c = []

    for line in input.split("\n"):
        if "is_less" not in line:
            continue

        a, _, b = line.partition("is_less:")
        len = int(a.partition("len:")[2].strip())
        is_less = int(b.strip())
        len_div_2 = len / 2

        # Ideally each partition operation halves the input.
        # Measure how far of that ideal it is on average.
        ideal_overshoot = (
            len_div_2 / is_less
            if is_less < len_div_2
            else len_div_2 / (len - is_less)
        )

        if len < bucket_a_min:
            bucket_a.append(ideal_overshoot)
        elif len < bucket_b_min:
            bucket_b.append(ideal_overshoot)
        else:
            bucket_c.append(ideal_overshoot)

    a_mean = round(statistics.mean(bucket_a), 2)
    b_mean = round(statistics.mean(bucket_b), 2)
    c_mean = round(statistics.mean(bucket_c), 2)

    print(f"a_mean: {a_mean}, b_mean: {b_mean}, c_mean: {c_mean}")
