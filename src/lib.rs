//! A no-std, stack-allocated vector with fixed capacity and dynamic length.
//!
//! [`StaticVector`] stores elements on the stack using a fixed-size array without heap allocations.
//!
//! Aims to be suitable for low-level projects and to have an API as safe and explicit as possible.
//! The goal is to allocate only when needed. When first constructed, the vector will not allocate.
//!
//! It's a learning project, so there are no guarantees.
//!
//! # Features
//! - No heap allocation (`#![no_std]` compatible)
//! - Constant-time indexed access
//! - Supports iteration, mutable access, clearing, resizing
//! - Compile-time enforced capacity
//!
//! # Requirements
//! - `T: Clone` for insertion: [`StaticVector::push()`]
//! - `T: Default` only if [`StaticVector::set_len()`] is used
//! - `CAPACITY > 0`
//!
//! # Example
//! ```rust
//! use static_vector::StaticVector;
//!
//! let mut vec = StaticVector::<i32, 4>::new();
//! vec.push(&1).unwrap();
//! vec.push(&2).unwrap();
//! assert_eq!(vec.len(), 2);
//! ```

#![no_std]
#![deny(missing_docs)]

use core::{array, mem::MaybeUninit};

/// Error type returned by [`StaticVector`].
#[derive(Debug)]
pub struct Error(pub &'static str);

/// A stack-allocated vector with fixed capacity and dynamic length.
///
/// See crate-level documentation for details and usage.
pub struct StaticVector<T: Clone, const CAPACITY: usize> {
    data: [MaybeUninit<T>; CAPACITY],
    length: usize,
}

impl<T: Clone, const CAPACITY: usize> Default for StaticVector<T, CAPACITY> {
    /// Creates an empty [`StaticVector`]. Equivalent to [`StaticVector::new()`].
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Clone, const CAPACITY: usize> StaticVector<T, CAPACITY> {
    const ASSERT_CAPACITY: () = assert!(CAPACITY > 0);

    /// Creates a new empty [`StaticVector`] with maximum `CAPACITY` elements of type `T`.
    #[inline]
    pub fn new() -> Self {
        let () = Self::ASSERT_CAPACITY;
        let data: [MaybeUninit<T>; CAPACITY] = array::from_fn(|_| MaybeUninit::uninit());
        Self { data, length: 0 }
    }

    /// Returns the maximum number of elements the vector can contain.
    #[inline(always)]
    pub fn capacity(&self) -> usize {
        CAPACITY
    }

    /// Returns the maximum number of elements the vector currenly contains.
    #[inline(always)]
    pub fn len(&self) -> usize {
        self.length
    }

    /// Returns whether the vector is empty or not.
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.length == 0
    }

    /// Adds a clone of the given `value` to the end of the vector.
    ///
    /// # Errors
    ///
    /// Returns [`Error`] if the vector is already at full capacity.
    pub fn push(&mut self, value: &T) -> Result<(), Error> {
        if self.length == CAPACITY {
            return Err(Error("capacity"));
        }

        self.data[self.length].write(value.clone());
        self.length += 1;

        Ok(())
    }

    /// Removes all elements. Size will be zero.
    pub fn clear(&mut self) {
        self.drop(0, self.length);
        self.length = 0
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
    /// Returns [`Error`] if `new_length` exceeds the vector's fixed capacity.
    pub fn set_len(&mut self, new_length: usize) -> Result<(), Error>
    where
        T: Default,
    {
        if new_length > CAPACITY {
            return Err(Error("new length > capacity"));
        }

        if new_length > self.length {
            for i in self.length..new_length {
                self.data[i].write(T::default());
            }
        } else {
            self.drop(new_length, self.length);
        }

        self.length = new_length;
        Ok(())
    }

    /// Returns a reference to the first element in the vector, or `None` if the vector is empty.
    #[must_use]
    #[inline]
    pub fn first(&self) -> Option<&T> {
        if self.length == 0 { None } else { Some(unsafe { &*self.data[0].as_ptr() }) }
    }

    /// Returns a reference to the last element in the vector, or `None` if the vector is empty.
    #[must_use]
    #[inline]
    pub fn last(&self) -> Option<&T> {
        if self.length == 0 { None } else { Some(unsafe { &*self.data[self.length - 1].as_ptr() }) }
    }

    /// Returns a reference to the element at the specified `index`, or `None` if out of bounds.
    #[must_use]
    pub fn get(&self, index: usize) -> Option<&T> {
        if index >= self.length { None } else { Some(unsafe { &*self.data[index].as_ptr() }) }
    }

    /// Returns a mutable reference to the element at the specified `index`, or `None` if out of bounds.
    #[must_use]
    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        if index >= self.length {
            None
        } else {
            Some(unsafe { &mut *self.data[index].as_mut_ptr() })
        }
    }

    /// Returns an iterator over immutable references to the elements in the vector.
    #[inline(always)]
    pub fn iter(&self) -> StaticVectorIterator<T> {
        StaticVectorIterator { data: &self.data, size: self.length, index: 0 }
    }

    /// Returns an iterator over mutable references to the elements in the vector.
    #[inline(always)]
    pub fn iter_mut(&mut self) -> StaticVectorMutableIterator<T> {
        StaticVectorMutableIterator { data: &mut self.data, size: self.length, index: 0 }
    }

    fn drop(&mut self, from: usize, to: usize) {
        for i in from..to {
            unsafe {
                self.data[i].as_mut_ptr().drop_in_place();
            }
        }
    }
}

impl<T: Clone, const CAPACITY: usize> Drop for StaticVector<T, CAPACITY> {
    fn drop(&mut self) {
        self.drop(0, self.length);
    }
}

/// Immutable iterator over a [`StaticVector`].
///
/// Created by calling [`StaticVector::iter()`].
#[must_use = "must consume iterator"]
pub struct StaticVectorIterator<'a, T> {
    data: &'a [MaybeUninit<T>],
    size: usize,
    index: usize,
}

impl<'a, T> Iterator for StaticVectorIterator<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index == self.size {
            None
        } else {
            let value = unsafe { &*self.data[self.index].as_ptr() };
            self.index += 1;
            Some(value)
        }
    }
}

/// Mutable iterator over a [`StaticVector`].
///
/// Created by calling [`StaticVector::iter_mut()`].
#[must_use = "must consume iterator"]
pub struct StaticVectorMutableIterator<'a, T> {
    data: &'a mut [MaybeUninit<T>],
    size: usize,
    index: usize,
}

impl<'a, T> Iterator for StaticVectorMutableIterator<'a, T> {
    type Item = &'a mut T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index == self.size {
            None
        } else {
            let value = unsafe { &mut *self.data[self.index].as_mut_ptr() };
            self.index += 1;
            Some(value)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn construct() {
        assert!(StaticVector::<i32, 3>::new().is_empty());
        assert!(StaticVector::<i32, 3>::default().is_empty());

        // Will not build because CAPACITY must be greater than zero
        // StaticVector::<i32, 0>::new().is_empty();
    }

    #[test]
    fn capacity() {
        let mut vec = StaticVector::<i32, 3>::new();

        assert_eq!(vec.capacity(), 3);

        vec.set_len(2).unwrap();
        assert_eq!(vec.capacity(), 3);

        vec.push(&1).unwrap();
        assert_eq!(vec.capacity(), 3);
    }

    #[test]
    fn push() {
        let mut vec = StaticVector::<i32, 2>::new();
        assert!(vec.push(&1).is_ok());
        assert!(vec.push(&2).is_ok());
        assert!(vec.push(&3).is_err());

        assert_eq!(vec.get(0).unwrap(), &1);
        assert_eq!(vec.get(1).unwrap(), &2);
        assert!(vec.get(2).is_none());
    }

    #[test]
    fn size() {
        let mut vec = StaticVector::<i32, 3>::new();
        assert_eq!(vec.len(), 0);
        assert!(vec.is_empty());

        vec.push(&1).unwrap();
        vec.push(&2).unwrap();
        assert_eq!(vec.len(), 2);
        assert!(!vec.is_empty());

        assert!(vec.set_len(1).is_ok());
        assert_eq!(vec.len(), 1);
        assert!(!vec.is_empty());

        assert!(vec.set_len(100).is_err());

        vec.clear();
        assert_eq!(vec.len(), 0);
        assert!(vec.is_empty());
    }

    #[test]
    fn get() {
        let mut vec = StaticVector::<i32, 4>::new();
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
    fn iter() {
        let mut vec = StaticVector::<i32, 10>::new();
        for i in 1..8 {
            vec.push(&i).unwrap()
        }

        let even_sum = vec.iter().filter(|v| *v % 2 == 0).sum::<i32>();
        assert_eq!(even_sum, 12);
    }

    #[test]
    fn iter_mut() {
        let mut vec = StaticVector::<i32, 10>::new();
        for i in 1..8 {
            vec.push(&i).unwrap()
        }

        let even_sum = vec.iter_mut().filter(|v| **v % 2 == 0).map(|v| *v).sum::<i32>();
        assert_eq!(even_sum, 12);
    }
}
