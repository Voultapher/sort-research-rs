"""
The goal of this file is to create a flush_btb function that will produce code
that flushes the Branch Target Buffer (BTB) regardless of compiler
optimizations.
"""

import random
import itertools

# Perform jumps from this many different locations.
FLUSH_SIZE = 16192


def semi_random_values(seed, start, end):
    """
    Generate values that are not tied logically to the condition values,
    to avoid compiler trickery.
    """

    rng = random.Random(seed)
    while True:
        yield rng.randint(start, end)


FUNC_ID = 0

i32_min = -(2**31)
i32_max = (2**31) - 1

u64_min = 0
u64_max = 2**64 - 1

cond_gen = semi_random_values(6907, u64_min, u64_max)
return_gen = semi_random_values(3251, i32_min, i32_max)


def generate_flush_btb_sub_fn():
    global FUNC_ID

    fn_body = """#[inline(never)]
fn flush_btb_{}(rng_val: u64, input: i32) -> i32 {{
""".format(
        FUNC_ID
    )
    FUNC_ID += 1

    fn_body += "    match rng_val {\n"

    for cond, ret in itertools.islice(zip(cond_gen, return_gen), 7):
        fn_body += f"        {cond} => {ret},\n"

    fn_body += "        _ => input,\n    }\n}\n\n"

    return fn_body


def generate_flush_btb_fn():
    fn_body = """#[inline(never)]
fn flush_btb(rng_val: u64, mut input: i32) -> i32 {
"""

    mod_body = ""

    # We assume on average 4 taken jumps per function.
    for i in range(int(FLUSH_SIZE / 4)):
        mod_body += generate_flush_btb_sub_fn()
        fn_body += f"    input = flush_btb_{i}(rng_val, input);\n"

    fn_body += "    input\n}\n\n"

    return fn_body + mod_body


if __name__ == "__main__":
    flush_btb_fn = generate_flush_btb_fn()
    print(flush_btb_fn)
