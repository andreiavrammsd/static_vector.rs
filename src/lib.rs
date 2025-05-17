//! A no-std, stack-allocated vector with fixed capacity and dynamic length.
//!
//! `StaticVector<T, CAPACITY>` stores elements on the stack using a fixed-size array
//! without heap allocations. Aims to be suitable for low-level projects and to have an API as safe and explicit as possible.
//! It's a learning project, so there are no guarantees.
//!
//! # Features
//! - No heap allocation (`#![no_std]` compatible)
//! - Constant-time indexed access
//! - Supports iteration, mutable access, clearing, and appending
//! - Compile-time enforced capacity
//!
//! # Requirements
//! - `T: Clone` for insertion (e.g., `push`, `append`)
//! - `T: Default` only if `set_len` is used
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

use core::{array, borrow::Borrow, mem::MaybeUninit};

/// Error type returned by `StaticVector`.
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
    /// Creates an empty `StaticVector`. Equivalent to `StaticVector::new()`.
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Clone, const CAPACITY: usize> StaticVector<T, CAPACITY> {
    const ASSERT_CAPACITY: () = assert!(CAPACITY > 0);

    /// Creates a new empty `StaticVector` with maximum `CAPACITY` elements of type `T`.
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

    /// Adds a clone of the given value to the end of the vector.
    ///
    /// # Errors
    ///
    /// Returns an `Error("capacity")` if the vector is already at full capacity.
    pub fn push(&mut self, value: &T) -> Result<(), Error> {
        if self.length == CAPACITY {
            return Err(Error("capacity"));
        }

        self.data[self.length].write(value.clone());
        self.length += 1;

        Ok(())
    }

    pub fn append<I>(&mut self, iter: I) -> Result<(), Error>
    where
        I: IntoIterator,
        I::Item: Borrow<T>,
    {
        iter.into_iter().try_for_each(|value| self.push(value.borrow()))
    }

    pub fn clear(&mut self) {
        self.drop(0, self.length);
        self.length = 0
    }

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

    #[must_use]
    #[inline]
    pub fn first(&self) -> Option<&T> {
        if self.length == 0 { None } else { Some(unsafe { &*self.data[0].as_ptr() }) }
    }

    #[must_use]
    #[inline]
    pub fn last(&self) -> Option<&T> {
        if self.length == 0 { None } else { Some(unsafe { &*self.data[self.length - 1].as_ptr() }) }
    }

    #[must_use]
    pub fn get(&self, index: usize) -> Option<&T> {
        if index >= self.length { None } else { Some(unsafe { &*self.data[index].as_ptr() }) }
    }

    #[must_use]
    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        if index >= self.length {
            None
        } else {
            Some(unsafe { &mut *self.data[index].as_mut_ptr() })
        }
    }

    #[inline(always)]
    pub fn iter(&self) -> StaticVectorIterator<T> {
        StaticVectorIterator { data: &self.data, size: self.length, index: 0 }
    }

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

    extern crate std;
    use std::string::{String, ToString};
    use std::vec;
    use std::vec::Vec;

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
    fn sizing() {
        let mut vec = StaticVector::<i32, 3>::new();
        assert_eq!(vec.len(), 0);
        assert!(vec.is_empty());

        vec.push(&1).unwrap();
        vec.push(&2).unwrap();
        assert_eq!(vec.len(), 2);
        assert!(!vec.is_empty());

        vec.set_len(1).unwrap();
        assert_eq!(vec.len(), 1);
        assert!(!vec.is_empty());

        vec.clear();
        assert_eq!(vec.len(), 0);
        assert!(vec.is_empty());
    }

    #[test]
    fn vector_set_len() {
        let mut vec = StaticVector::<i32, 3>::new();
        assert!(vec.set_len(100).is_err());
    }

    #[test]
    fn test() {
        let mut vec = StaticVector::<i32, 3>::new();
        assert_eq!(vec.capacity(), 3);
        assert_eq!(vec.len(), 0);
        assert!(vec.first().is_none());
        assert!(vec.last().is_none());
        assert!(vec.is_empty());

        assert!(vec.push(&1).is_ok());
        assert_eq!(vec.first().unwrap(), &1);
        assert_eq!(vec.last().unwrap(), vec.first().unwrap());
        assert_eq!(vec.get(0).unwrap(), &1);

        assert!(vec.push(&2).is_ok());
        assert_eq!(vec.first().unwrap(), &1);
        assert_eq!(vec.get(1).unwrap(), &2);
        assert_eq!(vec.last().unwrap(), &2);
        vec.set_len(1).unwrap();

        assert_eq!(vec.len(), 1);
        assert!(!vec.is_empty());
        assert!(vec.push(&1).is_ok());
        assert!(vec.push(&1).is_ok());
        assert!(vec.push(&1).is_err());

        assert_eq!(vec.iter().sum::<i32>(), 3);

        vec.clear();
        assert!(vec.is_empty());

        let other = vec![4, 5, 6, 7];
        assert!(vec.append(&other).is_err());
        assert_eq!(vec.iter().sum::<i32>(), 15);
        assert_eq!(other.iter().sum::<i32>(), 22);
        assert_eq!(other.iter().sum::<i32>(), 22);

        vec.clear();
        vec.push(&1).unwrap();
        assert!(vec.set_len(2).is_ok());
        assert_eq!(vec.get(0).unwrap(), &1);
        assert_eq!(vec.get(1).unwrap(), &0);

        vec.clear();
        assert!(vec.set_len(2).is_ok());
        assert_eq!(vec.get(0).unwrap(), &0);
        assert_eq!(vec.get(1).unwrap(), &0);

        {
            #[derive(Clone, Default)]
            struct Page {
                data: Vec<String>,
            }
            let mut pages = StaticVector::<Page, 4>::new();

            pages.push(&Page { data: vec!["a".to_string()] }).unwrap();
            pages.push(&Page { data: vec!["bc".to_string()] }).unwrap();

            assert_eq!(pages.iter().map(|value| value.data.len()).sum::<usize>(), 2);

            if let Some(page) = pages.get_mut(0) {
                page.data.push("d".into());
            };

            assert_eq!(pages.iter().map(|value| value.data.len()).sum::<usize>(), 3);

            pages.iter_mut().for_each(|page| page.data.clear());
            assert_eq!(pages.iter_mut().map(|value| value.data.len()).sum::<usize>(), 0);

            pages.clear();
            assert!(pages.is_empty());
        }

        {
            #[derive(Clone)]
            struct Page {
                data: Vec<String>,
            }
            let mut pages = StaticVector::<Page, 4>::new();

            pages.push(&Page { data: vec!["a".to_string()] }).unwrap();
            pages.push(&Page { data: vec!["bc".to_string()] }).unwrap();

            let _ = pages.first().unwrap().data;
            pages.clear();
        }

        {
            let mut vec = StaticVector::<String, 10>::new();
            vec.push(&"value".into()).unwrap();
        }
    }
}
