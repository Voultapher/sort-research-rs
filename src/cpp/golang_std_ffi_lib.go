package main

/*
#include <stdint.h>

typedef int64_t (*i32_by_cmp_fn_ptr_t) (int32_t, int32_t);
int64_t i32_by_bridge(i32_by_cmp_fn_ptr_t fn_ptr, int32_t a, int32_t b);

typedef int64_t (*u64_by_cmp_fn_ptr_t) (uint64_t, uint64_t);
int64_t u64_by_bridge(u64_by_cmp_fn_ptr_t fn_ptr, uint64_t a, uint64_t b);
*/
import "C"

import (
	"cmp"
	"slices"
)

const PANIC_MAGIC_NUMBER = 777;

//export StableSortI32
func StableSortI32(v []int32) {
	slices.SortStableFunc(v, func(a, b int32) int {
		return cmp.Compare(a, b)
	})
}

//export StableSortI32By
func StableSortI32By(v []int32, cmp C.i32_by_cmp_fn_ptr_t) bool {
	var did_panic = false

	func() {
		defer func() {
			if r := recover(); r != nil {
				did_panic = true
			}
		}()

		slices.SortStableFunc(v, func(a, b int32) int {
			var cmp_result = int(C.i32_by_bridge(cmp, C.int(a), C.int(b)));
			if cmp_result == PANIC_MAGIC_NUMBER {
				panic("");
			}

			return cmp_result
		})
	}()

	return did_panic
}

//export StableSortU64
func StableSortU64(v []uint64) {
	slices.SortStableFunc(v, func(a, b uint64) int {
		return cmp.Compare(a, b)
	})
}

//export StableSortU64By
func StableSortU64By(v []uint64, cmp C.u64_by_cmp_fn_ptr_t) bool {
	var did_panic = false

	func() {
		defer func() {
			if r := recover(); r != nil {
				did_panic = true
			}
		}()

		slices.SortStableFunc(v, func(a, b uint64) int {
			var cmp_result = int(C.u64_by_bridge(cmp, C.ulong(a), C.ulong(b)));
			if cmp_result == PANIC_MAGIC_NUMBER {
				panic("");
			}

			return cmp_result
		})
	}()

	return did_panic
}

//export UnstableSortI32
func UnstableSortI32(v []int32) {
	slices.Sort(v)
}

//export UnstableSortI32By
func UnstableSortI32By(v []int32, cmp C.i32_by_cmp_fn_ptr_t) bool {
	var did_panic = false

	func() {
		defer func() {
			if r := recover(); r != nil {
				did_panic = true
			}
		}()

		slices.SortFunc(v, func(a, b int32) int {
			var cmp_result = int(C.i32_by_bridge(cmp, C.int(a), C.int(b)));
			if cmp_result == PANIC_MAGIC_NUMBER {
				panic("");
			}

			return cmp_result
		})
	}()

	return did_panic
}

//export UnstableSortU64
func UnstableSortU64(v []uint64) {
	slices.Sort(v)
}

//export UnstableSortU64By
func UnstableSortU64By(v []uint64, cmp C.u64_by_cmp_fn_ptr_t) bool {
	var did_panic = false

	func() {
		defer func() {
			if r := recover(); r != nil {
				did_panic = true
			}
		}()

		slices.SortFunc(v, func(a, b uint64) int {
			var cmp_result = int(C.u64_by_bridge(cmp, C.ulong(a), C.ulong(b)));
			if cmp_result == PANIC_MAGIC_NUMBER {
				panic("");
			}

			return cmp_result
		})
	}()

	return did_panic
}



func main() {}
