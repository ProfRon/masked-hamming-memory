use core::{mem, slice};

#[inline]
pub fn get_bit(bytes: &[u8], bit_index: usize) -> bool {
    let byte_index = bit_index / 8;
    debug_assert!(byte_index < bytes.len());
    let mask_index = bit_index % 8;
    let bit_mask: u8 = 1u8 << mask_index;
    // Now, the result....
    0 != bytes[byte_index] & bit_mask
}

#[inline]
pub fn put_bit(bytes: &mut [u8], bit_index: usize, new_value: bool) {
    let byte_index = bit_index / 8;
    debug_assert!(byte_index < bytes.len());
    let mask_index = bit_index % 8;
    let bit_mask: u8 = 1u8 << mask_index;
    let not_bit_mask: u8 = !bit_mask;

    bytes[byte_index] &= not_bit_mask; // bitwise AND -- clears bit

    if new_value {
        bytes[byte_index] |= bit_mask; // boolean OR to set bit... maybe!
    } // bitwise OR -- sets bit, maybe
}

/// Reinterpret as much of `x` as a slice of (correctly aligned) `U`s
/// as possible. (Same as `slice::align_to` but available in earlier
/// compilers.)
/// # Safety
/// Tricky, this...
#[inline(never)] // critical for autovectorization in `weight`.
pub unsafe fn align_to<T, U>(x: &[T]) -> (&[T], &[U], &[T]) {
    let orig_size = mem::size_of::<T>();
    let size = mem::size_of::<U>();

    debug_assert!(orig_size < size);
    debug_assert!(size % orig_size == 0);
    let size_ratio = size / orig_size;

    let alignment = mem::align_of::<U>();

    let ptr = x.as_ptr() as usize;
    // round up to the nearest multiple
    let aligned = (ptr + alignment - 1) / alignment * alignment;
    let byte_distance = aligned - ptr;

    // can't fit a single U in
    if mem::size_of_val(x) < size + byte_distance {
        return (x, &[], &[]);
    }

    let (head, middle) = x.split_at(byte_distance / orig_size);

    debug_assert!(middle.as_ptr() as usize % alignment == 0);
    let cast_middle = slice::from_raw_parts(middle.as_ptr() as *const U, middle.len() / size_ratio);
    let tail = &middle[cast_middle.len() * size_ratio..];

    (head, cast_middle, tail)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_and_put_bit() {
        let mut buncha_bits = vec![0u8; 8]; // 64 bits
        assert!(!get_bit(&buncha_bits, 0));
        assert!(!get_bit(&buncha_bits, 7));
        assert!(!get_bit(&buncha_bits, 21));
        assert!(!get_bit(&buncha_bits, 28));
        assert!(!get_bit(&buncha_bits, 63));

        put_bit(&mut buncha_bits, 0, true);
        put_bit(&mut buncha_bits, 42, true);
        put_bit(&mut buncha_bits, 63, true);
        assert!(get_bit(&buncha_bits, 0));
        assert!(get_bit(&buncha_bits, 42));
        assert!(get_bit(&buncha_bits, 63));

        put_bit(&mut buncha_bits, 0, false);
        put_bit(&mut buncha_bits, 42, false);
        put_bit(&mut buncha_bits, 63, false);
        assert!(!get_bit(&buncha_bits, 0));
        assert!(!get_bit(&buncha_bits, 42));
        assert!(!get_bit(&buncha_bits, 63));
    }

    fn align_to_test(
        from: usize,
        to: usize,
        true_head: &[u8],
        true_le_middle: &[u32],
        true_tail: &[u8],
    ) {
        let true_middle = true_le_middle
            .iter()
            .map(|x| u32::from_le(*x))
            .collect::<Vec<_>>();

        let array_and_tuple = (0u64, [0u8, 1, 2, 3, 4, 5, 6, 7, 8, 9]);
        let array = &array_and_tuple.1;
        // the array should be aligned appropriately
        assert!((array.as_ptr() as usize) % 4 == 0);

        let (head, middle, tail) = unsafe { align_to::<_, u32>(&array[from..to]) };
        assert_eq!(head, true_head);
        assert_eq!(middle, true_middle.as_slice());
        assert_eq!(tail, true_tail);
    }

    #[test]
    fn align_to_empty() {
        align_to_test(0, 0, &[], &[], &[]);
        align_to_test(1, 1, &[], &[], &[]);
        align_to_test(2, 2, &[], &[], &[]);
        align_to_test(3, 3, &[], &[], &[]);
    }

    #[test]
    fn align_to_short() {
        align_to_test(0, 1, &[0], &[], &[]);
        align_to_test(1, 2, &[1], &[], &[]);
        align_to_test(2, 3, &[2], &[], &[]);
        align_to_test(3, 4, &[3], &[], &[]);

        align_to_test(0, 2, &[0, 1], &[], &[]);
        align_to_test(1, 3, &[1, 2], &[], &[]);
        align_to_test(2, 4, &[2, 3], &[], &[]);
        align_to_test(3, 5, &[3, 4], &[], &[]);

        align_to_test(0, 3, &[0, 1, 2], &[], &[]);
        align_to_test(1, 4, &[1, 2, 3], &[], &[]);
        align_to_test(2, 5, &[2, 3, 4], &[], &[]);
        align_to_test(3, 6, &[3, 4, 5], &[], &[]);
    }

    #[test]
    fn align_to_exact() {
        align_to_test(0, 4, &[], &[0x03020100], &[]);
        align_to_test(0, 8, &[], &[0x03020100, 0x07060504], &[]);
    }

    #[test]
    fn align_to_offset() {
        align_to_test(1, 5, &[1, 2, 3, 4], &[], &[]);
        align_to_test(2, 6, &[2, 3, 4, 5], &[], &[]);
        align_to_test(3, 7, &[3, 4, 5, 6], &[], &[]);
        align_to_test(1, 7, &[1, 2, 3, 4, 5, 6], &[], &[]);
    }

    #[test]
    fn align_to_overlap() {
        align_to_test(0, 10, &[], &[0x03020100, 0x07060504], &[8, 9]);
        align_to_test(0, 5, &[], &[0x03020100], &[4]);

        align_to_test(1, 8, &[1, 2, 3], &[0x07060504], &[]);
        align_to_test(3, 9, &[3], &[0x07060504], &[8]);
    }

    #[cfg_attr(not(debug_assertions), ignore)]
    #[test]
    #[should_panic]
    fn align_to_smaller() {
        let _ = unsafe { align_to::<u64, u8>(&[]) };
    }

    #[cfg_attr(not(debug_assertions), ignore)]
    #[test]
    #[should_panic]
    fn align_to_nondivisible() {
        let _ = unsafe { align_to::<[u8; 2], [u8; 3]>(&[]) };
    }
}
