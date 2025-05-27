/// Creates new vector.
///
/// # Forms
///
/// - `vec![T; CAPACITY]`: Creates an empty `Vec<T, CAPACITY>` with maximum `CAPACITY` elements of type `T`.
/// - `vec![x, y, z]`: Creates a `Vec` initialized with the given values. The capacity is inferred from the number of elements.
/// - `vec![CAPACITY; x, y, z]`: Creates a `Vec` with given `CAPACITY`, initialized with the given values.
/// - `vec![T; CAPACITY; LENGTH]`: Creates a `Vec<T, CAPACITY>` with maximum `CAPACITY` elements of type `T` and initialized with default `LENGTH` elements.
///
/// # Panics
///
/// - If `CAPACITY == 0`. Zero-capacity vectors are not supported.
/// - If the number of elements used to initialize the vector is larger than the given `CAPACITY`.
/// - If the given initial `LENGTH` is larger than the given `CAPACITY`.
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
///
/// Create a vector with a specific capacity and elements:
/// ```rust
/// let vec = static_vector::vec![10; 1, 2, 3];
/// assert_eq!(vec.as_slice(), &[1, 2, 3]);
/// ```
///
/// Create a vector with a specific type, capacity, length, and default values:
/// ```rust
/// let vec = static_vector::vec![i32; 10; 3];
/// assert_eq!(vec.as_slice(), &[0, 0, 0]);
/// ```
#[macro_export]
macro_rules! vec {
    ($type:ty; $capacity:literal) => {
        $crate::Vec::<$type, $capacity>::new()
    };

    ($($value:expr),+ $(,)?) => {
        {
            let mut vec = $crate::Vec::<_, { [$($value),+].len() }>::new();
            vec.extend_from_slice(&[$($value),+]).expect("length matches capacity");
            vec
        }
    };

    ($capacity:literal; $($value:expr),+ $(,)?) => {
        {
            let mut vec = $crate::Vec::<_, $capacity>::new();
            vec.extend_from_slice(&[$($value),+]).expect("length is less than or equal to capacity");
            vec
        }
    };

    ($type:ty; $capacity:literal; $length:literal) => {
        {
            let mut vec = $crate::Vec::<$type, $capacity>::new();
            vec.set_len($length).expect("length is less than or equal to capacity");
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
    #[should_panic(expected = "CAPACITY must be greater than 0")]
    fn vec_with_type_and_capacity_zero() {
        let _ = vec![i32; 0];
    }

    #[test]
    fn vec_with_elements() {
        let vec = vec![1, 2, 3];
        assert_eq!(vec.capacity(), 3);
        assert_eq!(vec.len(), 3);
        assert_eq!(vec.as_slice(), &[1, 2, 3]);
    }

    #[test]
    fn vec_with_capacity_and_elements() {
        let vec = vec![10; 1, 2, 3];
        assert_eq!(vec.capacity(), 10);
        assert_eq!(vec.len(), 3);
        assert_eq!(vec.as_slice(), &[1, 2, 3]);
    }

    #[test]
    #[should_panic(expected = "length is less than or equal to capacity: CapacityError")]
    fn vec_with_more_elements_than_capacity() {
        let _ = vec![2; 1, 2, 3];
    }

    #[test]
    fn vec_with_capacity_and_length() {
        let vec = vec![i32; 10; 3];
        assert_eq!(vec.capacity(), 10);
        assert_eq!(vec.len(), 3);
        assert_eq!(vec.as_slice(), &[0, 0, 0]);
    }

    #[test]
    fn vec_with_capacity_and_length_when_length_is_equal_to_capacity() {
        let vec = vec![i32; 3; 3];
        assert_eq!(vec.capacity(), 3);
        assert_eq!(vec.len(), 3);
        assert_eq!(vec.as_slice(), &[0, 0, 0]);
    }

    #[test]
    #[should_panic(expected = "length is less than or equal to capacity: CapacityError")]
    fn vec_with_capacity_and_length_when_length_is_greater_than_capacity() {
        let _ = vec![i32; 10; 30];
    }
}
