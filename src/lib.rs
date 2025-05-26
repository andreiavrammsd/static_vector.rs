#![no_std]
#![deny(missing_docs)]
#![doc = include_str!("../README.md")]

use core::mem::MaybeUninit;
use core::{error, fmt, slice};

/// Error for when the vector is full or the requested operation would need more space than the capacity.
///
/// See [`Vec::push()`] example for usage.
#[derive(Debug)]
#[non_exhaustive]
pub struct CapacityError;

impl fmt::Display for CapacityError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("vector needs larger capacity")
    }
}

impl error::Error for CapacityError {}

/// A stack-allocated vector with fixed capacity and dynamic length.
pub struct Vec<T, const CAPACITY: usize> {
    data: [MaybeUninit<T>; CAPACITY],
    length: usize,
}

impl<T, const CAPACITY: usize> Default for Vec<T, CAPACITY> {
    /// Creates an empty [`Vec`]. Equivalent to [`Vec::new()`].
    ///
    /// # Panics
    ///
    /// Panics if `CAPACITY == 0`. Zero-capacity vectors are not supported.
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<T, const CAPACITY: usize> Vec<T, CAPACITY> {
    /// Creates a new empty [`Vec`] with maximum `CAPACITY` elements of type `T`.
    ///
    /// # Panics
    ///
    /// Panics if `CAPACITY == 0`. Zero-capacity vectors are not supported.
    ///
    /// # Example
    ///
    /// ```rust
    /// use static_vector::Vec;
    ///
    /// let mut vec = Vec::<i32, 20>::new();
    /// vec.as_mut_slice().fill(0);
    /// // now vec has length 20 and all elements are zero
    /// ```
    #[must_use]
    #[inline]
    pub const fn new() -> Self {
        assert!(CAPACITY > 0, "CAPACITY must be greater than 0");

        // SAFETY: The elements in the array are not accessed before beign initialized.
        let data = unsafe { MaybeUninit::<[MaybeUninit<T>; CAPACITY]>::uninit().assume_init() };
        Self { data, length: 0 }
    }

    /// Returns the maximum number of elements the vector can contain.
    ///
    /// # Example
    ///
    /// ```rust
    /// use static_vector::Vec;
    ///
    /// let vec = Vec::<i32, 10>::new();
    /// const SOME_LIMIT: usize = 5;
    ///
    /// if vec.len() < vec.capacity() - SOME_LIMIT {
    ///     // do something
    /// }
    /// ```
    #[must_use]
    #[inline]
    #[doc(alias("max", "size", "limit", "length"))]
    pub const fn capacity(&self) -> usize {
        CAPACITY
    }

    /// Returns the number of elements the vector currently contains.
    ///
    /// # Example
    ///
    /// ```rust
    /// use static_vector::Vec;
    ///
    /// let mut vec = Vec::<i32, 10>::new();
    /// const SOME_LIMIT: usize = 5;
    ///
    /// if vec.len() < SOME_LIMIT {
    ///     // do something
    /// }
    /// ```
    #[must_use]
    #[inline]
    #[doc(alias("length", "size"))]
    pub const fn len(&self) -> usize {
        self.length
    }

    /// Returns whether the vector has no elements.
    ///
    /// # Example
    ///
    /// ```rust
    /// use static_vector::Vec;
    ///
    /// let mut vec = Vec::<i32, 10>::new();
    ///
    /// if vec.is_empty() {
    ///     // do something
    /// }
    /// ```
    #[must_use]
    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.length == 0
    }

    /// Returns whether the vector is at maximum capacity.
    ///
    ///
    /// # Example
    ///
    /// ```rust
    /// use static_vector::Vec;
    ///
    /// let mut vec = Vec::<i32, 20>::new();
    ///
    /// if vec.is_full() {
    ///     // cannot push elements anymore
    /// }
    /// ```
    #[must_use]
    #[inline]
    pub const fn is_full(&self) -> bool {
        self.len() == self.capacity()
    }

    /// Adds the given `value` to the end of the vector.
    ///
    /// # Errors
    ///
    /// Returns [`CapacityError`] if the vector is already at full capacity.
    ///
    /// # Example
    ///
    /// ```rust
    /// use static_vector::{CapacityError, Vec};
    ///
    /// #[derive(Debug)]
    /// enum AppError {
    ///     VectorCapacityError(CapacityError),
    /// }
    ///
    /// fn my_fn(vec: &mut Vec<i32, 2>) -> Result<(), AppError> {
    ///     vec.push(1).map_err(AppError::VectorCapacityError)?;
    ///     vec.push(1).map_err(AppError::VectorCapacityError)?;
    ///
    ///     // third push will fail because vector capacity is 2
    ///     vec.push(3).map_err(AppError::VectorCapacityError)?;
    ///
    ///     // other operations that could return errors
    ///     Ok(())
    /// }
    ///
    /// fn main() -> Result<(), AppError> {
    ///     let mut vec = Vec::<i32, 2>::new();
    ///
    ///     if let Err(err) = my_fn(&mut vec) {
    ///         match err {
    ///             AppError::VectorCapacityError(_) => {
    ///                 // handle case
    ///             },
    ///         }
    ///     }
    ///
    ///     Ok(())
    /// }
    /// ```
    #[inline]
    #[doc(alias("add", "append", "insert"))]
    pub fn push(&mut self, value: T) -> Result<(), CapacityError> {
        if self.is_full() {
            Err(CapacityError)
        } else {
            self.push_unchecked(value);
            Ok(())
        }
    }

    /// Removes all elements. Size will be zero.
    ///
    /// # Example
    ///
    /// ```rust
    /// use static_vector::Vec;
    ///
    /// let mut vec = Vec::<i32, 20>::new();
    ///
    /// // add some elements
    /// vec.clear();
    /// // elements will be gone
    /// ```
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
    /// - `T` must implement [`Default`] because new elements are created with `T::default()`
    ///   when increasing the length.
    ///
    /// # Errors
    ///
    /// Returns [`CapacityError`] if `new_length` exceeds the vector's fixed capacity.
    ///
    /// # Example
    ///
    /// ```rust
    /// use static_vector::Vec;
    ///
    /// #[derive(Debug)]
    /// enum AppError {
    ///     MyFnError,
    /// }
    ///
    /// fn my_fn(vec: &mut Vec<i32, 200>) -> Result<(), AppError> {
    ///     vec.set_len(100).map_err(|_| AppError::MyFnError)?;
    ///
    ///     // other operations that could return errors
    ///
    ///     Ok(())
    /// }
    ///
    /// fn main() -> Result<(), AppError> {
    ///     let mut vec = Vec::<i32, 200>::new();
    ///
    ///     if let Err(err) = my_fn(&mut vec) {
    ///         match err {
    ///             AppError::MyFnError => {
    ///                 // handle case
    ///             },
    ///         }
    ///     }
    ///
    ///     Ok(())
    /// }
    /// ```
    #[doc(alias("resize", "length"))]
    pub fn set_len(&mut self, new_length: usize) -> Result<(), CapacityError>
    where
        T: Default,
    {
        if new_length > CAPACITY {
            return Err(CapacityError);
        }

        if new_length > self.length {
            while self.length < new_length {
                self.push_unchecked(T::default());
            }
        } else {
            self.drop_range(new_length, self.length);
            self.length = new_length;
        }

        Ok(())
    }

    /// Returns a reference to the first element in the vector, or [`None`] if the vector is empty.
    ///
    /// # Example
    ///
    /// ```rust
    /// use static_vector::Vec;
    ///
    /// let mut vec = Vec::<i32, 20>::new();
    ///
    /// match vec.first() {
    ///     Some(num) => {
    ///         let _ = num;
    ///     },
    ///     None => {
    ///         // no first element
    ///     },
    /// }
    /// ```
    #[must_use]
    #[inline]
    #[doc(alias("front", "head", "start"))]
    pub const fn first(&self) -> Option<&T> {
        self.get(0)
    }

    /// Returns a mutable reference to the first element in the vector, or [`None`] if the vector is empty.
    ///
    /// # Example
    ///
    /// ```rust
    /// use static_vector::Vec;
    ///
    /// let mut vec = Vec::<i32, 20>::new();
    ///
    /// if let Some(num) = vec.first_mut() {
    ///     *num = 1;
    ///     let _ = num;
    /// }
    /// ```
    #[must_use]
    #[inline]
    #[doc(alias("front", "head", "start"))]
    pub const fn first_mut(&mut self) -> Option<&mut T> {
        self.get_mut(0)
    }

    /// Returns a reference to the last element in the vector, or [`None`] if the vector is empty.
    ///
    /// # Example
    ///
    /// ```rust
    /// use static_vector::Vec;
    ///
    /// let mut vec = Vec::<i32, 30>::new();
    ///
    /// if let Some(num) = vec.last() {
    ///     let _ = num;
    ///     // do something with the last element
    /// }
    /// ```
    #[must_use]
    #[inline]
    #[doc(alias("back", "tail", "end"))]
    pub const fn last(&self) -> Option<&T> {
        if self.is_empty() { None } else { self.get(self.len() - 1) }
    }

    /// Returns a mutable reference to the last element in the vector, or [`None`] if the vector is empty.
    ///
    /// # Example
    ///
    /// ```rust
    /// use static_vector::Vec;
    ///
    /// let mut vec = Vec::<i32, 20>::new();
    ///
    /// if let Some(num) = vec.last_mut() {
    ///     *num = 1;
    ///     let _ = num;
    /// }
    /// ```
    #[must_use]
    #[inline]
    #[doc(alias("back", "tail", "end"))]
    pub const fn last_mut(&mut self) -> Option<&mut T> {
        if self.is_empty() { None } else { self.get_mut(self.len() - 1) }
    }

    /// Returns a reference to the element at the specified `index`, or [`None`] if out of bounds.
    ///
    /// # Example
    ///
    /// ```rust
    /// use static_vector::Vec;
    ///
    /// let mut vec = Vec::<i32, 20>::new();
    ///
    /// match vec.get(22) {
    ///     Some(num) => {
    ///         let _ = num;
    ///     },
    ///     None => {
    ///         // element with index 22 does not exist
    ///     },
    /// }
    /// ```
    #[must_use]
    #[inline]
    #[doc(alias("at", "index"))]
    pub const fn get(&self, index: usize) -> Option<&T> {
        if index >= self.length {
            None
        } else {
            // SAFETY:
            // - `index` is within bounds of `self.data`.
            // - The element at `index` has been initialized.
            Some(unsafe { &*self.data[index].as_ptr() })
        }
    }

    /// Returns a mutable reference to the element at the specified `index`, or [`None`] if out of bounds.
    ///
    /// # Example
    ///
    /// ```rust
    /// use static_vector::Vec;
    ///
    /// let mut vec = Vec::<i32, 20>::new();
    ///
    /// if vec.push(1).is_ok() {
    ///     *vec.get_mut(0).unwrap() = 5;
    /// }
    /// ```
    #[must_use]
    #[inline]
    #[doc(alias("at", "index"))]
    pub const fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        if index >= self.len() {
            None
        } else {
            // SAFETY:
            // - `index` is within bounds of `self.data`.
            // - The element at `index` has been initialized.
            Some(unsafe { &mut *self.data[index].as_mut_ptr() })
        }
    }

    /// Returns (and removes) the last element from the vector, or [`None`] if the vector is empty.
    ///
    /// # Example
    ///
    /// ```rust
    /// use static_vector::Vec;
    ///
    /// let mut vec = Vec::<i32, 20>::new();
    ///
    /// // fill vector with number from 1 to 10
    /// vec.set_len(10).unwrap(); // can unwrap because 10 < 20
    /// vec.as_mut_slice().fill_with({
    ///     let mut n = 0;
    ///     move || {
    ///         n = n + 1;
    ///         n
    ///     }
    /// });
    ///
    /// // print in reverse order while removing from
    /// while let Some(num) = vec.pop() {
    ///     print!("{num} ");
    /// }
    ///
    /// // prints: 10 9 8 7 6 5 4 3 2 1
    /// // the vector is now empty
    /// ```
    #[must_use]
    #[inline]
    #[doc(alias("remove", "get"))]
    pub const fn pop(&mut self) -> Option<T> {
        if self.is_empty() {
            None
        } else {
            self.length -= 1;

            // SAFETY:
            // - `self.length` is within bounds of `self.data`.
            // - The element at `self.length` has been initialized.
            Some(unsafe { self.data[self.length].assume_init_read() })
        }
    }

    /// Returns (and removes) the last element from the vector if the predicate returns true,
    /// or [`None`] if the vector is empty or the predicate returns false.
    /// # Example
    ///
    /// Similar to [`Vec::pop()`], but needs a predicate
    ///
    /// ```rust
    /// use static_vector::Vec;
    ///
    /// let mut vec = Vec::<i32, 20>::new();
    ///
    /// // assuming vector has elements: 10 9 8 7 6 5 4 3 2 1
    /// if let Some(num) = vec.pop_if(|n| *n % 2 == 0) {
    ///     println!("{num}");
    ///     // prints: 10
    ///     // the vector has remained with: 1 2 3 4 5 6 7 8 9
    /// }
    /// ```
    #[must_use]
    #[inline]
    #[doc(alias("remove", "get"))]
    pub fn pop_if<F: FnOnce(&T) -> bool>(&mut self, predicate: F) -> Option<T> {
        let last = self.last()?;
        if predicate(last) { self.pop() } else { None }
    }

    /// Returns an iterator over immutable references to the elements in the vector.
    ///
    /// # Example
    ///
    /// ```rust
    /// use static_vector::Vec;
    ///
    /// let mut vec = Vec::<i32, 20>::new();
    ///
    /// vec.iter().map(|n| *n * 2).sum::<i32>();
    /// ```
    #[inline]
    pub const fn iter(&self) -> Iter<'_, T> {
        Iter::new(&self.data, self.length)
    }

    /// Returns an iterator over mutable references to the elements in the vector.
    ///
    /// # Example
    ///
    /// ```rust
    /// use static_vector::Vec;
    ///
    /// let mut vec = Vec::<i32, 20>::new();
    ///
    /// for num in vec.iter_mut() {
    ///     *num *= 2;
    /// }
    /// ```
    #[inline]
    pub const fn iter_mut(&mut self) -> IterMut<'_, T> {
        IterMut::new(&mut self.data, self.length)
    }

    /// Returns a slice of the entire vector.
    ///
    /// # Example
    ///
    /// ```rust
    /// use static_vector::Vec;
    ///
    /// fn count_ones(numbers: &[i32]) -> usize {
    ///     numbers.iter().filter(|n| *n == &1).count()
    /// }
    ///
    /// let vec = Vec::<i32, 1000>::new();
    ///
    /// // push numbers into vector
    ///     
    /// if vec.as_slice().binary_search(&1).is_ok() {
    ///     // found it
    /// }
    ///
    /// let ones = count_ones(vec.as_slice());
    /// if ones > 0 {
    ///     // ...
    /// }
    #[must_use]
    #[inline]
    pub const fn as_slice(&self) -> &[T] {
        // SAFETY: A correct length is used to avoid accessing uninitialized elements.
        unsafe { slice::from_raw_parts(self.data[0].as_ptr(), self.len()) }
    }

    /// Returns a mutable slice of the entire vector.
    ///
    /// # Example
    ///
    /// ```rust
    /// use static_vector::Vec;
    ///
    /// let mut vec = Vec::<i32, 10>::new();
    ///
    /// if vec.set_len(5).is_ok() {
    ///     vec.as_mut_slice().fill(1);
    /// } else {
    ///     // handle error
    /// }
    /// ```
    #[must_use]
    #[inline]
    pub const fn as_mut_slice(&mut self) -> &mut [T] {
        // SAFETY: A correct length is used to avoid accessing uninitialized elements.
        unsafe { slice::from_raw_parts_mut(self.data[0].as_mut_ptr(), self.len()) }
    }

    /// Inserts elements of given slice at the end of the vector.
    ///
    /// # Errors
    ///
    /// Returns [`CapacityError`] if adding elements of given slice would result in vector exceeding its capacity.
    ///
    /// # Example
    ///
    /// ```rust
    /// use static_vector::{CapacityError, Vec};
    ///
    /// #[derive(Debug)]
    /// enum AppError {
    ///     VectorCapacityError(CapacityError),
    /// }
    ///
    /// fn my_fn(src: &[i32], vec: &mut Vec<i32, 2>) -> Result<(), AppError> {
    ///     vec.extend_from_slice(src).map_err(AppError::VectorCapacityError)?;
    ///
    ///     // other operations that could return errors
    ///     Ok(())
    /// }
    ///
    /// fn main() -> Result<(), AppError> {
    ///     let src = [1, 2, 3];
    ///     let mut vec = Vec::<i32, 2>::new();
    ///
    ///     if let Err(err) = my_fn(&src, &mut vec) {
    ///         match err {
    ///             AppError::VectorCapacityError(_) => {
    ///                 // handle case
    ///             },
    ///         }
    ///     }
    ///
    ///     Ok(())
    /// }
    /// ```
    #[inline]
    pub fn extend_from_slice(&mut self, slice: &[T]) -> Result<(), CapacityError>
    where
        T: Clone,
    {
        if self.len() + slice.len() > CAPACITY {
            return Err(CapacityError);
        }

        for value in slice {
            self.push_unchecked(value.clone());
        }

        Ok(())
    }

    /// Moves elements of `other` vector at the end of the current vector. `other` will be empty.
    ///
    /// # Errors
    ///
    /// Returns [`CapacityError`] if adding elements from `other` would result in current vector exceeding its capacity.
    ///
    /// # Example
    ///
    /// ```rust
    /// use static_vector::Vec;
    ///
    /// let mut vec = vec![1, 2, 3];
    /// let mut other = vec![4, 5, 6];
    /// vec.append(&mut other);
    /// assert_eq!(vec, [1, 2, 3, 4, 5, 6]);
    /// assert_eq!(other, []);
    /// ```
    #[inline]
    pub fn append<const OTHER_CAPACITY: usize>(
        &mut self,
        other: &mut Vec<T, OTHER_CAPACITY>,
    ) -> Result<(), CapacityError>
    where
        T: Clone,
    {
        self.extend_from_slice(other.as_slice())?;
        other.clear();

        Ok(())
    }

    /// Adds the given `value` to the end of the vector without checking bounds.
    /// For internal and controlled use only.
    fn push_unchecked(&mut self, value: T) {
        debug_assert!(!self.is_full(), "cannot push to full vector");
        self.data[self.length].write(value);
        self.length += 1;
    }

    /// Drops all elements in given range. Needed when elements are considered to be going out of scope.
    /// E.g.: when the vector is going out of scope, when methods such as [`Vec::clear()`] and [`Vec::set_len()`] are called.
    fn drop_range(&mut self, from: usize, to: usize) {
        for i in from..to {
            // SAFETY:
            // - `i` is within bounds of `self.data`.
            // - The element at `i` has been initialized.
            unsafe {
                self.data[i].assume_init_drop();
            }
        }
    }
}

impl<T, const CAPACITY: usize> Drop for Vec<T, CAPACITY> {
    fn drop(&mut self) {
        self.drop_range(0, self.length);
    }
}

impl<T: Clone, const CAPACITY: usize> Clone for Vec<T, CAPACITY> {
    fn clone(&self) -> Self {
        let mut vec = Self::new();
        for value in self {
            vec.push_unchecked(value.clone());
        }
        vec
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
            // - `self.index` is within bounds of `self.data`.
            // - The element at `self.index` has been initialized.
            let value = unsafe { &*self.data[self.index].as_ptr() };
            self.index += 1;
            Some(value)
        }
    }
}

impl<'a, T: 'a, const CAPACITY: usize> IntoIterator for &'a Vec<T, CAPACITY> {
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
            // - `self.index` is within bounds of `self.data`.
            // - The element at `self.index` has been initialized.
            let value = unsafe { &mut *self.data[self.index].as_mut_ptr() };

            self.index += 1;
            Some(value)
        }
    }
}

impl<'a, T: 'a, const CAPACITY: usize> IntoIterator for &'a mut Vec<T, CAPACITY> {
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
    fn new() {
        let mut vec = Vec::<Struct, 10>::new();
        assert!(vec.is_empty());
        assert!(!vec.is_full());
        assert_eq!(vec.len(), 0);
        assert_eq!(vec.capacity(), 10);
        assert_eq!(vec.as_slice(), []);
        assert_eq!(vec.as_mut_slice(), []);
        assert_eq!(DEFAULTS.get(), 0);
    }

    #[test]
    #[should_panic(expected = "CAPACITY must be greater than 0")]
    fn new_with_capacity_zero() {
        let _ = Vec::<i32, 0>::new();
    }

    #[test]
    fn default() {
        let mut vec = Vec::<Struct, 10>::default();
        assert!(vec.is_empty());
        assert!(!vec.is_full());
        assert_eq!(vec.len(), 0);
        assert_eq!(vec.capacity(), 10);
        assert_eq!(vec.as_slice(), []);
        assert_eq!(vec.as_mut_slice(), []);
        assert_eq!(DEFAULTS.get(), 0);
    }

    #[test]
    #[should_panic(expected = "CAPACITY must be greater than 0")]
    fn default_with_capacity_zero() {
        let _ = Vec::<i32, 0>::default();
    }

    #[test]
    fn capacity() {
        let mut vec = Vec::<i32, 3>::new();
        assert_eq!(vec.capacity(), 3);

        vec.set_len(2).unwrap();
        assert_eq!(vec.capacity(), 3);

        vec.push(1).unwrap();
        assert_eq!(vec.capacity(), 3);

        vec.clear();
        assert_eq!(vec.capacity(), 3);
    }

    #[test]
    fn len() {
        let mut vec = Vec::<i32, 3>::new();
        assert_eq!(vec.len(), 0);

        vec.set_len(2).unwrap();
        assert_eq!(vec.len(), 2);

        vec.push(1).unwrap();
        assert_eq!(vec.len(), 3);

        vec.clear();
        assert_eq!(vec.len(), 0);
    }

    #[test]
    fn is_empty() {
        let mut vec = Vec::<i32, 3>::new();
        assert!(vec.is_empty());

        vec.push(1).unwrap();
        assert!(!vec.is_empty());

        vec.set_len(2).unwrap();
        assert!(!vec.is_empty());

        vec.clear();
        assert!(vec.is_empty());
    }

    #[test]
    fn is_full() {
        let mut vec = Vec::<i32, 3>::new();
        assert!(!vec.is_full());

        vec.push(1).unwrap();
        assert!(!vec.is_full());

        vec.set_len(3).unwrap();
        assert!(vec.is_full());

        vec.clear();
        assert!(!vec.is_full());

        vec.push(1).unwrap();
        vec.push(1).unwrap();
        vec.push(1).unwrap();
        assert!(vec.is_full());
    }

    #[test]
    fn push() {
        let mut vec = Vec::<i32, 2>::new();
        assert!(vec.push(1).is_ok());
        assert!(vec.push(2).is_ok());

        assert!(matches!(vec.push(3), Err(CapacityError)));
        assert_eq!(format!("{}", vec.push(3).unwrap_err()), "vector needs larger capacity");
        assert_is_core_error::<CapacityError>();

        assert_eq!(vec.as_slice(), &[1, 2]);
        assert!(vec.get(2).is_none());
        assert!(vec.get(99).is_none());
    }

    #[test]
    fn push_should_not_create_default_elements() {
        let mut vec = Vec::<Struct, 10>::new();

        vec.push(Struct { i: 0 }).unwrap();

        assert_eq!(DEFAULTS.get(), 0);
        assert_eq!(vec.as_slice(), &[Struct { i: 0 }]);
    }

    #[test]
    fn push_should_not_clone_element() {
        let mut vec = Vec::<Struct, 10>::new();

        vec.push(Struct { i: 1 }).unwrap();
        assert_eq!(CLONES.get(), 0);

        vec.push(Struct { i: 2 }).unwrap();
        vec.push(Struct { i: 3 }).unwrap();
        assert_eq!(CLONES.get(), 0);

        assert_eq!(vec.as_slice(), &[Struct { i: 1 }, Struct { i: 2 }, Struct { i: 3 }]);
    }

    #[test]
    fn clear() {
        let mut vec = Vec::<i32, 3>::new();

        vec.extend_from_slice(&[1, 2, 3]).unwrap();
        assert_eq!(vec.len(), 3);
        assert_eq!(vec.capacity(), 3);
        assert!(!vec.is_empty());
        assert!(vec.is_full());
        assert_eq!(vec.as_slice(), &[1, 2, 3]);

        vec.clear();

        assert_eq!(vec.len(), 0);
        assert_eq!(vec.capacity(), 3);
        assert!(vec.is_empty());
        assert!(!vec.is_full());
        assert_eq!(vec.as_slice(), &[]);
    }

    #[test]
    fn clear_should_drop_all_allocated_elements() {
        let mut vec = Vec::<Struct, 10>::new();
        assert_eq!(DROPS.get(), 0);

        let s = Struct { i: 0 };
        for _ in 1..=3 {
            vec.push(s.clone()).unwrap();
        }
        assert_eq!(DROPS.get(), 0);

        vec.clear();
        assert_eq!(DROPS.get(), 3);

        assert_eq!(CLONES.get(), 3); // the three clones before push
        assert_eq!(DEFAULTS.get(), 0);
    }

    #[test]
    fn set_len() {
        let mut vec = Vec::<i32, 3>::new();

        // New length less than capacity
        assert!(vec.set_len(1).is_ok());
        assert_eq!(vec.len(), 1);
        assert!(!vec.is_empty());
        assert!(!vec.is_full());
        assert_eq!(vec.as_slice(), [0]);

        // New length larger than capacity
        assert!(matches!(vec.set_len(100), Err(CapacityError)));
        assert_eq!(format!("{}", vec.set_len(100).unwrap_err()), "vector needs larger capacity");
        assert_is_core_error::<CapacityError>();
        assert_eq!(vec.len(), 1);
        assert!(!vec.is_empty());
        assert!(!vec.is_full());
        assert_eq!(vec.as_slice(), [0]);

        // New length equal to capacity
        vec.clear();
        vec.set_len(vec.capacity()).unwrap();
        assert_eq!(vec.len(), 3);
        assert!(!vec.is_empty());
        assert!(vec.is_full());
        assert_eq!(vec.as_slice(), [0, 0, 0]);

        // New length zero
        assert!(vec.set_len(0).is_ok());
        assert_eq!(vec.len(), 0);
        assert!(vec.is_empty());
        assert!(!vec.is_full());
        assert_eq!(vec.as_slice(), []);
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
    fn set_len_should_drop_all_allocated_elements() {
        let mut vec = Vec::<Struct, 10>::new();
        assert_eq!(DROPS.get(), 0);

        let s = Struct { i: 0 };
        for _ in 1..=5 {
            vec.push(s.clone()).unwrap();
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

        assert_eq!(CLONES.get(), 5); // the five clones before push
        assert_eq!(DEFAULTS.get(), 0);
    }

    #[test]
    fn first() {
        let mut vec = Vec::<i32, 4>::new();
        assert!(vec.first().is_none());

        vec.push(1).unwrap();
        assert_eq!(vec.first().unwrap(), &1);

        vec.push(2).unwrap();
        vec.push(3).unwrap();
        assert_eq!(vec.first(), Some(&1));
    }

    #[test]
    fn first_mut() {
        let mut vec = Vec::<i32, 4>::new();
        assert!(vec.first_mut().is_none());

        vec.push(1).unwrap();
        assert_eq!(vec.first_mut().unwrap(), &1);

        vec.push(2).unwrap();
        vec.push(3).unwrap();
        assert_eq!(vec.first_mut().unwrap(), &1);

        *vec.first_mut().unwrap() = 4;
        assert_eq!(vec.first_mut(), Some(&mut 4));
        assert_eq!(vec.as_slice(), [4, 2, 3]);
    }

    #[test]
    fn last() {
        let mut vec = Vec::<i32, 2>::new();
        assert!(vec.last().is_none());

        vec.push(1).unwrap();
        assert_eq!(vec.last().unwrap(), &1);

        vec.push(2).unwrap();
        assert_eq!(vec.last().unwrap(), &2);

        vec.push(3).unwrap_err();
        assert_eq!(vec.last(), Some(&2));
    }

    #[test]
    fn last_mut() {
        let mut vec = Vec::<i32, 2>::new();
        assert!(vec.last_mut().is_none());

        vec.push(1).unwrap();
        assert_eq!(vec.last_mut().unwrap(), &1);

        vec.push(2).unwrap();
        assert_eq!(vec.last_mut().unwrap(), &2);

        vec.push(3).unwrap_err();
        assert_eq!(vec.last_mut().unwrap(), &2);

        *vec.last_mut().unwrap() = 4;
        assert_eq!(vec.as_slice(), [1, 4]);

        vec.set_len(1).unwrap();
        assert_eq!(vec.last_mut(), Some(&mut 1));
        assert_eq!(vec.as_slice(), [1]);
    }

    #[test]
    fn get() {
        let mut vec = Vec::<i32, 2>::new();
        assert!(vec.get(0).is_none());

        vec.push(1).unwrap();
        assert_eq!(vec.get(0), Some(&1));

        vec.push(2).unwrap();
        assert_eq!(vec.get(1), Some(&2));

        assert_eq!(vec.get(2), None);
        assert_eq!(vec.get(3), None);
    }

    #[test]
    fn get_mut() {
        let mut vec = Vec::<i32, 2>::new();
        assert!(vec.get_mut(0).is_none());

        vec.push(1).unwrap();
        assert_eq!(vec.get_mut(0), Some(&mut 1));

        vec.push(2).unwrap();
        *vec.get_mut(1).unwrap() = 3;
        assert_eq!(vec.get_mut(1), Some(&mut 3));

        assert_eq!(vec.get_mut(2), None);
        assert_eq!(vec.get_mut(3), None);
    }

    #[test]
    fn pop() {
        let mut vec = Vec::<Struct, 4>::new();
        assert!(vec.pop().is_none());

        let s1 = Struct { i: 1 };
        vec.push(s1).unwrap();

        let s2 = Struct { i: 2 };
        vec.push(s2).unwrap();

        let s3 = Struct { i: 3 };
        vec.push(s3).unwrap();

        assert_eq!(vec.pop().unwrap().i, 3);
        assert_eq!(vec.len(), 2);
        assert_eq!(DROPS.get(), 1);

        assert_eq!(vec.pop().unwrap().i, 2);
        assert_eq!(vec.pop().unwrap().i, 1);
        assert!(vec.is_empty());
        assert!(vec.pop().is_none());
        assert_eq!(DROPS.get(), 3);

        assert_eq!(DEFAULTS.get(), 0);
        assert_eq!(CLONES.get(), 0); // from the three pushes
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
        vec.push(s1).unwrap();

        let s2 = Struct { i: 2 };
        vec.push(s2).unwrap();

        let s3 = Struct { i: 3 };
        vec.push(s3).unwrap();

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
        assert_eq!(CLONES.get(), 0); // from the three pushes
    }

    #[test]
    fn iter() {
        let mut vec = Vec::<i32, 10>::new();
        for i in 1..=7 {
            vec.push(i).unwrap();
        }

        let even_sum = vec.iter().filter(|v| *v % 2 == 0).sum::<i32>();
        assert_eq!(even_sum, 12);

        assert_eq!(vec.iter().count(), 7);
    }

    #[test]
    fn into_iter() {
        let mut vec = Vec::<i32, 10>::new();
        for i in 1..=7 {
            vec.push(i).unwrap();
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
        for i in 1..=7 {
            vec.push(i).unwrap();
        }

        let even_sum = vec.iter_mut().filter(|v| **v % 2 == 0).map(|v| *v).sum::<i32>();
        assert_eq!(even_sum, 12);

        assert_eq!(vec.iter().count(), 7);
    }

    #[test]
    fn into_iter_mut() {
        let mut vec = Vec::<i32, 10>::new();
        for i in 1..=7 {
            vec.push(i).unwrap();
        }

        let mut s = 0;
        for i in &mut vec {
            *i *= 2;
            s += *i;
        }
        assert_eq!(s, 56);
    }

    #[test]
    fn as_slice() {
        let mut vec = Vec::<i32, 1000>::new();
        assert_eq!(vec.as_slice(), []);

        vec.extend_from_slice(&[1, 2, 3]).unwrap();
        assert_eq!(vec.as_slice(), [1, 2, 3]);
    }

    #[test]
    fn as_mut_slice() {
        let mut vec = Vec::<i32, 1000>::new();
        assert_eq!(vec.as_mut_slice(), []);

        vec.extend_from_slice(&[1, 2, 3]).unwrap();
        assert_eq!(vec.as_mut_slice(), [1, 2, 3]);

        vec.set_len(1000).unwrap();
        vec.as_mut_slice().fill(2);
        assert_eq!(vec.as_slice().iter().sum::<i32>(), 2000);
    }

    #[test]
    fn extend_from_slice_with_empty_vector_and_empty_slice() {
        let src = [];
        let mut dst = Vec::<i32, 3>::new();
        let result = dst.extend_from_slice(&src);

        assert!(result.is_ok());
        assert!(dst.is_empty());
    }

    #[test]
    fn extend_from_slice_with_empty_vector_and_non_empty_slice_within_capacity() {
        let src = [1, 2];
        let mut dst = Vec::<i32, 3>::new();
        let result = dst.extend_from_slice(&src);

        assert!(result.is_ok());
        assert_eq!(dst.len(), 2);
        assert_eq!(dst.as_slice(), [1, 2]);
    }

    #[test]
    fn extend_from_slice_with_non_empty_vector_and_empty_slice() {
        let src = [];
        let mut dst = Vec::<i32, 3>::new();
        dst.push(1).unwrap();
        dst.push(2).unwrap();
        let result = dst.extend_from_slice(&src);

        assert!(result.is_ok());
        assert_eq!(dst.len(), 2);
        assert_eq!(dst.as_slice(), [1, 2]);
    }

    #[test]
    fn extend_from_slice_with_non_empty_vector_and_slice_fits_exactly_into_capacity() {
        let src = [3, 4, 5];
        let mut dst = Vec::<i32, 5>::new();
        dst.push(1).unwrap();
        dst.push(2).unwrap();
        let result = dst.extend_from_slice(&src);

        assert!(result.is_ok());
        assert_eq!(dst.len(), 5);
        assert!(dst.is_full());
        assert_eq!(dst.as_slice(), [1, 2, 3, 4, 5]);
    }

    #[test]
    fn extend_from_slice_with_non_empty_vector_and_slice_exceeds_capacity() {
        let src = [3, 4, 5, 6];
        let mut dst = Vec::<i32, 5>::new();
        dst.push(1).unwrap();
        dst.push(2).unwrap();
        let result = dst.extend_from_slice(&src);

        assert!(result.is_err());
        assert_eq!(dst.len(), 2);
        assert_eq!(dst.as_slice(), [1, 2]);
    }

    #[test]
    fn extend_from_slice_with_vector_full_and_non_empty_slice() {
        let src = [3, 4, 5, 6];
        let mut dst = Vec::<i32, 2>::new();
        dst.push(1).unwrap();
        dst.push(2).unwrap();
        let result = dst.extend_from_slice(&src);

        assert!(result.is_err());
        assert_eq!(dst.len(), 2);
        assert!(dst.is_full());
        assert_eq!(dst.as_slice(), [1, 2]);
    }

    #[test]
    fn extend_from_slice_with_non_empty_vector_and_non_empty_slice() {
        let src = [3];
        let mut dst = Vec::<i32, 5>::new();
        dst.push(1).unwrap();
        dst.push(2).unwrap();
        let result = dst.extend_from_slice(&src);

        assert!(result.is_ok());
        assert_eq!(dst.len(), 3);
        assert!(!dst.is_full());
        assert_eq!(dst.as_slice(), [1, 2, 3]);
    }

    #[test]
    fn append_with_enough_room() {
        let mut vec = Vec::<i32, 5>::new();
        vec.push(1).unwrap();
        vec.push(2).unwrap();

        let mut other = Vec::<i32, 20>::new();
        other.push(3).unwrap();
        other.push(4).unwrap();

        let result = vec.append(&mut other);

        assert!(result.is_ok());
        assert_eq!(vec.len(), 4);
        assert_eq!(vec.as_slice(), [1, 2, 3, 4]);
        assert!(other.is_empty());
        assert_eq!(other.as_slice(), []);
    }

    #[test]
    fn append_with_not_enough_room() {
        let mut vec = Vec::<i32, 2>::new();
        vec.push(1).unwrap();
        vec.push(2).unwrap();

        let mut other = Vec::<i32, 20>::new();
        other.push(3).unwrap();
        other.push(4).unwrap();

        let result = vec.append(&mut other);

        assert!(result.is_err());
        assert_eq!(vec.len(), 2);
        assert_eq!(vec.as_slice(), [1, 2]);
        assert_eq!(other.len(), 2);
        assert_eq!(other.as_slice(), [3, 4]);
    }

    #[test]
    fn clone() {
        let mut vec = Vec::<i32, 5>::new();
        let elements = [1, 2, 3];
        vec.extend_from_slice(&elements).unwrap();

        let new = vec.clone();
        assert_eq!(new.len(), 3);
        assert_eq!(new.as_slice(), elements);
    }

    #[test]
    fn going_out_of_scope_should_drop_all_allocated_elements() {
        let s = Struct { i: 0 };

        {
            let mut vec = Vec::<Struct, 10>::new();
            assert_eq!(DROPS.get(), 0);

            for _ in 1..=3 {
                vec.push(s.clone()).unwrap();
            }
            assert_eq!(DROPS.get(), 0);
        };

        assert_eq!(DROPS.get(), 3);
        assert_eq!(CLONES.get(), 3); // the three clones before push
    }

    #[derive(Debug)]
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

    impl PartialEq for Struct {
        fn eq(&self, other: &Self) -> bool {
            self.i == other.i
        }
    }
}
