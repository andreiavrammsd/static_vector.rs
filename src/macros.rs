/// Creates new vector.
///
/// # Forms
///
/// - `vec![T; CAPACITY]`: Creates a new empty `Vec<T, CAPACITY>` with maximum `CAPACITY` elements of type `T`.
/// - `vec![x, y, z]`: Creates a `Vec` initialized with the given values. The capacity is inferred from the number of elements.
///
/// # Examples
///
/// Create an empty vector with capacity:
///
/// ```rust
/// let vec = static_vector::vec![i32; 10];
/// assert!(vec.is_empty());
/// ```
///
/// Create a vector from elements:
///
/// ```rust
/// let vec = static_vector::vec![1, 2, 3];
/// assert_eq!(vec.as_slice(), &[1, 2, 3]);
/// ```
#[macro_export]
macro_rules! vec {
    ($type:ty; $capacity:literal) => {
        $crate::Vec::<$type, $capacity>::new()
    };

    ($($value:expr),+ $(,)?) => {
        {
            let mut vec = $crate::Vec::<_, { [$($value),+].len() }>::new();
            // It's safe to call unwrap because we are initializing the vector with a known number of elements
            // (which is also the capacity).
            vec.extend_from_slice(&[$($value),+]).unwrap();
            vec
        }
    };
}

#[cfg(test)]
mod tests {
    #[test]
    fn vec_with_type_and_capacity() {
        let vec = vec![i32; 10];
        assert_eq!(vec.capacity(), 10);
        assert!(vec.is_empty());
    }

    #[test]
    fn vec_with_elements() {
        let vec = vec![1, 2, 3];
        assert_eq!(vec.capacity(), 3);
        assert_eq!(vec.len(), 3);
        assert_eq!(vec.as_slice(), &[1, 2, 3]);
    }
}
