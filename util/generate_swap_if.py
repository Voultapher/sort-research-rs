vals = """
[(0,2),(1,3),(4,6),(5,7)]
[(0,4),(1,5),(2,6),(3,7)]
[(0,1),(2,3),(4,5),(6,7)]
[(2,4),(3,5)]
[(1,4),(3,6)]
[(1,2),(3,4),(5,6)]
"""

for line in vals.strip().split('\n'):
    for pair in line.replace('[', '').replace(']', ',').split("),"):
        parts = pair[1:].split(",")
        if len(parts) != 2:
            continue

        a = int(parts[0])
        b = int(parts[1])

        print(f"swap_if_less(arr_ptr, {a}, {b}, is_less);")
        # print(f"swap_next_if_less(arr_ptr.add({a}), is_less);")
