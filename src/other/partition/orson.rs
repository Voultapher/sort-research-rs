use core::mem::ManuallyDrop;
use core::ptr;

partition_impl!("orson");

fn partition<T, F: FnMut(&T, &T) -> bool>(v: &mut [T], pivot: &T, is_less: &mut F) -> usize {
    let len = v.len();
    let v_base = v.as_mut_ptr();

    if len == 0 {
        return 0;
    }

    // SAFETY: TODO
    unsafe {
        let mut tmp = ManuallyDrop::new(ptr::read(v_base.add(len - 1)));
        let mut left = 0;
        let mut right = len - 1;

        while right > 1 {
            right -= 1;
            let tmp_is_lt = is_less(&*tmp, pivot);
            let offset = if tmp_is_lt { left } else { left + right };
            let tmp2 = ManuallyDrop::new(ptr::read(v_base.add(offset)));
            ptr::copy_nonoverlapping(&mut *tmp, v_base.add(offset), 1);
            left += tmp_is_lt as usize;

            right -= 1;
            let tmp2_is_lt = is_less(&*tmp2, pivot);
            let offset2 = if tmp2_is_lt { left } else { left + right };
            ptr::copy_nonoverlapping(v_base.add(offset2), &mut *tmp, 1);
            ptr::copy_nonoverlapping(&*tmp2, v_base.add(offset2), 1);
            left += tmp2_is_lt as usize;
        }

        if right > 0 {
            right -= 1;
            let tmp_is_lt = is_less(&*tmp, pivot);
            let offset = if tmp_is_lt { left } else { left + right };
            ptr::swap(&mut *tmp, v_base.add(offset));
            left += tmp_is_lt as usize;
        }

        let tmp_is_lt = is_less(&*tmp, pivot);
        ptr::copy(v_base.add(left), v_base.add(len - 1), 1);
        ptr::write(v_base.add(left), ManuallyDrop::into_inner(tmp));
        left += tmp_is_lt as usize;

        left
    }
}
