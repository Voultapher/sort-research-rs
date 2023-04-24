vals = """
[(0,2),(1,3)]
[(0,1),(2,3)]
[(1,2)]
"""


def yield_pairs():
    for line in vals.strip().split("\n"):
        for pair in line.replace("[", "").replace("]", ",").split("),"):
            parts = pair[1:].split(",")
            if len(parts) != 2:
                continue

            a = int(parts[0])
            b = int(parts[1])

            yield (a, b)


def print_simple_swap_if(pairs):
    for a, b in pairs:
        print(f"swap_next_if_less(arr_ptr.add({a}), is_less);")


def print_ptr_select(pairs):
    net_size = max(max(a, b) for a, b in pairs) + 1

    for i in range(net_size):
        print(f"let mut val_{i}_ptr = arr_ptr.add({i});")

    for a, b in pairs:
        print(
            f"(val_{a}_ptr, val_{b}_ptr) = cmp_select(val_{a}_ptr, val_{b}_ptr, is_less);"
        )

    print("")

    for i in range(net_size):
        print(f"ptr::copy_nonoverlapping(val_{i}_ptr, dest_ptr.add({i}), 1);")


if __name__ == "__main__":
    pairs = list(yield_pairs())

    # print_simple_swap_if(pairs)
    print_ptr_select(pairs)
