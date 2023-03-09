//! Slice trait to add more methods.

use std::slice::from_raw_parts_mut;

/// The `SliceExt` trait add extra methods to standard slices.
pub trait SliceExt<T> {
    /// Divide one slice into a mutable prefix, a mutable suffix, plus the pivot
    /// item.
    ///
    /// The prefix will contain all indices from `[0, index)` (excluding the
    /// index `mid` itself), and the second will contain all indices from
    /// `[index + 1, len)` (excluding the index `len` itself).
    ///
    /// It returns `None` if the slice is empty.
    ///
    /// # Safety
    ///
    /// This method  takes `&self`, not `&mut self`, but it returns mutable
    /// references to prefix and suffix. The caller must ensure the prefix and
    /// suffix are never stored anywhere or don't outlive `self`.
    ///
    /// # Panics
    ///
    /// Panics if `index > len - 1`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use weld_object::slice::SliceExt;
    ///
    /// let vec = vec![1, 2, 3, 4, 5];
    /// let (prefix, pivot, suffix) =
    ///     unsafe { vec.split_around_at_mut(2).unwrap() };
    ///
    /// assert_eq!(prefix, [1, 2]);
    /// assert_eq!(pivot, &3);
    /// assert_eq!(suffix, [4, 5]);
    ///
    /// prefix[0] += 10;
    /// prefix[1] += 10;
    /// suffix[0] += 10;
    /// suffix[1] += 10;
    ///
    /// assert_eq!(vec, [11, 12, 3, 14, 15]);
    /// ```
    unsafe fn split_around_at_mut(&self, index: usize) -> Option<(&mut [T], &T, &mut [T])>;
}

impl<T> SliceExt<T> for [T] {
    unsafe fn split_around_at_mut(&self, index: usize) -> Option<(&mut [T], &T, &mut [T])> {
        let len = self.len();

        if len == 0 {
            return None;
        }

        assert!(index < len);

        let ptr = self.as_ptr() as *mut _;

        // SAFETY: `[ptr; index]` and `[index + 1; len]` are inside `self`,
        // which fulfills the requirements of `from_raw_parts_mut`. `index`
        // is also part of `self` and can be dereferenced safely as it's
        // guaranteed to not be `null`.
        Some(unsafe {
            (
                from_raw_parts_mut(ptr, index),
                &mut *ptr.add(index),
                from_raw_parts_mut(ptr.add(index + 1), len - index - 1),
            )
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_around_at_mut() {
        // Casual.
        {
            let vec = vec![1, 2, 3, 4, 5];
            let (prefix, pivot, suffix): (&mut [_], &_, &mut [_]) =
                unsafe { vec.split_around_at_mut(2) }.unwrap();

            assert_eq!(prefix, [1, 2]);
            assert_eq!(pivot, &3);
            assert_eq!(suffix, [4, 5]);

            prefix[0] += 10;
            prefix[1] += 10;
            suffix[0] += 10;
            suffix[1] += 10;

            assert_eq!(vec, [11, 12, 3, 14, 15]);
        }

        // Middle.
        {
            let vec = vec![1, 2, 3];

            {
                let (prefix, pivot, suffix) = unsafe { vec.split_around_at_mut(0) }.unwrap();

                assert_eq!(prefix, []);
                assert_eq!(pivot, &1);
                assert_eq!(suffix, [2, 3]);
            }

            {
                let (prefix, pivot, suffix) = unsafe { vec.split_around_at_mut(1) }.unwrap();

                assert_eq!(prefix, [1]);
                assert_eq!(pivot, &2);
                assert_eq!(suffix, [3]);
            }

            {
                let (prefix, pivot, suffix) = unsafe { vec.split_around_at_mut(2) }.unwrap();

                assert_eq!(prefix, [1, 2]);
                assert_eq!(pivot, &3);
                assert_eq!(suffix, []);
            }
        }

        // Small.
        {
            let vec = vec![1, 2];

            {
                let (prefix, pivot, suffix) = unsafe { vec.split_around_at_mut(0) }.unwrap();

                assert_eq!(prefix, []);
                assert_eq!(pivot, &1);
                assert_eq!(suffix, [2]);
            }

            {
                let (prefix, pivot, suffix) = unsafe { vec.split_around_at_mut(1) }.unwrap();

                assert_eq!(prefix, [1]);
                assert_eq!(pivot, &2);
                assert_eq!(suffix, []);
            }
        }

        // Tiny.
        {
            let vec = vec![1];
            let (prefix, pivot, suffix) = unsafe { vec.split_around_at_mut(0) }.unwrap();

            assert_eq!(prefix, []);
            assert_eq!(pivot, &1);
            assert_eq!(suffix, []);
        }

        // Empty.
        {
            let vec: Vec<u8> = vec![];

            assert!(unsafe { vec.split_around_at_mut(0) }.is_none());
        }
    }

    #[test]
    #[should_panic]
    fn test_split_around_at_mut_out_of_bound() {
        let vec = vec![1, 2, 3];
        let _ = unsafe { vec.split_around_at_mut(vec.len()) };
    }
}
