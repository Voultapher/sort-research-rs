#![allow(non_snake_case, non_camel_case_types)]

#[inline(never)]
fn instantiate_test_sort<T: Ord>(v: &mut [T]) {
    {
        // v.sort();
        // v.sort_unstable();
        ipnsort::sort(v);
    }
}

#[derive(Copy, Clone, Ord, PartialOrd, PartialEq, Eq)]
struct U64_0(u64);

#[inline(never)]
fn instantiate_U64_0(data_ptr: *mut u8, len: usize) {
    let v: &mut [U64_0] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut U64_0, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Copy, Clone, Ord, PartialOrd, PartialEq, Eq)]
struct U64_1(u64);

#[inline(never)]
fn instantiate_U64_1(data_ptr: *mut u8, len: usize) {
    let v: &mut [U64_1] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut U64_1, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Copy, Clone, Ord, PartialOrd, PartialEq, Eq)]
struct U64_2(u64);

#[inline(never)]
fn instantiate_U64_2(data_ptr: *mut u8, len: usize) {
    let v: &mut [U64_2] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut U64_2, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Copy, Clone, Ord, PartialOrd, PartialEq, Eq)]
struct U64_3(u64);

#[inline(never)]
fn instantiate_U64_3(data_ptr: *mut u8, len: usize) {
    let v: &mut [U64_3] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut U64_3, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Copy, Clone, Ord, PartialOrd, PartialEq, Eq)]
struct U64_4(u64);

#[inline(never)]
fn instantiate_U64_4(data_ptr: *mut u8, len: usize) {
    let v: &mut [U64_4] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut U64_4, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Copy, Clone, Ord, PartialOrd, PartialEq, Eq)]
struct U64_5(u64);

#[inline(never)]
fn instantiate_U64_5(data_ptr: *mut u8, len: usize) {
    let v: &mut [U64_5] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut U64_5, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Copy, Clone, Ord, PartialOrd, PartialEq, Eq)]
struct U64_6(u64);

#[inline(never)]
fn instantiate_U64_6(data_ptr: *mut u8, len: usize) {
    let v: &mut [U64_6] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut U64_6, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Copy, Clone, Ord, PartialOrd, PartialEq, Eq)]
struct U64_7(u64);

#[inline(never)]
fn instantiate_U64_7(data_ptr: *mut u8, len: usize) {
    let v: &mut [U64_7] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut U64_7, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Copy, Clone, Ord, PartialOrd, PartialEq, Eq)]
struct U64_8(u64);

#[inline(never)]
fn instantiate_U64_8(data_ptr: *mut u8, len: usize) {
    let v: &mut [U64_8] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut U64_8, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Copy, Clone, Ord, PartialOrd, PartialEq, Eq)]
struct U64_9(u64);

#[inline(never)]
fn instantiate_U64_9(data_ptr: *mut u8, len: usize) {
    let v: &mut [U64_9] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut U64_9, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Copy, Clone, Ord, PartialOrd, PartialEq, Eq)]
struct U64_10(u64);

#[inline(never)]
fn instantiate_U64_10(data_ptr: *mut u8, len: usize) {
    let v: &mut [U64_10] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut U64_10, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Copy, Clone, Ord, PartialOrd, PartialEq, Eq)]
struct U64_11(u64);

#[inline(never)]
fn instantiate_U64_11(data_ptr: *mut u8, len: usize) {
    let v: &mut [U64_11] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut U64_11, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Copy, Clone, Ord, PartialOrd, PartialEq, Eq)]
struct U64_12(u64);

#[inline(never)]
fn instantiate_U64_12(data_ptr: *mut u8, len: usize) {
    let v: &mut [U64_12] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut U64_12, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Copy, Clone, Ord, PartialOrd, PartialEq, Eq)]
struct U64_13(u64);

#[inline(never)]
fn instantiate_U64_13(data_ptr: *mut u8, len: usize) {
    let v: &mut [U64_13] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut U64_13, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Copy, Clone, Ord, PartialOrd, PartialEq, Eq)]
struct U64_14(u64);

#[inline(never)]
fn instantiate_U64_14(data_ptr: *mut u8, len: usize) {
    let v: &mut [U64_14] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut U64_14, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Copy, Clone, Ord, PartialOrd, PartialEq, Eq)]
struct U64_15(u64);

#[inline(never)]
fn instantiate_U64_15(data_ptr: *mut u8, len: usize) {
    let v: &mut [U64_15] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut U64_15, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Copy, Clone, Ord, PartialOrd, PartialEq, Eq)]
struct U64_16(u64);

#[inline(never)]
fn instantiate_U64_16(data_ptr: *mut u8, len: usize) {
    let v: &mut [U64_16] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut U64_16, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Copy, Clone, Ord, PartialOrd, PartialEq, Eq)]
struct U64_17(u64);

#[inline(never)]
fn instantiate_U64_17(data_ptr: *mut u8, len: usize) {
    let v: &mut [U64_17] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut U64_17, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Copy, Clone, Ord, PartialOrd, PartialEq, Eq)]
struct U64_18(u64);

#[inline(never)]
fn instantiate_U64_18(data_ptr: *mut u8, len: usize) {
    let v: &mut [U64_18] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut U64_18, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Copy, Clone, Ord, PartialOrd, PartialEq, Eq)]
struct U64_19(u64);

#[inline(never)]
fn instantiate_U64_19(data_ptr: *mut u8, len: usize) {
    let v: &mut [U64_19] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut U64_19, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Copy, Clone, Ord, PartialOrd, PartialEq, Eq)]
struct U64_20(u64);

#[inline(never)]
fn instantiate_U64_20(data_ptr: *mut u8, len: usize) {
    let v: &mut [U64_20] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut U64_20, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Copy, Clone, Ord, PartialOrd, PartialEq, Eq)]
struct U64_21(u64);

#[inline(never)]
fn instantiate_U64_21(data_ptr: *mut u8, len: usize) {
    let v: &mut [U64_21] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut U64_21, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Copy, Clone, Ord, PartialOrd, PartialEq, Eq)]
struct U64_22(u64);

#[inline(never)]
fn instantiate_U64_22(data_ptr: *mut u8, len: usize) {
    let v: &mut [U64_22] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut U64_22, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Copy, Clone, Ord, PartialOrd, PartialEq, Eq)]
struct U64_23(u64);

#[inline(never)]
fn instantiate_U64_23(data_ptr: *mut u8, len: usize) {
    let v: &mut [U64_23] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut U64_23, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Copy, Clone, Ord, PartialOrd, PartialEq, Eq)]
struct U64_24(u64);

#[inline(never)]
fn instantiate_U64_24(data_ptr: *mut u8, len: usize) {
    let v: &mut [U64_24] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut U64_24, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Copy, Clone, Ord, PartialOrd, PartialEq, Eq)]
struct U64_25(u64);

#[inline(never)]
fn instantiate_U64_25(data_ptr: *mut u8, len: usize) {
    let v: &mut [U64_25] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut U64_25, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Copy, Clone, Ord, PartialOrd, PartialEq, Eq)]
struct U64_26(u64);

#[inline(never)]
fn instantiate_U64_26(data_ptr: *mut u8, len: usize) {
    let v: &mut [U64_26] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut U64_26, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Copy, Clone, Ord, PartialOrd, PartialEq, Eq)]
struct U64_27(u64);

#[inline(never)]
fn instantiate_U64_27(data_ptr: *mut u8, len: usize) {
    let v: &mut [U64_27] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut U64_27, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Copy, Clone, Ord, PartialOrd, PartialEq, Eq)]
struct U64_28(u64);

#[inline(never)]
fn instantiate_U64_28(data_ptr: *mut u8, len: usize) {
    let v: &mut [U64_28] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut U64_28, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Copy, Clone, Ord, PartialOrd, PartialEq, Eq)]
struct U64_29(u64);

#[inline(never)]
fn instantiate_U64_29(data_ptr: *mut u8, len: usize) {
    let v: &mut [U64_29] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut U64_29, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Copy, Clone, Ord, PartialOrd, PartialEq, Eq)]
struct U64_30(u64);

#[inline(never)]
fn instantiate_U64_30(data_ptr: *mut u8, len: usize) {
    let v: &mut [U64_30] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut U64_30, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Copy, Clone, Ord, PartialOrd, PartialEq, Eq)]
struct U64_31(u64);

#[inline(never)]
fn instantiate_U64_31(data_ptr: *mut u8, len: usize) {
    let v: &mut [U64_31] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut U64_31, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Copy, Clone, Ord, PartialOrd, PartialEq, Eq)]
struct U64_32(u64);

#[inline(never)]
fn instantiate_U64_32(data_ptr: *mut u8, len: usize) {
    let v: &mut [U64_32] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut U64_32, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Copy, Clone, Ord, PartialOrd, PartialEq, Eq)]
struct U64_33(u64);

#[inline(never)]
fn instantiate_U64_33(data_ptr: *mut u8, len: usize) {
    let v: &mut [U64_33] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut U64_33, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Copy, Clone, Ord, PartialOrd, PartialEq, Eq)]
struct U64_34(u64);

#[inline(never)]
fn instantiate_U64_34(data_ptr: *mut u8, len: usize) {
    let v: &mut [U64_34] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut U64_34, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Copy, Clone, Ord, PartialOrd, PartialEq, Eq)]
struct U64_35(u64);

#[inline(never)]
fn instantiate_U64_35(data_ptr: *mut u8, len: usize) {
    let v: &mut [U64_35] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut U64_35, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Copy, Clone, Ord, PartialOrd, PartialEq, Eq)]
struct U64_36(u64);

#[inline(never)]
fn instantiate_U64_36(data_ptr: *mut u8, len: usize) {
    let v: &mut [U64_36] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut U64_36, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Copy, Clone, Ord, PartialOrd, PartialEq, Eq)]
struct U64_37(u64);

#[inline(never)]
fn instantiate_U64_37(data_ptr: *mut u8, len: usize) {
    let v: &mut [U64_37] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut U64_37, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Copy, Clone, Ord, PartialOrd, PartialEq, Eq)]
struct U64_38(u64);

#[inline(never)]
fn instantiate_U64_38(data_ptr: *mut u8, len: usize) {
    let v: &mut [U64_38] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut U64_38, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Copy, Clone, Ord, PartialOrd, PartialEq, Eq)]
struct U64_39(u64);

#[inline(never)]
fn instantiate_U64_39(data_ptr: *mut u8, len: usize) {
    let v: &mut [U64_39] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut U64_39, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Copy, Clone, Ord, PartialOrd, PartialEq, Eq)]
struct U64_40(u64);

#[inline(never)]
fn instantiate_U64_40(data_ptr: *mut u8, len: usize) {
    let v: &mut [U64_40] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut U64_40, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Copy, Clone, Ord, PartialOrd, PartialEq, Eq)]
struct U64_41(u64);

#[inline(never)]
fn instantiate_U64_41(data_ptr: *mut u8, len: usize) {
    let v: &mut [U64_41] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut U64_41, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Copy, Clone, Ord, PartialOrd, PartialEq, Eq)]
struct U64_42(u64);

#[inline(never)]
fn instantiate_U64_42(data_ptr: *mut u8, len: usize) {
    let v: &mut [U64_42] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut U64_42, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Copy, Clone, Ord, PartialOrd, PartialEq, Eq)]
struct U64_43(u64);

#[inline(never)]
fn instantiate_U64_43(data_ptr: *mut u8, len: usize) {
    let v: &mut [U64_43] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut U64_43, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Copy, Clone, Ord, PartialOrd, PartialEq, Eq)]
struct U64_44(u64);

#[inline(never)]
fn instantiate_U64_44(data_ptr: *mut u8, len: usize) {
    let v: &mut [U64_44] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut U64_44, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Copy, Clone, Ord, PartialOrd, PartialEq, Eq)]
struct U64_45(u64);

#[inline(never)]
fn instantiate_U64_45(data_ptr: *mut u8, len: usize) {
    let v: &mut [U64_45] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut U64_45, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Copy, Clone, Ord, PartialOrd, PartialEq, Eq)]
struct U64_46(u64);

#[inline(never)]
fn instantiate_U64_46(data_ptr: *mut u8, len: usize) {
    let v: &mut [U64_46] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut U64_46, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Copy, Clone, Ord, PartialOrd, PartialEq, Eq)]
struct U64_47(u64);

#[inline(never)]
fn instantiate_U64_47(data_ptr: *mut u8, len: usize) {
    let v: &mut [U64_47] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut U64_47, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Copy, Clone, Ord, PartialOrd, PartialEq, Eq)]
struct U64_48(u64);

#[inline(never)]
fn instantiate_U64_48(data_ptr: *mut u8, len: usize) {
    let v: &mut [U64_48] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut U64_48, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Copy, Clone, Ord, PartialOrd, PartialEq, Eq)]
struct U64_49(u64);

#[inline(never)]
fn instantiate_U64_49(data_ptr: *mut u8, len: usize) {
    let v: &mut [U64_49] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut U64_49, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Copy, Clone, Ord, PartialOrd, PartialEq, Eq)]
struct U64_50(u64);

#[inline(never)]
fn instantiate_U64_50(data_ptr: *mut u8, len: usize) {
    let v: &mut [U64_50] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut U64_50, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Copy, Clone, Ord, PartialOrd, PartialEq, Eq)]
struct U64_51(u64);

#[inline(never)]
fn instantiate_U64_51(data_ptr: *mut u8, len: usize) {
    let v: &mut [U64_51] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut U64_51, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Copy, Clone, Ord, PartialOrd, PartialEq, Eq)]
struct U64_52(u64);

#[inline(never)]
fn instantiate_U64_52(data_ptr: *mut u8, len: usize) {
    let v: &mut [U64_52] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut U64_52, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Copy, Clone, Ord, PartialOrd, PartialEq, Eq)]
struct U64_53(u64);

#[inline(never)]
fn instantiate_U64_53(data_ptr: *mut u8, len: usize) {
    let v: &mut [U64_53] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut U64_53, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Copy, Clone, Ord, PartialOrd, PartialEq, Eq)]
struct U64_54(u64);

#[inline(never)]
fn instantiate_U64_54(data_ptr: *mut u8, len: usize) {
    let v: &mut [U64_54] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut U64_54, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Copy, Clone, Ord, PartialOrd, PartialEq, Eq)]
struct U64_55(u64);

#[inline(never)]
fn instantiate_U64_55(data_ptr: *mut u8, len: usize) {
    let v: &mut [U64_55] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut U64_55, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Copy, Clone, Ord, PartialOrd, PartialEq, Eq)]
struct U64_56(u64);

#[inline(never)]
fn instantiate_U64_56(data_ptr: *mut u8, len: usize) {
    let v: &mut [U64_56] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut U64_56, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Copy, Clone, Ord, PartialOrd, PartialEq, Eq)]
struct U64_57(u64);

#[inline(never)]
fn instantiate_U64_57(data_ptr: *mut u8, len: usize) {
    let v: &mut [U64_57] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut U64_57, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Copy, Clone, Ord, PartialOrd, PartialEq, Eq)]
struct U64_58(u64);

#[inline(never)]
fn instantiate_U64_58(data_ptr: *mut u8, len: usize) {
    let v: &mut [U64_58] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut U64_58, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Copy, Clone, Ord, PartialOrd, PartialEq, Eq)]
struct U64_59(u64);

#[inline(never)]
fn instantiate_U64_59(data_ptr: *mut u8, len: usize) {
    let v: &mut [U64_59] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut U64_59, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Copy, Clone, Ord, PartialOrd, PartialEq, Eq)]
struct U64_60(u64);

#[inline(never)]
fn instantiate_U64_60(data_ptr: *mut u8, len: usize) {
    let v: &mut [U64_60] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut U64_60, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Copy, Clone, Ord, PartialOrd, PartialEq, Eq)]
struct U64_61(u64);

#[inline(never)]
fn instantiate_U64_61(data_ptr: *mut u8, len: usize) {
    let v: &mut [U64_61] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut U64_61, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Copy, Clone, Ord, PartialOrd, PartialEq, Eq)]
struct U64_62(u64);

#[inline(never)]
fn instantiate_U64_62(data_ptr: *mut u8, len: usize) {
    let v: &mut [U64_62] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut U64_62, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Copy, Clone, Ord, PartialOrd, PartialEq, Eq)]
struct U64_63(u64);

#[inline(never)]
fn instantiate_U64_63(data_ptr: *mut u8, len: usize) {
    let v: &mut [U64_63] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut U64_63, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Copy, Clone, Ord, PartialOrd, PartialEq, Eq)]
struct U64_64(u64);

#[inline(never)]
fn instantiate_U64_64(data_ptr: *mut u8, len: usize) {
    let v: &mut [U64_64] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut U64_64, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Copy, Clone, Ord, PartialOrd, PartialEq, Eq)]
struct U64_65(u64);

#[inline(never)]
fn instantiate_U64_65(data_ptr: *mut u8, len: usize) {
    let v: &mut [U64_65] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut U64_65, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Copy, Clone, Ord, PartialOrd, PartialEq, Eq)]
struct U64_66(u64);

#[inline(never)]
fn instantiate_U64_66(data_ptr: *mut u8, len: usize) {
    let v: &mut [U64_66] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut U64_66, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Copy, Clone, Ord, PartialOrd, PartialEq, Eq)]
struct U64_67(u64);

#[inline(never)]
fn instantiate_U64_67(data_ptr: *mut u8, len: usize) {
    let v: &mut [U64_67] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut U64_67, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Copy, Clone, Ord, PartialOrd, PartialEq, Eq)]
struct U64_68(u64);

#[inline(never)]
fn instantiate_U64_68(data_ptr: *mut u8, len: usize) {
    let v: &mut [U64_68] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut U64_68, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Copy, Clone, Ord, PartialOrd, PartialEq, Eq)]
struct U64_69(u64);

#[inline(never)]
fn instantiate_U64_69(data_ptr: *mut u8, len: usize) {
    let v: &mut [U64_69] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut U64_69, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Copy, Clone, Ord, PartialOrd, PartialEq, Eq)]
struct U64_70(u64);

#[inline(never)]
fn instantiate_U64_70(data_ptr: *mut u8, len: usize) {
    let v: &mut [U64_70] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut U64_70, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Copy, Clone, Ord, PartialOrd, PartialEq, Eq)]
struct U64_71(u64);

#[inline(never)]
fn instantiate_U64_71(data_ptr: *mut u8, len: usize) {
    let v: &mut [U64_71] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut U64_71, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Copy, Clone, Ord, PartialOrd, PartialEq, Eq)]
struct U64_72(u64);

#[inline(never)]
fn instantiate_U64_72(data_ptr: *mut u8, len: usize) {
    let v: &mut [U64_72] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut U64_72, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Copy, Clone, Ord, PartialOrd, PartialEq, Eq)]
struct U64_73(u64);

#[inline(never)]
fn instantiate_U64_73(data_ptr: *mut u8, len: usize) {
    let v: &mut [U64_73] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut U64_73, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Copy, Clone, Ord, PartialOrd, PartialEq, Eq)]
struct U64_74(u64);

#[inline(never)]
fn instantiate_U64_74(data_ptr: *mut u8, len: usize) {
    let v: &mut [U64_74] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut U64_74, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Copy, Clone, Ord, PartialOrd, PartialEq, Eq)]
struct U64_75(u64);

#[inline(never)]
fn instantiate_U64_75(data_ptr: *mut u8, len: usize) {
    let v: &mut [U64_75] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut U64_75, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Copy, Clone, Ord, PartialOrd, PartialEq, Eq)]
struct U64_76(u64);

#[inline(never)]
fn instantiate_U64_76(data_ptr: *mut u8, len: usize) {
    let v: &mut [U64_76] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut U64_76, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Copy, Clone, Ord, PartialOrd, PartialEq, Eq)]
struct U64_77(u64);

#[inline(never)]
fn instantiate_U64_77(data_ptr: *mut u8, len: usize) {
    let v: &mut [U64_77] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut U64_77, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Copy, Clone, Ord, PartialOrd, PartialEq, Eq)]
struct U64_78(u64);

#[inline(never)]
fn instantiate_U64_78(data_ptr: *mut u8, len: usize) {
    let v: &mut [U64_78] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut U64_78, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Copy, Clone, Ord, PartialOrd, PartialEq, Eq)]
struct U64_79(u64);

#[inline(never)]
fn instantiate_U64_79(data_ptr: *mut u8, len: usize) {
    let v: &mut [U64_79] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut U64_79, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Copy, Clone, Ord, PartialOrd, PartialEq, Eq)]
struct U64_80(u64);

#[inline(never)]
fn instantiate_U64_80(data_ptr: *mut u8, len: usize) {
    let v: &mut [U64_80] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut U64_80, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Copy, Clone, Ord, PartialOrd, PartialEq, Eq)]
struct U64_81(u64);

#[inline(never)]
fn instantiate_U64_81(data_ptr: *mut u8, len: usize) {
    let v: &mut [U64_81] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut U64_81, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Copy, Clone, Ord, PartialOrd, PartialEq, Eq)]
struct U64_82(u64);

#[inline(never)]
fn instantiate_U64_82(data_ptr: *mut u8, len: usize) {
    let v: &mut [U64_82] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut U64_82, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Copy, Clone, Ord, PartialOrd, PartialEq, Eq)]
struct U64_83(u64);

#[inline(never)]
fn instantiate_U64_83(data_ptr: *mut u8, len: usize) {
    let v: &mut [U64_83] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut U64_83, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Copy, Clone, Ord, PartialOrd, PartialEq, Eq)]
struct U64_84(u64);

#[inline(never)]
fn instantiate_U64_84(data_ptr: *mut u8, len: usize) {
    let v: &mut [U64_84] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut U64_84, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Copy, Clone, Ord, PartialOrd, PartialEq, Eq)]
struct U64_85(u64);

#[inline(never)]
fn instantiate_U64_85(data_ptr: *mut u8, len: usize) {
    let v: &mut [U64_85] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut U64_85, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Copy, Clone, Ord, PartialOrd, PartialEq, Eq)]
struct U64_86(u64);

#[inline(never)]
fn instantiate_U64_86(data_ptr: *mut u8, len: usize) {
    let v: &mut [U64_86] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut U64_86, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Copy, Clone, Ord, PartialOrd, PartialEq, Eq)]
struct U64_87(u64);

#[inline(never)]
fn instantiate_U64_87(data_ptr: *mut u8, len: usize) {
    let v: &mut [U64_87] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut U64_87, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Copy, Clone, Ord, PartialOrd, PartialEq, Eq)]
struct U64_88(u64);

#[inline(never)]
fn instantiate_U64_88(data_ptr: *mut u8, len: usize) {
    let v: &mut [U64_88] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut U64_88, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Copy, Clone, Ord, PartialOrd, PartialEq, Eq)]
struct U64_89(u64);

#[inline(never)]
fn instantiate_U64_89(data_ptr: *mut u8, len: usize) {
    let v: &mut [U64_89] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut U64_89, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Copy, Clone, Ord, PartialOrd, PartialEq, Eq)]
struct U64_90(u64);

#[inline(never)]
fn instantiate_U64_90(data_ptr: *mut u8, len: usize) {
    let v: &mut [U64_90] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut U64_90, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Copy, Clone, Ord, PartialOrd, PartialEq, Eq)]
struct U64_91(u64);

#[inline(never)]
fn instantiate_U64_91(data_ptr: *mut u8, len: usize) {
    let v: &mut [U64_91] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut U64_91, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Copy, Clone, Ord, PartialOrd, PartialEq, Eq)]
struct U64_92(u64);

#[inline(never)]
fn instantiate_U64_92(data_ptr: *mut u8, len: usize) {
    let v: &mut [U64_92] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut U64_92, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Copy, Clone, Ord, PartialOrd, PartialEq, Eq)]
struct U64_93(u64);

#[inline(never)]
fn instantiate_U64_93(data_ptr: *mut u8, len: usize) {
    let v: &mut [U64_93] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut U64_93, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Copy, Clone, Ord, PartialOrd, PartialEq, Eq)]
struct U64_94(u64);

#[inline(never)]
fn instantiate_U64_94(data_ptr: *mut u8, len: usize) {
    let v: &mut [U64_94] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut U64_94, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Copy, Clone, Ord, PartialOrd, PartialEq, Eq)]
struct U64_95(u64);

#[inline(never)]
fn instantiate_U64_95(data_ptr: *mut u8, len: usize) {
    let v: &mut [U64_95] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut U64_95, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Copy, Clone, Ord, PartialOrd, PartialEq, Eq)]
struct U64_96(u64);

#[inline(never)]
fn instantiate_U64_96(data_ptr: *mut u8, len: usize) {
    let v: &mut [U64_96] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut U64_96, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Copy, Clone, Ord, PartialOrd, PartialEq, Eq)]
struct U64_97(u64);

#[inline(never)]
fn instantiate_U64_97(data_ptr: *mut u8, len: usize) {
    let v: &mut [U64_97] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut U64_97, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Copy, Clone, Ord, PartialOrd, PartialEq, Eq)]
struct U64_98(u64);

#[inline(never)]
fn instantiate_U64_98(data_ptr: *mut u8, len: usize) {
    let v: &mut [U64_98] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut U64_98, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Copy, Clone, Ord, PartialOrd, PartialEq, Eq)]
struct U64_99(u64);

#[inline(never)]
fn instantiate_U64_99(data_ptr: *mut u8, len: usize) {
    let v: &mut [U64_99] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut U64_99, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Copy, Clone, Ord, PartialOrd, PartialEq, Eq)]
struct U64_100(u64);

#[inline(never)]
fn instantiate_U64_100(data_ptr: *mut u8, len: usize) {
    let v: &mut [U64_100] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut U64_100, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Copy, Clone, Ord, PartialOrd, PartialEq, Eq)]
struct U64_101(u64);

#[inline(never)]
fn instantiate_U64_101(data_ptr: *mut u8, len: usize) {
    let v: &mut [U64_101] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut U64_101, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Copy, Clone, Ord, PartialOrd, PartialEq, Eq)]
struct U64_102(u64);

#[inline(never)]
fn instantiate_U64_102(data_ptr: *mut u8, len: usize) {
    let v: &mut [U64_102] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut U64_102, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Copy, Clone, Ord, PartialOrd, PartialEq, Eq)]
struct U64_103(u64);

#[inline(never)]
fn instantiate_U64_103(data_ptr: *mut u8, len: usize) {
    let v: &mut [U64_103] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut U64_103, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Copy, Clone, Ord, PartialOrd, PartialEq, Eq)]
struct U64_104(u64);

#[inline(never)]
fn instantiate_U64_104(data_ptr: *mut u8, len: usize) {
    let v: &mut [U64_104] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut U64_104, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Copy, Clone, Ord, PartialOrd, PartialEq, Eq)]
struct U64_105(u64);

#[inline(never)]
fn instantiate_U64_105(data_ptr: *mut u8, len: usize) {
    let v: &mut [U64_105] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut U64_105, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Copy, Clone, Ord, PartialOrd, PartialEq, Eq)]
struct U64_106(u64);

#[inline(never)]
fn instantiate_U64_106(data_ptr: *mut u8, len: usize) {
    let v: &mut [U64_106] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut U64_106, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Copy, Clone, Ord, PartialOrd, PartialEq, Eq)]
struct U64_107(u64);

#[inline(never)]
fn instantiate_U64_107(data_ptr: *mut u8, len: usize) {
    let v: &mut [U64_107] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut U64_107, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Copy, Clone, Ord, PartialOrd, PartialEq, Eq)]
struct U64_108(u64);

#[inline(never)]
fn instantiate_U64_108(data_ptr: *mut u8, len: usize) {
    let v: &mut [U64_108] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut U64_108, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Copy, Clone, Ord, PartialOrd, PartialEq, Eq)]
struct U64_109(u64);

#[inline(never)]
fn instantiate_U64_109(data_ptr: *mut u8, len: usize) {
    let v: &mut [U64_109] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut U64_109, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Copy, Clone, Ord, PartialOrd, PartialEq, Eq)]
struct U64_110(u64);

#[inline(never)]
fn instantiate_U64_110(data_ptr: *mut u8, len: usize) {
    let v: &mut [U64_110] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut U64_110, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Copy, Clone, Ord, PartialOrd, PartialEq, Eq)]
struct U64_111(u64);

#[inline(never)]
fn instantiate_U64_111(data_ptr: *mut u8, len: usize) {
    let v: &mut [U64_111] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut U64_111, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Copy, Clone, Ord, PartialOrd, PartialEq, Eq)]
struct U64_112(u64);

#[inline(never)]
fn instantiate_U64_112(data_ptr: *mut u8, len: usize) {
    let v: &mut [U64_112] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut U64_112, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Copy, Clone, Ord, PartialOrd, PartialEq, Eq)]
struct U64_113(u64);

#[inline(never)]
fn instantiate_U64_113(data_ptr: *mut u8, len: usize) {
    let v: &mut [U64_113] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut U64_113, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Copy, Clone, Ord, PartialOrd, PartialEq, Eq)]
struct U64_114(u64);

#[inline(never)]
fn instantiate_U64_114(data_ptr: *mut u8, len: usize) {
    let v: &mut [U64_114] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut U64_114, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Copy, Clone, Ord, PartialOrd, PartialEq, Eq)]
struct U64_115(u64);

#[inline(never)]
fn instantiate_U64_115(data_ptr: *mut u8, len: usize) {
    let v: &mut [U64_115] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut U64_115, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Copy, Clone, Ord, PartialOrd, PartialEq, Eq)]
struct U64_116(u64);

#[inline(never)]
fn instantiate_U64_116(data_ptr: *mut u8, len: usize) {
    let v: &mut [U64_116] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut U64_116, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Copy, Clone, Ord, PartialOrd, PartialEq, Eq)]
struct U64_117(u64);

#[inline(never)]
fn instantiate_U64_117(data_ptr: *mut u8, len: usize) {
    let v: &mut [U64_117] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut U64_117, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Copy, Clone, Ord, PartialOrd, PartialEq, Eq)]
struct U64_118(u64);

#[inline(never)]
fn instantiate_U64_118(data_ptr: *mut u8, len: usize) {
    let v: &mut [U64_118] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut U64_118, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Copy, Clone, Ord, PartialOrd, PartialEq, Eq)]
struct U64_119(u64);

#[inline(never)]
fn instantiate_U64_119(data_ptr: *mut u8, len: usize) {
    let v: &mut [U64_119] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut U64_119, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Copy, Clone, Ord, PartialOrd, PartialEq, Eq)]
struct U64_120(u64);

#[inline(never)]
fn instantiate_U64_120(data_ptr: *mut u8, len: usize) {
    let v: &mut [U64_120] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut U64_120, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Copy, Clone, Ord, PartialOrd, PartialEq, Eq)]
struct U64_121(u64);

#[inline(never)]
fn instantiate_U64_121(data_ptr: *mut u8, len: usize) {
    let v: &mut [U64_121] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut U64_121, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Copy, Clone, Ord, PartialOrd, PartialEq, Eq)]
struct U64_122(u64);

#[inline(never)]
fn instantiate_U64_122(data_ptr: *mut u8, len: usize) {
    let v: &mut [U64_122] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut U64_122, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Copy, Clone, Ord, PartialOrd, PartialEq, Eq)]
struct U64_123(u64);

#[inline(never)]
fn instantiate_U64_123(data_ptr: *mut u8, len: usize) {
    let v: &mut [U64_123] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut U64_123, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Copy, Clone, Ord, PartialOrd, PartialEq, Eq)]
struct U64_124(u64);

#[inline(never)]
fn instantiate_U64_124(data_ptr: *mut u8, len: usize) {
    let v: &mut [U64_124] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut U64_124, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Copy, Clone, Ord, PartialOrd, PartialEq, Eq)]
struct U64_125(u64);

#[inline(never)]
fn instantiate_U64_125(data_ptr: *mut u8, len: usize) {
    let v: &mut [U64_125] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut U64_125, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Copy, Clone, Ord, PartialOrd, PartialEq, Eq)]
struct U64_126(u64);

#[inline(never)]
fn instantiate_U64_126(data_ptr: *mut u8, len: usize) {
    let v: &mut [U64_126] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut U64_126, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Copy, Clone, Ord, PartialOrd, PartialEq, Eq)]
struct U64_127(u64);

#[inline(never)]
fn instantiate_U64_127(data_ptr: *mut u8, len: usize) {
    let v: &mut [U64_127] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut U64_127, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Clone, Ord, PartialOrd, PartialEq, Eq)]
struct String_0(String);

#[inline(never)]
fn instantiate_String_0(data_ptr: *mut u8, len: usize) {
    let v: &mut [String_0] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut String_0, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Clone, Ord, PartialOrd, PartialEq, Eq)]
struct String_1(String);

#[inline(never)]
fn instantiate_String_1(data_ptr: *mut u8, len: usize) {
    let v: &mut [String_1] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut String_1, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Clone, Ord, PartialOrd, PartialEq, Eq)]
struct String_2(String);

#[inline(never)]
fn instantiate_String_2(data_ptr: *mut u8, len: usize) {
    let v: &mut [String_2] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut String_2, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Clone, Ord, PartialOrd, PartialEq, Eq)]
struct String_3(String);

#[inline(never)]
fn instantiate_String_3(data_ptr: *mut u8, len: usize) {
    let v: &mut [String_3] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut String_3, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Clone, Ord, PartialOrd, PartialEq, Eq)]
struct String_4(String);

#[inline(never)]
fn instantiate_String_4(data_ptr: *mut u8, len: usize) {
    let v: &mut [String_4] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut String_4, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Clone, Ord, PartialOrd, PartialEq, Eq)]
struct String_5(String);

#[inline(never)]
fn instantiate_String_5(data_ptr: *mut u8, len: usize) {
    let v: &mut [String_5] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut String_5, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Clone, Ord, PartialOrd, PartialEq, Eq)]
struct String_6(String);

#[inline(never)]
fn instantiate_String_6(data_ptr: *mut u8, len: usize) {
    let v: &mut [String_6] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut String_6, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Clone, Ord, PartialOrd, PartialEq, Eq)]
struct String_7(String);

#[inline(never)]
fn instantiate_String_7(data_ptr: *mut u8, len: usize) {
    let v: &mut [String_7] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut String_7, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Clone, Ord, PartialOrd, PartialEq, Eq)]
struct String_8(String);

#[inline(never)]
fn instantiate_String_8(data_ptr: *mut u8, len: usize) {
    let v: &mut [String_8] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut String_8, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Clone, Ord, PartialOrd, PartialEq, Eq)]
struct String_9(String);

#[inline(never)]
fn instantiate_String_9(data_ptr: *mut u8, len: usize) {
    let v: &mut [String_9] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut String_9, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Clone, Ord, PartialOrd, PartialEq, Eq)]
struct String_10(String);

#[inline(never)]
fn instantiate_String_10(data_ptr: *mut u8, len: usize) {
    let v: &mut [String_10] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut String_10, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Clone, Ord, PartialOrd, PartialEq, Eq)]
struct String_11(String);

#[inline(never)]
fn instantiate_String_11(data_ptr: *mut u8, len: usize) {
    let v: &mut [String_11] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut String_11, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Clone, Ord, PartialOrd, PartialEq, Eq)]
struct String_12(String);

#[inline(never)]
fn instantiate_String_12(data_ptr: *mut u8, len: usize) {
    let v: &mut [String_12] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut String_12, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Clone, Ord, PartialOrd, PartialEq, Eq)]
struct String_13(String);

#[inline(never)]
fn instantiate_String_13(data_ptr: *mut u8, len: usize) {
    let v: &mut [String_13] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut String_13, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Clone, Ord, PartialOrd, PartialEq, Eq)]
struct String_14(String);

#[inline(never)]
fn instantiate_String_14(data_ptr: *mut u8, len: usize) {
    let v: &mut [String_14] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut String_14, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Clone, Ord, PartialOrd, PartialEq, Eq)]
struct String_15(String);

#[inline(never)]
fn instantiate_String_15(data_ptr: *mut u8, len: usize) {
    let v: &mut [String_15] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut String_15, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Clone, Ord, PartialOrd, PartialEq, Eq)]
struct String_16(String);

#[inline(never)]
fn instantiate_String_16(data_ptr: *mut u8, len: usize) {
    let v: &mut [String_16] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut String_16, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Clone, Ord, PartialOrd, PartialEq, Eq)]
struct String_17(String);

#[inline(never)]
fn instantiate_String_17(data_ptr: *mut u8, len: usize) {
    let v: &mut [String_17] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut String_17, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Clone, Ord, PartialOrd, PartialEq, Eq)]
struct String_18(String);

#[inline(never)]
fn instantiate_String_18(data_ptr: *mut u8, len: usize) {
    let v: &mut [String_18] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut String_18, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Clone, Ord, PartialOrd, PartialEq, Eq)]
struct String_19(String);

#[inline(never)]
fn instantiate_String_19(data_ptr: *mut u8, len: usize) {
    let v: &mut [String_19] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut String_19, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Clone, Ord, PartialOrd, PartialEq, Eq)]
struct String_20(String);

#[inline(never)]
fn instantiate_String_20(data_ptr: *mut u8, len: usize) {
    let v: &mut [String_20] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut String_20, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Clone, Ord, PartialOrd, PartialEq, Eq)]
struct String_21(String);

#[inline(never)]
fn instantiate_String_21(data_ptr: *mut u8, len: usize) {
    let v: &mut [String_21] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut String_21, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Clone, Ord, PartialOrd, PartialEq, Eq)]
struct String_22(String);

#[inline(never)]
fn instantiate_String_22(data_ptr: *mut u8, len: usize) {
    let v: &mut [String_22] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut String_22, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Clone, Ord, PartialOrd, PartialEq, Eq)]
struct String_23(String);

#[inline(never)]
fn instantiate_String_23(data_ptr: *mut u8, len: usize) {
    let v: &mut [String_23] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut String_23, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Clone, Ord, PartialOrd, PartialEq, Eq)]
struct String_24(String);

#[inline(never)]
fn instantiate_String_24(data_ptr: *mut u8, len: usize) {
    let v: &mut [String_24] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut String_24, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Clone, Ord, PartialOrd, PartialEq, Eq)]
struct String_25(String);

#[inline(never)]
fn instantiate_String_25(data_ptr: *mut u8, len: usize) {
    let v: &mut [String_25] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut String_25, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Clone, Ord, PartialOrd, PartialEq, Eq)]
struct String_26(String);

#[inline(never)]
fn instantiate_String_26(data_ptr: *mut u8, len: usize) {
    let v: &mut [String_26] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut String_26, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Clone, Ord, PartialOrd, PartialEq, Eq)]
struct String_27(String);

#[inline(never)]
fn instantiate_String_27(data_ptr: *mut u8, len: usize) {
    let v: &mut [String_27] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut String_27, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Clone, Ord, PartialOrd, PartialEq, Eq)]
struct String_28(String);

#[inline(never)]
fn instantiate_String_28(data_ptr: *mut u8, len: usize) {
    let v: &mut [String_28] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut String_28, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Clone, Ord, PartialOrd, PartialEq, Eq)]
struct String_29(String);

#[inline(never)]
fn instantiate_String_29(data_ptr: *mut u8, len: usize) {
    let v: &mut [String_29] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut String_29, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Clone, Ord, PartialOrd, PartialEq, Eq)]
struct String_30(String);

#[inline(never)]
fn instantiate_String_30(data_ptr: *mut u8, len: usize) {
    let v: &mut [String_30] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut String_30, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Clone, Ord, PartialOrd, PartialEq, Eq)]
struct String_31(String);

#[inline(never)]
fn instantiate_String_31(data_ptr: *mut u8, len: usize) {
    let v: &mut [String_31] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut String_31, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Clone, Ord, PartialOrd, PartialEq, Eq)]
struct String_32(String);

#[inline(never)]
fn instantiate_String_32(data_ptr: *mut u8, len: usize) {
    let v: &mut [String_32] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut String_32, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Clone, Ord, PartialOrd, PartialEq, Eq)]
struct String_33(String);

#[inline(never)]
fn instantiate_String_33(data_ptr: *mut u8, len: usize) {
    let v: &mut [String_33] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut String_33, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Clone, Ord, PartialOrd, PartialEq, Eq)]
struct String_34(String);

#[inline(never)]
fn instantiate_String_34(data_ptr: *mut u8, len: usize) {
    let v: &mut [String_34] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut String_34, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Clone, Ord, PartialOrd, PartialEq, Eq)]
struct String_35(String);

#[inline(never)]
fn instantiate_String_35(data_ptr: *mut u8, len: usize) {
    let v: &mut [String_35] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut String_35, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Clone, Ord, PartialOrd, PartialEq, Eq)]
struct String_36(String);

#[inline(never)]
fn instantiate_String_36(data_ptr: *mut u8, len: usize) {
    let v: &mut [String_36] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut String_36, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Clone, Ord, PartialOrd, PartialEq, Eq)]
struct String_37(String);

#[inline(never)]
fn instantiate_String_37(data_ptr: *mut u8, len: usize) {
    let v: &mut [String_37] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut String_37, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Clone, Ord, PartialOrd, PartialEq, Eq)]
struct String_38(String);

#[inline(never)]
fn instantiate_String_38(data_ptr: *mut u8, len: usize) {
    let v: &mut [String_38] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut String_38, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Clone, Ord, PartialOrd, PartialEq, Eq)]
struct String_39(String);

#[inline(never)]
fn instantiate_String_39(data_ptr: *mut u8, len: usize) {
    let v: &mut [String_39] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut String_39, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Clone, Ord, PartialOrd, PartialEq, Eq)]
struct String_40(String);

#[inline(never)]
fn instantiate_String_40(data_ptr: *mut u8, len: usize) {
    let v: &mut [String_40] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut String_40, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Clone, Ord, PartialOrd, PartialEq, Eq)]
struct String_41(String);

#[inline(never)]
fn instantiate_String_41(data_ptr: *mut u8, len: usize) {
    let v: &mut [String_41] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut String_41, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Clone, Ord, PartialOrd, PartialEq, Eq)]
struct String_42(String);

#[inline(never)]
fn instantiate_String_42(data_ptr: *mut u8, len: usize) {
    let v: &mut [String_42] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut String_42, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Clone, Ord, PartialOrd, PartialEq, Eq)]
struct String_43(String);

#[inline(never)]
fn instantiate_String_43(data_ptr: *mut u8, len: usize) {
    let v: &mut [String_43] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut String_43, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Clone, Ord, PartialOrd, PartialEq, Eq)]
struct String_44(String);

#[inline(never)]
fn instantiate_String_44(data_ptr: *mut u8, len: usize) {
    let v: &mut [String_44] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut String_44, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Clone, Ord, PartialOrd, PartialEq, Eq)]
struct String_45(String);

#[inline(never)]
fn instantiate_String_45(data_ptr: *mut u8, len: usize) {
    let v: &mut [String_45] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut String_45, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Clone, Ord, PartialOrd, PartialEq, Eq)]
struct String_46(String);

#[inline(never)]
fn instantiate_String_46(data_ptr: *mut u8, len: usize) {
    let v: &mut [String_46] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut String_46, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Clone, Ord, PartialOrd, PartialEq, Eq)]
struct String_47(String);

#[inline(never)]
fn instantiate_String_47(data_ptr: *mut u8, len: usize) {
    let v: &mut [String_47] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut String_47, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Clone, Ord, PartialOrd, PartialEq, Eq)]
struct String_48(String);

#[inline(never)]
fn instantiate_String_48(data_ptr: *mut u8, len: usize) {
    let v: &mut [String_48] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut String_48, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Clone, Ord, PartialOrd, PartialEq, Eq)]
struct String_49(String);

#[inline(never)]
fn instantiate_String_49(data_ptr: *mut u8, len: usize) {
    let v: &mut [String_49] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut String_49, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Clone, Ord, PartialOrd, PartialEq, Eq)]
struct String_50(String);

#[inline(never)]
fn instantiate_String_50(data_ptr: *mut u8, len: usize) {
    let v: &mut [String_50] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut String_50, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Clone, Ord, PartialOrd, PartialEq, Eq)]
struct String_51(String);

#[inline(never)]
fn instantiate_String_51(data_ptr: *mut u8, len: usize) {
    let v: &mut [String_51] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut String_51, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Clone, Ord, PartialOrd, PartialEq, Eq)]
struct String_52(String);

#[inline(never)]
fn instantiate_String_52(data_ptr: *mut u8, len: usize) {
    let v: &mut [String_52] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut String_52, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Clone, Ord, PartialOrd, PartialEq, Eq)]
struct String_53(String);

#[inline(never)]
fn instantiate_String_53(data_ptr: *mut u8, len: usize) {
    let v: &mut [String_53] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut String_53, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Clone, Ord, PartialOrd, PartialEq, Eq)]
struct String_54(String);

#[inline(never)]
fn instantiate_String_54(data_ptr: *mut u8, len: usize) {
    let v: &mut [String_54] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut String_54, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Clone, Ord, PartialOrd, PartialEq, Eq)]
struct String_55(String);

#[inline(never)]
fn instantiate_String_55(data_ptr: *mut u8, len: usize) {
    let v: &mut [String_55] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut String_55, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Clone, Ord, PartialOrd, PartialEq, Eq)]
struct String_56(String);

#[inline(never)]
fn instantiate_String_56(data_ptr: *mut u8, len: usize) {
    let v: &mut [String_56] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut String_56, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Clone, Ord, PartialOrd, PartialEq, Eq)]
struct String_57(String);

#[inline(never)]
fn instantiate_String_57(data_ptr: *mut u8, len: usize) {
    let v: &mut [String_57] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut String_57, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Clone, Ord, PartialOrd, PartialEq, Eq)]
struct String_58(String);

#[inline(never)]
fn instantiate_String_58(data_ptr: *mut u8, len: usize) {
    let v: &mut [String_58] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut String_58, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Clone, Ord, PartialOrd, PartialEq, Eq)]
struct String_59(String);

#[inline(never)]
fn instantiate_String_59(data_ptr: *mut u8, len: usize) {
    let v: &mut [String_59] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut String_59, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Clone, Ord, PartialOrd, PartialEq, Eq)]
struct String_60(String);

#[inline(never)]
fn instantiate_String_60(data_ptr: *mut u8, len: usize) {
    let v: &mut [String_60] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut String_60, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Clone, Ord, PartialOrd, PartialEq, Eq)]
struct String_61(String);

#[inline(never)]
fn instantiate_String_61(data_ptr: *mut u8, len: usize) {
    let v: &mut [String_61] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut String_61, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Clone, Ord, PartialOrd, PartialEq, Eq)]
struct String_62(String);

#[inline(never)]
fn instantiate_String_62(data_ptr: *mut u8, len: usize) {
    let v: &mut [String_62] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut String_62, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Clone, Ord, PartialOrd, PartialEq, Eq)]
struct String_63(String);

#[inline(never)]
fn instantiate_String_63(data_ptr: *mut u8, len: usize) {
    let v: &mut [String_63] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut String_63, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Clone, Ord, PartialOrd, PartialEq, Eq)]
struct String_64(String);

#[inline(never)]
fn instantiate_String_64(data_ptr: *mut u8, len: usize) {
    let v: &mut [String_64] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut String_64, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Clone, Ord, PartialOrd, PartialEq, Eq)]
struct String_65(String);

#[inline(never)]
fn instantiate_String_65(data_ptr: *mut u8, len: usize) {
    let v: &mut [String_65] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut String_65, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Clone, Ord, PartialOrd, PartialEq, Eq)]
struct String_66(String);

#[inline(never)]
fn instantiate_String_66(data_ptr: *mut u8, len: usize) {
    let v: &mut [String_66] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut String_66, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Clone, Ord, PartialOrd, PartialEq, Eq)]
struct String_67(String);

#[inline(never)]
fn instantiate_String_67(data_ptr: *mut u8, len: usize) {
    let v: &mut [String_67] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut String_67, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Clone, Ord, PartialOrd, PartialEq, Eq)]
struct String_68(String);

#[inline(never)]
fn instantiate_String_68(data_ptr: *mut u8, len: usize) {
    let v: &mut [String_68] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut String_68, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Clone, Ord, PartialOrd, PartialEq, Eq)]
struct String_69(String);

#[inline(never)]
fn instantiate_String_69(data_ptr: *mut u8, len: usize) {
    let v: &mut [String_69] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut String_69, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Clone, Ord, PartialOrd, PartialEq, Eq)]
struct String_70(String);

#[inline(never)]
fn instantiate_String_70(data_ptr: *mut u8, len: usize) {
    let v: &mut [String_70] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut String_70, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Clone, Ord, PartialOrd, PartialEq, Eq)]
struct String_71(String);

#[inline(never)]
fn instantiate_String_71(data_ptr: *mut u8, len: usize) {
    let v: &mut [String_71] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut String_71, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Clone, Ord, PartialOrd, PartialEq, Eq)]
struct String_72(String);

#[inline(never)]
fn instantiate_String_72(data_ptr: *mut u8, len: usize) {
    let v: &mut [String_72] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut String_72, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Clone, Ord, PartialOrd, PartialEq, Eq)]
struct String_73(String);

#[inline(never)]
fn instantiate_String_73(data_ptr: *mut u8, len: usize) {
    let v: &mut [String_73] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut String_73, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Clone, Ord, PartialOrd, PartialEq, Eq)]
struct String_74(String);

#[inline(never)]
fn instantiate_String_74(data_ptr: *mut u8, len: usize) {
    let v: &mut [String_74] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut String_74, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Clone, Ord, PartialOrd, PartialEq, Eq)]
struct String_75(String);

#[inline(never)]
fn instantiate_String_75(data_ptr: *mut u8, len: usize) {
    let v: &mut [String_75] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut String_75, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Clone, Ord, PartialOrd, PartialEq, Eq)]
struct String_76(String);

#[inline(never)]
fn instantiate_String_76(data_ptr: *mut u8, len: usize) {
    let v: &mut [String_76] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut String_76, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Clone, Ord, PartialOrd, PartialEq, Eq)]
struct String_77(String);

#[inline(never)]
fn instantiate_String_77(data_ptr: *mut u8, len: usize) {
    let v: &mut [String_77] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut String_77, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Clone, Ord, PartialOrd, PartialEq, Eq)]
struct String_78(String);

#[inline(never)]
fn instantiate_String_78(data_ptr: *mut u8, len: usize) {
    let v: &mut [String_78] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut String_78, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Clone, Ord, PartialOrd, PartialEq, Eq)]
struct String_79(String);

#[inline(never)]
fn instantiate_String_79(data_ptr: *mut u8, len: usize) {
    let v: &mut [String_79] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut String_79, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Clone, Ord, PartialOrd, PartialEq, Eq)]
struct String_80(String);

#[inline(never)]
fn instantiate_String_80(data_ptr: *mut u8, len: usize) {
    let v: &mut [String_80] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut String_80, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Clone, Ord, PartialOrd, PartialEq, Eq)]
struct String_81(String);

#[inline(never)]
fn instantiate_String_81(data_ptr: *mut u8, len: usize) {
    let v: &mut [String_81] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut String_81, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Clone, Ord, PartialOrd, PartialEq, Eq)]
struct String_82(String);

#[inline(never)]
fn instantiate_String_82(data_ptr: *mut u8, len: usize) {
    let v: &mut [String_82] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut String_82, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Clone, Ord, PartialOrd, PartialEq, Eq)]
struct String_83(String);

#[inline(never)]
fn instantiate_String_83(data_ptr: *mut u8, len: usize) {
    let v: &mut [String_83] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut String_83, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Clone, Ord, PartialOrd, PartialEq, Eq)]
struct String_84(String);

#[inline(never)]
fn instantiate_String_84(data_ptr: *mut u8, len: usize) {
    let v: &mut [String_84] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut String_84, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Clone, Ord, PartialOrd, PartialEq, Eq)]
struct String_85(String);

#[inline(never)]
fn instantiate_String_85(data_ptr: *mut u8, len: usize) {
    let v: &mut [String_85] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut String_85, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Clone, Ord, PartialOrd, PartialEq, Eq)]
struct String_86(String);

#[inline(never)]
fn instantiate_String_86(data_ptr: *mut u8, len: usize) {
    let v: &mut [String_86] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut String_86, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Clone, Ord, PartialOrd, PartialEq, Eq)]
struct String_87(String);

#[inline(never)]
fn instantiate_String_87(data_ptr: *mut u8, len: usize) {
    let v: &mut [String_87] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut String_87, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Clone, Ord, PartialOrd, PartialEq, Eq)]
struct String_88(String);

#[inline(never)]
fn instantiate_String_88(data_ptr: *mut u8, len: usize) {
    let v: &mut [String_88] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut String_88, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Clone, Ord, PartialOrd, PartialEq, Eq)]
struct String_89(String);

#[inline(never)]
fn instantiate_String_89(data_ptr: *mut u8, len: usize) {
    let v: &mut [String_89] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut String_89, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Clone, Ord, PartialOrd, PartialEq, Eq)]
struct String_90(String);

#[inline(never)]
fn instantiate_String_90(data_ptr: *mut u8, len: usize) {
    let v: &mut [String_90] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut String_90, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Clone, Ord, PartialOrd, PartialEq, Eq)]
struct String_91(String);

#[inline(never)]
fn instantiate_String_91(data_ptr: *mut u8, len: usize) {
    let v: &mut [String_91] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut String_91, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Clone, Ord, PartialOrd, PartialEq, Eq)]
struct String_92(String);

#[inline(never)]
fn instantiate_String_92(data_ptr: *mut u8, len: usize) {
    let v: &mut [String_92] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut String_92, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Clone, Ord, PartialOrd, PartialEq, Eq)]
struct String_93(String);

#[inline(never)]
fn instantiate_String_93(data_ptr: *mut u8, len: usize) {
    let v: &mut [String_93] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut String_93, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Clone, Ord, PartialOrd, PartialEq, Eq)]
struct String_94(String);

#[inline(never)]
fn instantiate_String_94(data_ptr: *mut u8, len: usize) {
    let v: &mut [String_94] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut String_94, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Clone, Ord, PartialOrd, PartialEq, Eq)]
struct String_95(String);

#[inline(never)]
fn instantiate_String_95(data_ptr: *mut u8, len: usize) {
    let v: &mut [String_95] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut String_95, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Clone, Ord, PartialOrd, PartialEq, Eq)]
struct String_96(String);

#[inline(never)]
fn instantiate_String_96(data_ptr: *mut u8, len: usize) {
    let v: &mut [String_96] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut String_96, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Clone, Ord, PartialOrd, PartialEq, Eq)]
struct String_97(String);

#[inline(never)]
fn instantiate_String_97(data_ptr: *mut u8, len: usize) {
    let v: &mut [String_97] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut String_97, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Clone, Ord, PartialOrd, PartialEq, Eq)]
struct String_98(String);

#[inline(never)]
fn instantiate_String_98(data_ptr: *mut u8, len: usize) {
    let v: &mut [String_98] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut String_98, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Clone, Ord, PartialOrd, PartialEq, Eq)]
struct String_99(String);

#[inline(never)]
fn instantiate_String_99(data_ptr: *mut u8, len: usize) {
    let v: &mut [String_99] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut String_99, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Clone, Ord, PartialOrd, PartialEq, Eq)]
struct String_100(String);

#[inline(never)]
fn instantiate_String_100(data_ptr: *mut u8, len: usize) {
    let v: &mut [String_100] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut String_100, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Clone, Ord, PartialOrd, PartialEq, Eq)]
struct String_101(String);

#[inline(never)]
fn instantiate_String_101(data_ptr: *mut u8, len: usize) {
    let v: &mut [String_101] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut String_101, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Clone, Ord, PartialOrd, PartialEq, Eq)]
struct String_102(String);

#[inline(never)]
fn instantiate_String_102(data_ptr: *mut u8, len: usize) {
    let v: &mut [String_102] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut String_102, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Clone, Ord, PartialOrd, PartialEq, Eq)]
struct String_103(String);

#[inline(never)]
fn instantiate_String_103(data_ptr: *mut u8, len: usize) {
    let v: &mut [String_103] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut String_103, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Clone, Ord, PartialOrd, PartialEq, Eq)]
struct String_104(String);

#[inline(never)]
fn instantiate_String_104(data_ptr: *mut u8, len: usize) {
    let v: &mut [String_104] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut String_104, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Clone, Ord, PartialOrd, PartialEq, Eq)]
struct String_105(String);

#[inline(never)]
fn instantiate_String_105(data_ptr: *mut u8, len: usize) {
    let v: &mut [String_105] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut String_105, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Clone, Ord, PartialOrd, PartialEq, Eq)]
struct String_106(String);

#[inline(never)]
fn instantiate_String_106(data_ptr: *mut u8, len: usize) {
    let v: &mut [String_106] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut String_106, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Clone, Ord, PartialOrd, PartialEq, Eq)]
struct String_107(String);

#[inline(never)]
fn instantiate_String_107(data_ptr: *mut u8, len: usize) {
    let v: &mut [String_107] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut String_107, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Clone, Ord, PartialOrd, PartialEq, Eq)]
struct String_108(String);

#[inline(never)]
fn instantiate_String_108(data_ptr: *mut u8, len: usize) {
    let v: &mut [String_108] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut String_108, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Clone, Ord, PartialOrd, PartialEq, Eq)]
struct String_109(String);

#[inline(never)]
fn instantiate_String_109(data_ptr: *mut u8, len: usize) {
    let v: &mut [String_109] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut String_109, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Clone, Ord, PartialOrd, PartialEq, Eq)]
struct String_110(String);

#[inline(never)]
fn instantiate_String_110(data_ptr: *mut u8, len: usize) {
    let v: &mut [String_110] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut String_110, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Clone, Ord, PartialOrd, PartialEq, Eq)]
struct String_111(String);

#[inline(never)]
fn instantiate_String_111(data_ptr: *mut u8, len: usize) {
    let v: &mut [String_111] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut String_111, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Clone, Ord, PartialOrd, PartialEq, Eq)]
struct String_112(String);

#[inline(never)]
fn instantiate_String_112(data_ptr: *mut u8, len: usize) {
    let v: &mut [String_112] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut String_112, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Clone, Ord, PartialOrd, PartialEq, Eq)]
struct String_113(String);

#[inline(never)]
fn instantiate_String_113(data_ptr: *mut u8, len: usize) {
    let v: &mut [String_113] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut String_113, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Clone, Ord, PartialOrd, PartialEq, Eq)]
struct String_114(String);

#[inline(never)]
fn instantiate_String_114(data_ptr: *mut u8, len: usize) {
    let v: &mut [String_114] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut String_114, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Clone, Ord, PartialOrd, PartialEq, Eq)]
struct Cell_0(std::cell::Cell<u64>);

#[inline(never)]
fn instantiate_Cell_0(data_ptr: *mut u8, len: usize) {
    let v: &mut [Cell_0] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut Cell_0, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Clone, Ord, PartialOrd, PartialEq, Eq)]
struct Cell_1(std::cell::Cell<u64>);

#[inline(never)]
fn instantiate_Cell_1(data_ptr: *mut u8, len: usize) {
    let v: &mut [Cell_1] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut Cell_1, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Clone, Ord, PartialOrd, PartialEq, Eq)]
struct Cell_2(std::cell::Cell<u64>);

#[inline(never)]
fn instantiate_Cell_2(data_ptr: *mut u8, len: usize) {
    let v: &mut [Cell_2] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut Cell_2, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Clone, Ord, PartialOrd, PartialEq, Eq)]
struct Cell_3(std::cell::Cell<u64>);

#[inline(never)]
fn instantiate_Cell_3(data_ptr: *mut u8, len: usize) {
    let v: &mut [Cell_3] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut Cell_3, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Clone, Ord, PartialOrd, PartialEq, Eq)]
struct Cell_4(std::cell::Cell<u64>);

#[inline(never)]
fn instantiate_Cell_4(data_ptr: *mut u8, len: usize) {
    let v: &mut [Cell_4] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut Cell_4, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Clone, Ord, PartialOrd, PartialEq, Eq)]
struct Cell_5(std::cell::Cell<u64>);

#[inline(never)]
fn instantiate_Cell_5(data_ptr: *mut u8, len: usize) {
    let v: &mut [Cell_5] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut Cell_5, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Clone, Ord, PartialOrd, PartialEq, Eq)]
struct Cell_6(std::cell::Cell<u64>);

#[inline(never)]
fn instantiate_Cell_6(data_ptr: *mut u8, len: usize) {
    let v: &mut [Cell_6] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut Cell_6, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Clone, Ord, PartialOrd, PartialEq, Eq)]
struct Cell_7(std::cell::Cell<u64>);

#[inline(never)]
fn instantiate_Cell_7(data_ptr: *mut u8, len: usize) {
    let v: &mut [Cell_7] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut Cell_7, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Clone, Ord, PartialOrd, PartialEq, Eq)]
struct Cell_8(std::cell::Cell<u64>);

#[inline(never)]
fn instantiate_Cell_8(data_ptr: *mut u8, len: usize) {
    let v: &mut [Cell_8] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut Cell_8, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Clone, Ord, PartialOrd, PartialEq, Eq)]
struct Cell_9(std::cell::Cell<u64>);

#[inline(never)]
fn instantiate_Cell_9(data_ptr: *mut u8, len: usize) {
    let v: &mut [Cell_9] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut Cell_9, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Clone, Ord, PartialOrd, PartialEq, Eq)]
struct Cell_10(std::cell::Cell<u64>);

#[inline(never)]
fn instantiate_Cell_10(data_ptr: *mut u8, len: usize) {
    let v: &mut [Cell_10] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut Cell_10, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Clone, Ord, PartialOrd, PartialEq, Eq)]
struct Cell_11(std::cell::Cell<u64>);

#[inline(never)]
fn instantiate_Cell_11(data_ptr: *mut u8, len: usize) {
    let v: &mut [Cell_11] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut Cell_11, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

#[derive(Clone, Ord, PartialOrd, PartialEq, Eq)]
struct Cell_12(std::cell::Cell<u64>);

#[inline(never)]
fn instantiate_Cell_12(data_ptr: *mut u8, len: usize) {
    let v: &mut [Cell_12] =
        unsafe { &mut *std::ptr::slice_from_raw_parts_mut(data_ptr as *mut Cell_12, len) };

    instantiate_test_sort(v);

    if v[0] > v[v.len() - 1] {
        panic!(); // side-effect
    }
}

fn instantiate_all(data_ptr: *mut u8, len: usize) {
    instantiate_U64_0(data_ptr, len);
    instantiate_U64_1(data_ptr, len);
    instantiate_U64_2(data_ptr, len);
    instantiate_U64_3(data_ptr, len);
    instantiate_U64_4(data_ptr, len);
    instantiate_U64_5(data_ptr, len);
    instantiate_U64_6(data_ptr, len);
    instantiate_U64_7(data_ptr, len);
    instantiate_U64_8(data_ptr, len);
    instantiate_U64_9(data_ptr, len);
    instantiate_U64_10(data_ptr, len);
    instantiate_U64_11(data_ptr, len);
    instantiate_U64_12(data_ptr, len);
    instantiate_U64_13(data_ptr, len);
    instantiate_U64_14(data_ptr, len);
    instantiate_U64_15(data_ptr, len);
    instantiate_U64_16(data_ptr, len);
    instantiate_U64_17(data_ptr, len);
    instantiate_U64_18(data_ptr, len);
    instantiate_U64_19(data_ptr, len);
    instantiate_U64_20(data_ptr, len);
    instantiate_U64_21(data_ptr, len);
    instantiate_U64_22(data_ptr, len);
    instantiate_U64_23(data_ptr, len);
    instantiate_U64_24(data_ptr, len);
    instantiate_U64_25(data_ptr, len);
    instantiate_U64_26(data_ptr, len);
    instantiate_U64_27(data_ptr, len);
    instantiate_U64_28(data_ptr, len);
    instantiate_U64_29(data_ptr, len);
    instantiate_U64_30(data_ptr, len);
    instantiate_U64_31(data_ptr, len);
    instantiate_U64_32(data_ptr, len);
    instantiate_U64_33(data_ptr, len);
    instantiate_U64_34(data_ptr, len);
    instantiate_U64_35(data_ptr, len);
    instantiate_U64_36(data_ptr, len);
    instantiate_U64_37(data_ptr, len);
    instantiate_U64_38(data_ptr, len);
    instantiate_U64_39(data_ptr, len);
    instantiate_U64_40(data_ptr, len);
    instantiate_U64_41(data_ptr, len);
    instantiate_U64_42(data_ptr, len);
    instantiate_U64_43(data_ptr, len);
    instantiate_U64_44(data_ptr, len);
    instantiate_U64_45(data_ptr, len);
    instantiate_U64_46(data_ptr, len);
    instantiate_U64_47(data_ptr, len);
    instantiate_U64_48(data_ptr, len);
    instantiate_U64_49(data_ptr, len);
    instantiate_U64_50(data_ptr, len);
    instantiate_U64_51(data_ptr, len);
    instantiate_U64_52(data_ptr, len);
    instantiate_U64_53(data_ptr, len);
    instantiate_U64_54(data_ptr, len);
    instantiate_U64_55(data_ptr, len);
    instantiate_U64_56(data_ptr, len);
    instantiate_U64_57(data_ptr, len);
    instantiate_U64_58(data_ptr, len);
    instantiate_U64_59(data_ptr, len);
    instantiate_U64_60(data_ptr, len);
    instantiate_U64_61(data_ptr, len);
    instantiate_U64_62(data_ptr, len);
    instantiate_U64_63(data_ptr, len);
    instantiate_U64_64(data_ptr, len);
    instantiate_U64_65(data_ptr, len);
    instantiate_U64_66(data_ptr, len);
    instantiate_U64_67(data_ptr, len);
    instantiate_U64_68(data_ptr, len);
    instantiate_U64_69(data_ptr, len);
    instantiate_U64_70(data_ptr, len);
    instantiate_U64_71(data_ptr, len);
    instantiate_U64_72(data_ptr, len);
    instantiate_U64_73(data_ptr, len);
    instantiate_U64_74(data_ptr, len);
    instantiate_U64_75(data_ptr, len);
    instantiate_U64_76(data_ptr, len);
    instantiate_U64_77(data_ptr, len);
    instantiate_U64_78(data_ptr, len);
    instantiate_U64_79(data_ptr, len);
    instantiate_U64_80(data_ptr, len);
    instantiate_U64_81(data_ptr, len);
    instantiate_U64_82(data_ptr, len);
    instantiate_U64_83(data_ptr, len);
    instantiate_U64_84(data_ptr, len);
    instantiate_U64_85(data_ptr, len);
    instantiate_U64_86(data_ptr, len);
    instantiate_U64_87(data_ptr, len);
    instantiate_U64_88(data_ptr, len);
    instantiate_U64_89(data_ptr, len);
    instantiate_U64_90(data_ptr, len);
    instantiate_U64_91(data_ptr, len);
    instantiate_U64_92(data_ptr, len);
    instantiate_U64_93(data_ptr, len);
    instantiate_U64_94(data_ptr, len);
    instantiate_U64_95(data_ptr, len);
    instantiate_U64_96(data_ptr, len);
    instantiate_U64_97(data_ptr, len);
    instantiate_U64_98(data_ptr, len);
    instantiate_U64_99(data_ptr, len);
    instantiate_U64_100(data_ptr, len);
    instantiate_U64_101(data_ptr, len);
    instantiate_U64_102(data_ptr, len);
    instantiate_U64_103(data_ptr, len);
    instantiate_U64_104(data_ptr, len);
    instantiate_U64_105(data_ptr, len);
    instantiate_U64_106(data_ptr, len);
    instantiate_U64_107(data_ptr, len);
    instantiate_U64_108(data_ptr, len);
    instantiate_U64_109(data_ptr, len);
    instantiate_U64_110(data_ptr, len);
    instantiate_U64_111(data_ptr, len);
    instantiate_U64_112(data_ptr, len);
    instantiate_U64_113(data_ptr, len);
    instantiate_U64_114(data_ptr, len);
    instantiate_U64_115(data_ptr, len);
    instantiate_U64_116(data_ptr, len);
    instantiate_U64_117(data_ptr, len);
    instantiate_U64_118(data_ptr, len);
    instantiate_U64_119(data_ptr, len);
    instantiate_U64_120(data_ptr, len);
    instantiate_U64_121(data_ptr, len);
    instantiate_U64_122(data_ptr, len);
    instantiate_U64_123(data_ptr, len);
    instantiate_U64_124(data_ptr, len);
    instantiate_U64_125(data_ptr, len);
    instantiate_U64_126(data_ptr, len);
    instantiate_U64_127(data_ptr, len);
    instantiate_String_0(data_ptr, len);
    instantiate_String_1(data_ptr, len);
    instantiate_String_2(data_ptr, len);
    instantiate_String_3(data_ptr, len);
    instantiate_String_4(data_ptr, len);
    instantiate_String_5(data_ptr, len);
    instantiate_String_6(data_ptr, len);
    instantiate_String_7(data_ptr, len);
    instantiate_String_8(data_ptr, len);
    instantiate_String_9(data_ptr, len);
    instantiate_String_10(data_ptr, len);
    instantiate_String_11(data_ptr, len);
    instantiate_String_12(data_ptr, len);
    instantiate_String_13(data_ptr, len);
    instantiate_String_14(data_ptr, len);
    instantiate_String_15(data_ptr, len);
    instantiate_String_16(data_ptr, len);
    instantiate_String_17(data_ptr, len);
    instantiate_String_18(data_ptr, len);
    instantiate_String_19(data_ptr, len);
    instantiate_String_20(data_ptr, len);
    instantiate_String_21(data_ptr, len);
    instantiate_String_22(data_ptr, len);
    instantiate_String_23(data_ptr, len);
    instantiate_String_24(data_ptr, len);
    instantiate_String_25(data_ptr, len);
    instantiate_String_26(data_ptr, len);
    instantiate_String_27(data_ptr, len);
    instantiate_String_28(data_ptr, len);
    instantiate_String_29(data_ptr, len);
    instantiate_String_30(data_ptr, len);
    instantiate_String_31(data_ptr, len);
    instantiate_String_32(data_ptr, len);
    instantiate_String_33(data_ptr, len);
    instantiate_String_34(data_ptr, len);
    instantiate_String_35(data_ptr, len);
    instantiate_String_36(data_ptr, len);
    instantiate_String_37(data_ptr, len);
    instantiate_String_38(data_ptr, len);
    instantiate_String_39(data_ptr, len);
    instantiate_String_40(data_ptr, len);
    instantiate_String_41(data_ptr, len);
    instantiate_String_42(data_ptr, len);
    instantiate_String_43(data_ptr, len);
    instantiate_String_44(data_ptr, len);
    instantiate_String_45(data_ptr, len);
    instantiate_String_46(data_ptr, len);
    instantiate_String_47(data_ptr, len);
    instantiate_String_48(data_ptr, len);
    instantiate_String_49(data_ptr, len);
    instantiate_String_50(data_ptr, len);
    instantiate_String_51(data_ptr, len);
    instantiate_String_52(data_ptr, len);
    instantiate_String_53(data_ptr, len);
    instantiate_String_54(data_ptr, len);
    instantiate_String_55(data_ptr, len);
    instantiate_String_56(data_ptr, len);
    instantiate_String_57(data_ptr, len);
    instantiate_String_58(data_ptr, len);
    instantiate_String_59(data_ptr, len);
    instantiate_String_60(data_ptr, len);
    instantiate_String_61(data_ptr, len);
    instantiate_String_62(data_ptr, len);
    instantiate_String_63(data_ptr, len);
    instantiate_String_64(data_ptr, len);
    instantiate_String_65(data_ptr, len);
    instantiate_String_66(data_ptr, len);
    instantiate_String_67(data_ptr, len);
    instantiate_String_68(data_ptr, len);
    instantiate_String_69(data_ptr, len);
    instantiate_String_70(data_ptr, len);
    instantiate_String_71(data_ptr, len);
    instantiate_String_72(data_ptr, len);
    instantiate_String_73(data_ptr, len);
    instantiate_String_74(data_ptr, len);
    instantiate_String_75(data_ptr, len);
    instantiate_String_76(data_ptr, len);
    instantiate_String_77(data_ptr, len);
    instantiate_String_78(data_ptr, len);
    instantiate_String_79(data_ptr, len);
    instantiate_String_80(data_ptr, len);
    instantiate_String_81(data_ptr, len);
    instantiate_String_82(data_ptr, len);
    instantiate_String_83(data_ptr, len);
    instantiate_String_84(data_ptr, len);
    instantiate_String_85(data_ptr, len);
    instantiate_String_86(data_ptr, len);
    instantiate_String_87(data_ptr, len);
    instantiate_String_88(data_ptr, len);
    instantiate_String_89(data_ptr, len);
    instantiate_String_90(data_ptr, len);
    instantiate_String_91(data_ptr, len);
    instantiate_String_92(data_ptr, len);
    instantiate_String_93(data_ptr, len);
    instantiate_String_94(data_ptr, len);
    instantiate_String_95(data_ptr, len);
    instantiate_String_96(data_ptr, len);
    instantiate_String_97(data_ptr, len);
    instantiate_String_98(data_ptr, len);
    instantiate_String_99(data_ptr, len);
    instantiate_String_100(data_ptr, len);
    instantiate_String_101(data_ptr, len);
    instantiate_String_102(data_ptr, len);
    instantiate_String_103(data_ptr, len);
    instantiate_String_104(data_ptr, len);
    instantiate_String_105(data_ptr, len);
    instantiate_String_106(data_ptr, len);
    instantiate_String_107(data_ptr, len);
    instantiate_String_108(data_ptr, len);
    instantiate_String_109(data_ptr, len);
    instantiate_String_110(data_ptr, len);
    instantiate_String_111(data_ptr, len);
    instantiate_String_112(data_ptr, len);
    instantiate_String_113(data_ptr, len);
    instantiate_String_114(data_ptr, len);
    instantiate_Cell_0(data_ptr, len);
    instantiate_Cell_1(data_ptr, len);
    instantiate_Cell_2(data_ptr, len);
    instantiate_Cell_3(data_ptr, len);
    instantiate_Cell_4(data_ptr, len);
    instantiate_Cell_5(data_ptr, len);
    instantiate_Cell_6(data_ptr, len);
    instantiate_Cell_7(data_ptr, len);
    instantiate_Cell_8(data_ptr, len);
    instantiate_Cell_9(data_ptr, len);
    instantiate_Cell_10(data_ptr, len);
    instantiate_Cell_11(data_ptr, len);
    instantiate_Cell_12(data_ptr, len);
}

fn main() {
    // This is only meant to test compile impact, never run this.

    // source of compiler unpredictable values.
    let data_ptr: *mut u8 = std::hint::black_box(std::ptr::null_mut());
    let len: usize = std::env::args().len();

    instantiate_all(data_ptr, len);
}
