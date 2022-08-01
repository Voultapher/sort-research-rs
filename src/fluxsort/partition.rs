use std::mem;
use std::ptr;

use crate::fluxsort::median;
use crate::fluxsort::FLUX_OUT;

pub unsafe fn flux_reverse_partition<T, F>(
    arr_ptr: *mut T,
    swap_ptr: *mut T,
    x_ptr: *mut T,
    mut pivot_ptr: *mut T,
    len: usize,
    is_less: &mut F,
) where
    F: FnMut(&T, &T) -> bool,
{
    //     void FUNC(flux_reverse_partition)(VAR *array, VAR *swap, VAR *ptx, VAR *piv, size_t nmemb, CMPFUNC *cmp)
    // {
    // 	size_t a_size, s_size;

    // 	{
    // 		size_t cnt, val, m;
    // 		VAR *pts = swap;

    // 		for (m = 0, cnt = nmemb / 8 ; cnt ; cnt--)
    // 		{
    // 			val = cmp(piv, ptx) > 0; pts[-m] = array[m] = *ptx++; m += val; pts++;
    // 			val = cmp(piv, ptx) > 0; pts[-m] = array[m] = *ptx++; m += val; pts++;
    // 			val = cmp(piv, ptx) > 0; pts[-m] = array[m] = *ptx++; m += val; pts++;
    // 			val = cmp(piv, ptx) > 0; pts[-m] = array[m] = *ptx++; m += val; pts++;
    // 			val = cmp(piv, ptx) > 0; pts[-m] = array[m] = *ptx++; m += val; pts++;
    // 			val = cmp(piv, ptx) > 0; pts[-m] = array[m] = *ptx++; m += val; pts++;
    // 			val = cmp(piv, ptx) > 0; pts[-m] = array[m] = *ptx++; m += val; pts++;
    // 			val = cmp(piv, ptx) > 0; pts[-m] = array[m] = *ptx++; m += val; pts++;
    // 		}

    // 		for (cnt = nmemb % 8 ; cnt ; cnt--)
    // 		{
    // 			val = cmp(piv, ptx) > 0; pts[-m] = array[m] = *ptx++; m += val; pts++;
    // 		}
    // 		a_size = m;
    // 		s_size = nmemb - a_size;
    // 	}
    // 	memcpy(array + a_size, swap, s_size * sizeof(VAR));

    // 	if (s_size <= a_size / 16 || a_size <= FLUX_OUT)
    // 	{
    // 		return FUNC(quadsort_swap)(array, swap, a_size, a_size, cmp);
    // 	}
    // 	FUNC(flux_partition)(array, swap, array, piv, a_size, cmp);
    // }

    todo!();
}

pub unsafe fn flux_default_partition<T, F>(
    arr_ptr: *mut T,
    swap_ptr: *mut T,
    x_ptr: *mut T,
    mut pivot_ptr: *mut T,
    len: usize,
    is_less: &mut F,
) -> usize
where
    F: FnMut(&T, &T) -> bool,
{
    todo!();

    // size_t FUNC(flux_default_partition)(VAR *array, VAR *swap, VAR *ptx, VAR *piv, size_t nmemb, CMPFUNC *cmp)
    // {
    // 	size_t cnt, val, m = 0;

    // 	for (cnt = nmemb / 8 ; cnt ; cnt--)
    // 	{
    // 		val = cmp(ptx, piv) <= 0; swap[-m] = array[m] = *ptx++; m += val; swap++;
    // 		val = cmp(ptx, piv) <= 0; swap[-m] = array[m] = *ptx++; m += val; swap++;
    // 		val = cmp(ptx, piv) <= 0; swap[-m] = array[m] = *ptx++; m += val; swap++;
    // 		val = cmp(ptx, piv) <= 0; swap[-m] = array[m] = *ptx++; m += val; swap++;
    // 		val = cmp(ptx, piv) <= 0; swap[-m] = array[m] = *ptx++; m += val; swap++;
    // 		val = cmp(ptx, piv) <= 0; swap[-m] = array[m] = *ptx++; m += val; swap++;
    // 		val = cmp(ptx, piv) <= 0; swap[-m] = array[m] = *ptx++; m += val; swap++;
    // 		val = cmp(ptx, piv) <= 0; swap[-m] = array[m] = *ptx++; m += val; swap++;
    // 	}

    // 	for (cnt = nmemb % 8 ; cnt ; cnt--)
    // 	{
    // 		val = cmp(ptx, piv) <= 0; swap[-m] = array[m] = *ptx++; m += val; swap++;
    // 	}
    // 	return m;
    // }
}

#[inline]
pub unsafe fn flux_partition<T, F>(
    arr_ptr: *mut T,
    swap_ptr: *mut T,
    mut x_ptr: *mut T,
    mut pivot_ptr: *mut T,
    mut len: usize,
    is_less: &mut F,
) where
    F: FnMut(&T, &T) -> bool,
{
    // SAFETY: swap must be TODO
    debug_assert!(len > 16);

    let mut a_size = 0usize;
    let mut s_size = 0usize;

    loop {
        pivot_ptr = pivot_ptr.offset(-1);

        if len <= 2048 {
            // FIXME panic safety and holes.
            ptr::copy(
                median::median_of_nine(x_ptr, len, is_less).as_ptr(),
                pivot_ptr,
                1,
            );
        } else if len <= 65536 {
            ptr::copy(
                median::median_of_twentyfive(x_ptr, len, is_less).as_ptr(),
                pivot_ptr,
                1,
            );
        } else {
            ptr::copy(
                median::median_of_sqrt(arr_ptr, swap_ptr, x_ptr, len, is_less).as_ptr(),
                pivot_ptr,
                1,
            );
        }

        // cmp(piv + 1, piv) <= 0)
        // piv <  piv1 -> false
        // piv == piv1 -> true
        // piv >  piv1 -> true

        if a_size == 0 && !is_less(&*pivot_ptr, &*pivot_ptr.add(1)) {
            flux_reverse_partition(arr_ptr, swap_ptr, arr_ptr, pivot_ptr, len, is_less);
            return;
        }

        // 		a_size = FUNC(flux_default_partition)(array, swap, ptx, piv, nmemb, cmp);
        a_size = flux_default_partition(arr_ptr, swap_ptr, x_ptr, pivot_ptr, len, is_less);
        s_size = len - a_size;

        if a_size <= (s_size / 16) || s_size <= FLUX_OUT {
            if s_size == 0 {
                flux_reverse_partition(arr_ptr, swap_ptr, arr_ptr, pivot_ptr, a_size, is_less);
                return;
            }
            // 			memcpy(array + a_size, swap, s_size * sizeof(VAR));
            // 			FUNC(quadsort_swap)(array + a_size, swap, s_size, s_size, cmp);
        } else {
            flux_partition(
                arr_ptr.add(a_size),
                swap_ptr,
                swap_ptr,
                pivot_ptr,
                s_size,
                is_less,
            );
        }

        if s_size <= (a_size / 16) || a_size <= FLUX_OUT {
            // 			return FUNC(quadsort_swap)(array, swap, a_size, a_size, cmp);
            todo!()
        }

        len = a_size;
        x_ptr = arr_ptr;
    }
}
