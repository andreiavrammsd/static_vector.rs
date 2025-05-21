#![no_std]
#![deny(missing_docs)]
#![doc = include_str!("../README.md")]

use core::{array, error, fmt, mem::MaybeUninit};

/// Attempted to push to a full vector
#[derive(Debug)]
#[non_exhaustive]
pub struct CapacityExceededError;

impl fmt::Display for CapacityExceededError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("attempted to push to a full vector")
    }
}

impl error::Error for CapacityExceededError {}

/// Attempted to resize the vector to a length greater than its fixed capacity.
#[derive(Debug)]
#[non_exhaustive]
pub struct LengthTooLargeError;

impl fmt::Display for LengthTooLargeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("attempted to resize the vector to a length greater than its fixed capacity")
    }
}

impl error::Error for LengthTooLargeError {}

/// A stack-allocated vector with fixed capacity and dynamic length.
///
/// See crate-level documentation for details and usage.
pub struct Vec<T: Clone, const CAPACITY: usize> {
    data: [MaybeUninit<T>; CAPACITY],
    length: usize,
}

impl<T: Clone, const CAPACITY: usize> Default for Vec<T, CAPACITY> {
    /// Creates an empty [`Vec`]. Equivalent to [`Vec::new()`].
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Clone, const CAPACITY: usize> Vec<T, CAPACITY> {
    /// Creates a new empty [`Vec`] with maximum `CAPACITY` elements of type `T`.
    #[must_use]
    #[inline]
    pub fn new() -> Self {
        let data: [MaybeUninit<T>; CAPACITY] = array::from_fn(|_| MaybeUninit::uninit());
        Self { data, length: 0 }
    }

    /// Returns the maximum number of elements the vector can contain.
    #[must_use]
    #[inline]
    #[doc(alias("max", "size", "limit", "length"))]
    pub const fn capacity(&self) -> usize {
        CAPACITY
    }

    /// Returns the maximum number of elements the vector currenly contains.
    #[must_use]
    #[inline]
    #[doc(alias("length", "size"))]
    pub const fn len(&self) -> usize {
        self.length
    }

    /// Returns whether the vector has no elements or any.
    #[must_use]
    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.length == 0
    }

    /// Returns whether the vector is at maximum capacity.
    #[must_use]
    #[inline]
    pub const fn is_full(&self) -> bool {
        self.len() == self.capacity()
    }

    /// Adds a clone of the given `value` to the end of the vector.
    ///
    /// # Errors
    ///
    /// Returns [`CapacityExceededError`] if the vector is already at full capacity.
    #[inline]
    #[doc(alias("add", "append", "insert"))]
    pub fn push(&mut self, value: &T) -> Result<(), CapacityExceededError> {
        if self.is_full() {
            return Err(CapacityExceededError);
        }

        self.data[self.length].write(value.clone());
        self.length += 1;

        Ok(())
    }

    /// Removes all elements. Size will be zero.
    #[inline]
    #[doc(alias("reset", "remove", "truncate", "empty"))]
    pub fn clear(&mut self) {
        self.drop_range(0, self.length);
        self.length = 0;
    }

    /// Resizes the vector to the `new_length`.
    ///
    /// # Requirements
    ///
    /// - `T` must implement `Default` because new elements are created with `T::default()`
    ///   when increasing the length.
    ///
    /// # Errors
    ///
    /// Returns [`LengthTooLargeError`] if `new_length` exceeds the vector's fixed capacity.
    #[doc(alias("resize", "length"))]
    pub fn set_len(&mut self, new_length: usize) -> Result<(), LengthTooLargeError>
    where
        T: Default,
    {
        if new_length > CAPACITY {
            return Err(LengthTooLargeError);
        }

        if new_length > self.length {
            for i in self.length..new_length {
                self.data[i].write(T::default());
            }
        } else {
            self.drop_range(new_length, self.length);
        }

        self.length = new_length;
        Ok(())
    }

    /// Returns a reference to the first element in the vector, or [`None`] if the vector is empty.
    #[must_use]
    #[inline]
    #[doc(alias("front", "head", "start"))]
    pub const fn first(&self) -> Option<&T> {
        if self.is_empty() {
            None
        } else {
            // SAFETY:
            // We ensure that:
            // - `0` is within bounds of `self.data`.
            // - The element at `0` has been initialized.
            Some(unsafe { &*self.data[0].as_ptr() })
        }
    }

    /// Returns a reference to the last element in the vector, or [`None`] if the vector is empty.
    #[must_use]
    #[inline]
    #[doc(alias("back", "tail", "end"))]
    pub const fn last(&self) -> Option<&T> {
        if self.is_empty() {
            None
        } else {
            // SAFETY:
            // We ensure that:
            // - `self.length - 1` is within bounds of `self.data`.
            // - The element at `self.length - 1` has been initialized.
            Some(unsafe { &*self.data[self.length - 1].as_ptr() })
        }
    }

    /// Returns a reference to the element at the specified `index`, or [`None`] if out of bounds.
    #[must_use]
    #[inline]
    #[doc(alias("at", "index"))]
    pub const fn get(&self, index: usize) -> Option<&T> {
        if index >= self.length {
            None
        } else {
            // SAFETY:
            // We ensure that:
            // - `index` is within bounds of `self.data`.
            // - The element at `index` has been initialized.
            Some(unsafe { &*self.data[index].as_ptr() })
        }
    }

    /// Returns a mutable reference to the element at the specified `index`, or [`None`] if out of bounds.
    #[must_use]
    #[inline]
    #[doc(alias("at", "index"))]
    pub const fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        if index >= self.length {
            None
        } else {
            // SAFETY:
            // We ensure that:
            // - `index` is within bounds of `self.data`.
            // - The element at `index` has been initialized.
            Some(unsafe { &mut *self.data[index].as_mut_ptr() })
        }
    }

    /// Returns (and removes) the last element from the vector, or [`None`] if the vector is empty.
    #[must_use]
    #[inline]
    #[doc(alias("remove", "get"))]
    pub const fn pop(&mut self) -> Option<T> {
        if self.is_empty() {
            None
        } else {
            self.length -= 1;

            // SAFETY:
            // We ensure that:
            // - `self.length` is within bounds of `self.data`.
            // - The element at `self.length` has been initialized.
            Some(unsafe { self.data[self.length].assume_init_read() })
        }
    }

    /// Returns (and removes) the last element from the vector if the predicate returns true,
    /// or [`None`] if the vector is empty or the predicate returns false.
    #[must_use]
    #[inline]
    #[doc(alias("remove", "get"))]
    pub fn pop_if<F: FnOnce(&T) -> bool>(&mut self, predicate: F) -> Option<T> {
        let last = self.last()?;
        if predicate(last) { self.pop() } else { None }
    }

    /// Returns an iterator over immutable references to the elements in the vector.
    #[inline]
    pub const fn iter(&self) -> Iter<'_, T> {
        Iter::new(&self.data, self.length)
    }

    /// Returns an iterator over mutable references to the elements in the vector.
    #[inline]
    pub const fn iter_mut(&mut self) -> IterMut<'_, T> {
        IterMut::new(&mut self.data, self.length)
    }

    /// Drops all elements in given range. Needed when elements are considered to be going out of scope.
    /// E.g.: when the vector is going out of scope, when methods such as [`Vec::clear()`] and [`Vec::set_len()`] are called.
    fn drop_range(&mut self, from: usize, to: usize) {
        for i in from..to {
            // SAFETY:
            // We ensure that:
            // - `i` is within bounds of `self.data`.
            // - The element at `i` has been initialized.
            unsafe {
                self.data[i].assume_init_drop();
            }
        }
    }
}

impl<T: Clone, const CAPACITY: usize> Drop for Vec<T, CAPACITY> {
    fn drop(&mut self) {
        self.drop_range(0, self.length);
    }
}

/// Immutable iterator over a [`Vec`].
///
/// Created by calling [`Vec::iter()`].
#[must_use = "must consume iterator"]
pub struct Iter<'a, T> {
    data: &'a [MaybeUninit<T>],
    size: usize,
    index: usize,
}

impl<'a, T> Iter<'a, T> {
    /// Creates immutable iterator.
    #[inline]
    pub const fn new(data: &'a [MaybeUninit<T>], size: usize) -> Self {
        Self { data, size, index: 0 }
    }
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index == self.size {
            None
        } else {
            // SAFETY:
            // We ensure that:
            // - `self.index` is within bounds of `self.data`.
            // - The element at `self.index` has been initialized.
            let value = unsafe { &*self.data[self.index].as_ptr() };
            self.index += 1;
            Some(value)
        }
    }
}

impl<'a, T: 'a + Clone, const CAPACITY: usize> IntoIterator for &'a Vec<T, CAPACITY> {
    type Item = &'a T;
    type IntoIter = Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

/// Mutable iterator over a [`Vec`].
///
/// Created by calling [`Vec::iter_mut()`].
#[must_use = "must consume iterator"]
pub struct IterMut<'a, T> {
    data: &'a mut [MaybeUninit<T>],
    size: usize,
    index: usize,
}

impl<'a, T> IterMut<'a, T> {
    /// Creates mutable iterator.
    #[inline]
    pub const fn new(data: &'a mut [MaybeUninit<T>], size: usize) -> Self {
        Self { data, size, index: 0 }
    }
}

impl<'a, T> Iterator for IterMut<'a, T> {
    type Item = &'a mut T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index == self.size {
            None
        } else {
            // SAFETY:
            // We ensure that:
            // - `self.index` is within bounds of `self.data`.
            // - The element at `self.index` has been initialized.
            let value = unsafe { &mut *self.data[self.index].as_mut_ptr() };

            self.index += 1;
            Some(value)
        }
    }
}

impl<'a, T: 'a + Clone, const CAPACITY: usize> IntoIterator for &'a mut Vec<T, CAPACITY> {
    type Item = &'a mut T;
    type IntoIter = IterMut<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    extern crate alloc;
    extern crate std;
    use alloc::format;
    use core::cell::Cell;
    use core::error::Error;
    use std::thread_local;

    fn assert_is_core_error<T: Error>() {}

    #[test]
    fn construct() {
        assert!(Vec::<i32, 3>::new().is_empty());
        assert!(Vec::<i32, 3>::default().is_empty());
    }

    #[test]
    fn zero_capacity() {
        let mut empty = Vec::<i32, 0>::new();
        assert_eq!(empty.capacity(), 0);
        assert_eq!(empty.len(), 0);
        assert!(empty.is_empty());
        assert!(empty.is_full());
        assert!(empty.push(&0).is_err());
        assert!(empty.set_len(0).is_ok());
        assert!(empty.set_len(1).is_err());
        assert!(empty.first().is_none());
        assert!(empty.last().is_none());
        assert!(empty.get(0).is_none());
        assert!(empty.get_mut(0).is_none());
        assert!(empty.pop().is_none());
        assert_eq!(empty.iter().count(), 0);
    }

    #[test]
    fn capacity() {
        let mut vec = Vec::<i32, 3>::new();

        assert_eq!(vec.capacity(), 3);

        vec.set_len(2).unwrap();
        assert_eq!(vec.capacity(), 3);

        vec.push(&1).unwrap();
        assert_eq!(vec.capacity(), 3);
    }

    #[test]
    fn push() {
        let mut vec = Vec::<i32, 2>::new();
        assert!(vec.push(&1).is_ok());
        assert!(vec.push(&2).is_ok());

        assert!(matches!(vec.push(&3), Err(CapacityExceededError)));
        assert_eq!(format!("{}", vec.push(&3).unwrap_err()), "attempted to push to a full vector");
        assert_is_core_error::<CapacityExceededError>();

        assert_eq!(vec.get(0).unwrap(), &1);
        assert_eq!(vec.get(1).unwrap(), &2);
        assert!(vec.get(2).is_none());
    }

    #[test]
    fn size() {
        let mut vec = Vec::<i32, 3>::new();
        assert_eq!(vec.len(), 0);
        assert!(vec.is_empty());
        assert!(!vec.is_full());

        vec.push(&1).unwrap();
        vec.push(&2).unwrap();
        assert_eq!(vec.len(), 2);
        assert!(!vec.is_empty());
        assert!(!vec.is_full());

        assert!(vec.set_len(1).is_ok());
        assert_eq!(vec.len(), 1);
        assert!(!vec.is_empty());
        assert!(!vec.is_full());

        assert!(matches!(vec.set_len(100), Err(LengthTooLargeError)));
        assert_eq!(
            format!("{}", vec.set_len(100).unwrap_err()),
            "attempted to resize the vector to a length greater than its fixed capacity"
        );
        assert_is_core_error::<LengthTooLargeError>();
        assert!(!vec.is_full());

        vec.clear();
        assert!(!vec.is_full());
        assert_eq!(vec.len(), 0);
        assert!(vec.is_empty());

        vec.set_len(vec.capacity()).unwrap();
        assert!(vec.is_full());
    }

    #[test]
    fn get() {
        let mut vec = Vec::<i32, 4>::new();
        assert!(vec.first().is_none());
        assert!(vec.last().is_none());
        assert!(vec.get(0).is_none());

        vec.push(&1).unwrap();
        assert_eq!(vec.first().unwrap(), &1);
        assert_eq!(vec.get(0).unwrap(), &1);
        assert_eq!(vec.last().unwrap(), &1);

        vec.push(&2).unwrap();
        vec.push(&3).unwrap();
        assert_eq!(vec.first().unwrap(), &1);
        assert_eq!(vec.last().unwrap(), &3);
        assert_eq!(vec.get(0).unwrap(), &1);
        assert_eq!(vec.get(1).unwrap(), &2);
        assert_eq!(vec.get(2).unwrap(), &3);
        assert!(vec.get(3).is_none());

        assert_eq!(vec.get_mut(0).unwrap(), &1);
        *vec.get_mut(0).unwrap() = 5;
        assert_eq!(vec.get(0).unwrap(), &5);
        assert_eq!(vec.get_mut(0).unwrap(), &5);
        assert!(vec.get_mut(3).is_none());
    }

    #[test]
    fn pop() {
        let mut vec = Vec::<Struct, 4>::new();
        assert!(vec.pop().is_none());

        let s1 = Struct { i: 1 };
        vec.push(&s1).unwrap();

        let s2 = Struct { i: 2 };
        vec.push(&s2).unwrap();

        let s3 = Struct { i: 3 };
        vec.push(&s3).unwrap();

        assert_eq!(vec.pop().unwrap().i, 3);
        assert_eq!(vec.len(), 2);
        assert_eq!(DROPS.get(), 1);

        assert_eq!(vec.pop().unwrap().i, 2);
        assert_eq!(vec.pop().unwrap().i, 1);
        assert!(vec.is_empty());
        assert!(vec.pop().is_none());
        assert_eq!(DROPS.get(), 3);

        assert_eq!(DEFAULTS.get(), 0);
        assert_eq!(CLONES.get(), 3); // from the three pushes
    }

    fn not<F>(f: F) -> impl Fn(&Struct) -> bool
    where
        F: Fn(&Struct) -> bool,
    {
        move |s| !f(s)
    }

    #[test]
    fn pop_if() {
        let is_even = |s: &Struct| s.i % 2 == 0;

        let mut vec = Vec::<Struct, 4>::new();
        assert!(vec.pop_if(is_even).is_none());

        let s1 = Struct { i: 1 };
        vec.push(&s1).unwrap();

        let s2 = Struct { i: 2 };
        vec.push(&s2).unwrap();

        let s3 = Struct { i: 3 };
        vec.push(&s3).unwrap();

        assert!(vec.pop_if(is_even).is_none());
        assert_eq!(vec.len(), 3);
        assert_eq!(DROPS.get(), 0);

        assert_eq!(vec.pop_if(not(is_even)).unwrap().i, 3);
        assert_eq!(vec.len(), 2);
        assert_eq!(DROPS.get(), 1);

        assert!(vec.pop_if(not(is_even)).is_none());
        assert_eq!(vec.len(), 2);
        assert_eq!(DROPS.get(), 1);

        assert_eq!(vec.pop_if(is_even).unwrap().i, 2);
        assert_eq!(vec.len(), 1);
        assert_eq!(DROPS.get(), 2);

        assert_eq!(vec.pop_if(not(is_even)).unwrap().i, 1);
        assert!(vec.is_empty());
        assert_eq!(DROPS.get(), 3);

        assert!(vec.pop_if(is_even).is_none());
        assert!(vec.is_empty());
        assert_eq!(DROPS.get(), 3);

        assert_eq!(DEFAULTS.get(), 0);
        assert_eq!(CLONES.get(), 3); // from the three pushes
    }

    #[test]
    fn iter() {
        let mut vec = Vec::<i32, 10>::new();
        for i in 1..8 {
            vec.push(&i).unwrap();
        }

        let even_sum = vec.iter().filter(|v| *v % 2 == 0).sum::<i32>();
        assert_eq!(even_sum, 12);

        assert_eq!(vec.iter().count(), 7);
    }

    #[test]
    fn into_iter() {
        let mut vec = Vec::<i32, 10>::new();
        for i in 1..8 {
            vec.push(&i).unwrap();
        }

        let mut s = 0;
        for i in &vec {
            s += i;
        }
        assert_eq!(s, 28);
    }

    #[test]
    fn iter_mut() {
        let mut vec = Vec::<i32, 10>::new();
        for i in 1..8 {
            vec.push(&i).unwrap();
        }

        let even_sum = vec.iter_mut().filter(|v| **v % 2 == 0).map(|v| *v).sum::<i32>();
        assert_eq!(even_sum, 12);

        assert_eq!(vec.iter().count(), 7);
    }

    #[test]
    fn into_iter_mut() {
        let mut vec = Vec::<i32, 10>::new();
        for i in 1..8 {
            vec.push(&i).unwrap();
        }

        let mut s = 0;
        for i in &mut vec {
            *i *= 2;
            s += *i;
        }
        assert_eq!(s, 56);
    }

    #[test]
    fn construct_should_not_create_default_elements() {
        let _ = Vec::<Struct, 10>::new();
        assert_eq!(DEFAULTS.get(), 0);
    }

    #[test]
    fn push_should_not_create_default_elements() {
        let mut vec = Vec::<Struct, 10>::new();
        vec.push(&Struct { i: 0 }).unwrap();
        assert_eq!(DEFAULTS.get(), 0);
    }

    #[test]
    fn set_len_should_create_default_elements() {
        let mut vec = Vec::<Struct, 10>::new();

        // Length zero, no defaults
        vec.set_len(0).unwrap();
        assert_eq!(DEFAULTS.get(), 0);

        // Length error, no defaults
        vec.set_len(99).unwrap_err();
        assert_eq!(DEFAULTS.get(), 0);

        // Maximum length, create `CAPACITY` default values
        vec.set_len(10).unwrap();
        assert_eq!(DEFAULTS.get(), 10);

        // Smaller length than current, no defaults
        DEFAULTS.set(0);
        vec.set_len(5).unwrap();
        assert_eq!(DEFAULTS.get(), 0);

        // Larger length than current, create `current length - new length` default values
        DEFAULTS.set(0);
        vec.set_len(8).unwrap();
        assert_eq!(DEFAULTS.get(), 3);
    }

    #[test]
    fn push_should_clone_element() {
        let mut vec = Vec::<Struct, 10>::new();

        vec.push(&Struct { i: 0 }).unwrap();
        assert_eq!(CLONES.get(), 1);

        vec.push(&Struct { i: 0 }).unwrap();
        vec.push(&Struct { i: 0 }).unwrap();
        assert_eq!(CLONES.get(), 3);
    }

    #[test]
    fn clear_should_drop_all_allocated_elements() {
        let mut vec = Vec::<Struct, 10>::new();
        assert_eq!(DROPS.get(), 0);

        let s = Struct { i: 0 };
        for _ in 1..4 {
            vec.push(&s).unwrap();
        }
        assert_eq!(DROPS.get(), 0);

        vec.clear();
        assert_eq!(DROPS.get(), 3);
    }

    #[test]
    fn set_len_should_drop_all_allocated_elements() {
        let mut vec = Vec::<Struct, 10>::new();
        assert_eq!(DROPS.get(), 0);

        let s = Struct { i: 0 };
        for _ in 1..6 {
            vec.push(&s).unwrap();
        }
        assert_eq!(DROPS.get(), 0);

        // Same length, no drops
        vec.set_len(5).unwrap();
        assert_eq!(DROPS.get(), 0);

        // Length error, no drop
        vec.set_len(999).unwrap_err();
        assert_eq!(DROPS.get(), 0);

        // Length smaller, drop elements after
        vec.set_len(2).unwrap();
        assert_eq!(DROPS.get(), 3);

        // Same length again, no change in number of drops
        vec.set_len(2).unwrap();
        assert_eq!(DROPS.get(), 3);

        // Length zero, drop all
        DROPS.set(0);
        vec.set_len(0).unwrap();
        assert_eq!(DROPS.get(), 2);
    }

    #[test]
    fn going_out_of_scope_should_drop_all_allocated_elements() {
        let s = Struct { i: 0 };

        {
            let mut vec = Vec::<Struct, 10>::new();
            assert_eq!(DROPS.get(), 0);

            for _ in 1..4 {
                vec.push(&s).unwrap();
            }
            assert_eq!(DROPS.get(), 0);
        };

        assert_eq!(DROPS.get(), 3);
    }

    struct Struct {
        i: i32,
    }

    thread_local! {
        static DEFAULTS: Cell<usize> = const {Cell::new(0)};
        static CLONES: Cell<usize> = const {Cell::new(0)};
        static DROPS: Cell<usize> = const {Cell::new(0)};
    }

    impl Default for Struct {
        fn default() -> Self {
            DEFAULTS.set(DEFAULTS.get() + 1);
            Self { i: 0 }
        }
    }

    impl Clone for Struct {
        fn clone(&self) -> Self {
            CLONES.set(CLONES.get() + 1);
            Self { i: self.i }
        }
    }

    impl Drop for Struct {
        fn drop(&mut self) {
            DROPS.set(DROPS.get() + 1);
        }
    }
}
