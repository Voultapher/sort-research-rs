"""
The goal of this file is to create a rust file that stresses the compile time
impact of using different sort implementations.
"""

import itertools


UNIQUE_TYPE_INSTANTIATIONS = 256
INT_PERCENT = 50.0
STRING_PERCENT = 45.0
CELL_PERCENT = 5.0


def generate_new_int_type(name):
    return (
        f"#[derive(Copy, Clone, Ord, PartialOrd, PartialEq, Eq)]struct {name}(u64);"
    )


def generate_new_string_type(name):
    return f"#[derive(Clone, Ord, PartialOrd, PartialEq, Eq)]struct {name}(String);"


def generate_new_cell_type(name):
    return f"#[derive(Clone, Ord, PartialOrd, PartialEq, Eq)]struct {name}(std::cell::Cell<u64>);"


def type_instantiations(type_percent):
    return int(round(UNIQUE_TYPE_INSTANTIATIONS * (type_percent / 100.0)))


def generate_test_fn(type_defs):
    result = """#![allow(non_snake_case, non_camel_case_types)]

include!("sort_impl_inject.rs");

#[inline(never)]
fn instantiate_test_sort<T: Ord>(v: &mut [T]) {{
    sort(v);
}}

"""

    type_inst_template = """{}

#[inline(never)]
fn instantiate_{}(data_ptr: *mut u8, len: usize) {{
    let v: &mut [{}] = unsafe {{
        &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut {}, len)
    }};

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {{
        panic!(); // side-effect
    }}
}}

"""
    for name, type_def in type_defs:
        # print(name, type_def)
        result += type_inst_template.format(type_def, name, name, name)

    result += """
fn instantiate_all(data_ptr: *mut u8, len: usize) {
"""

    for name, _ in type_defs:
        result += f"    instantiate_{name}(data_ptr, len);\n"

    result += "}\n"

    result += """
fn main() {
    // This is only meant to test compile impact, never run this.

    // source of compiler unpredictable values.
    let data_ptr: *mut u8 = std::hint::black_box(std::ptr::null_mut());
    let len: usize = std::env::args().len();

    instantiate_all(data_ptr, len);
}
"""

    return result


if __name__ == "__main__":
    int_name_ids = list(range(type_instantiations(INT_PERCENT)))
    string_name_ids = list(range(type_instantiations(STRING_PERCENT)))
    cell_name_ids = list(range(type_instantiations(CELL_PERCENT)))

    int_names = [f"U64_{name_id}" for name_id in int_name_ids]
    string_names = [f"String_{name_id}" for name_id in string_name_ids]
    cell_names = [f"Cell_{name_id}" for name_id in cell_name_ids]

    int_types = [generate_new_int_type(name) for name in int_names]
    string_types = [generate_new_string_type(name) for name in string_names]
    cell_types = [generate_new_cell_type(name) for name in cell_names]

    names = itertools.chain.from_iterable(
        [int_names, string_names, cell_names]
    )
    type_defs = itertools.chain.from_iterable(
        [int_types, string_types, cell_types]
    )

    x = generate_test_fn(list(zip(names, type_defs)))
    print(x)
