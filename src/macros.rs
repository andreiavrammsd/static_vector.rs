#[macro_export]
/// A macro for creating a static vector (`Vec`) with various initialization patterns.
///
/// # Usage
///
/// - `vec![Type; CAPACITY]`
///   - Creates an empty static vector of the specified type and capacity.
///   - Example: `vec![u32; 8]`
///
/// - `vec![value1, value2, ..., valueN]`
///   - Creates a static vector with the given values, inferring the type and capacity from the values.
///   - Example: `vec![1, 2, 3]`
///
/// - `vec![CAPACITY; value1, value2, ..., valueN]`
///   - Creates a static vector with the specified capacity and initializes it with the given values.
///   - Example: `vec![8; 1, 2, 3]`
///
/// - `vec![Type; CAPACITY; Length]`
///   - Creates a static vector of the specified type and capacity, and sets its length to `Length`.
///   - Example: `vec![u32; 8; 4]`
///
/// # Panics
///
/// Panics if the specified capacity is zero, or the number of provided values exceeds the capacity, or the requested length is greater than the capacity.
///
/// # Examples
///
/// ```rust
/// use static_vector::vec;
/// let vec = vec![u8; 4]; // Empty vector with capacity 4
/// let vec = vec![1, 2, 3]; // Vector with 3 elements
/// let vec = vec![4; 1, 2]; // Vector with capacity 4, initialized with 2 elements
/// let vec = vec![u16; 8; 5]; // Vector with capacity 8, length set to 5, initialized with zeros
/// ```
macro_rules! vec {
    ($type:ty; $capacity:expr) => {
        $crate::Vec::<$type, $capacity>::new()
    };

    ($($value:expr),+ $(,)?) => {
        {
            let mut vec = $crate::Vec::<_, { [$($value),+].len() }>::new();
            vec.extend_from_slice(&[$($value),+]).expect("length matches capacity");
            vec
        }
    };

    ($capacity:expr; $($value:expr),+ $(,)?) => {
        {
            let mut vec = $crate::Vec::<_, $capacity>::new();
            vec.extend_from_slice(&[$($value),+]).expect("length is less than or equal to capacity");
            vec
        }
    };

    ($type:ty; $capacity:expr; $length:expr) => {
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
    fn vec_with_type_and_capacity_literal() {
        let vec = vec![i32; 10];
        assert_eq!(vec.capacity(), 10);
        assert!(vec.is_empty());
    }

    #[test]
    fn vec_with_type_and_capacity_constant() {
        const CAPACITY: usize = 10;
        let vec = vec![i32; CAPACITY];
        assert_eq!(vec.capacity(), 10);
        assert!(vec.is_empty());
    }

    #[test]
    #[should_panic(expected = "CAPACITY must be greater than 0")]
    fn vec_with_type_and_capacity_zero() {
        let _ = vec![i32; 0];
    }

    #[test]
    fn vec_with_one_element() {
        let vec = vec![999];
        assert_eq!(vec.capacity(), 1);
        assert_eq!(vec.len(), 1);
        assert_eq!(vec.as_slice(), &[999]);
    }

    #[test]
    fn vec_with_elements() {
        let vec = vec![1, 2, 3];
        assert_eq!(vec.capacity(), 3);
        assert_eq!(vec.len(), 3);
        assert_eq!(vec.as_slice(), &[1, 2, 3]);
    }

    #[test]
    fn vec_with_capacity_literal_and_elements() {
        let vec = vec![10; 1, 2, 3];
        assert_eq!(vec.capacity(), 10);
        assert_eq!(vec.len(), 3);
        assert_eq!(vec.as_slice(), &[1, 2, 3]);
    }

    #[test]
    fn vec_with_capacity_constants_and_elements() {
        const CAPACITY: usize = 10;
        let vec = vec![CAPACITY; 1, 2, 3];
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
    fn vec_with_capacity_and_length_literals() {
        let vec = vec![i32; 10; 3];
        assert_eq!(vec.capacity(), 10);
        assert_eq!(vec.len(), 3);
        assert_eq!(vec.as_slice(), &[0, 0, 0]);
    }

    #[test]
    fn vec_with_capacity_and_length_constants() {
        const CAPACITY: usize = 10;
        const LENGTH: usize = 3;
        let vec = vec![i32; CAPACITY; LENGTH];
        assert_eq!(vec.capacity(), 10);
        assert_eq!(vec.len(), 3);
        assert_eq!(vec.as_slice(), &[0, 0, 0]);
    }

    #[test]
    fn vec_with_capacity_and_length_zero() {
        let vec = vec![i32; 10; 0];
        assert_eq!(vec.capacity(), 10);
        assert!(vec.is_empty());
        assert_eq!(vec.as_slice(), &[]);
    }

    #[test]
    fn vec_with_capacity_and_length_equal_to_capacity() {
        let vec = vec![i32; 3; 3];
        assert_eq!(vec.capacity(), 3);
        assert_eq!(vec.len(), 3);
        assert_eq!(vec.as_slice(), &[0, 0, 0]);
    }

    #[test]
    #[should_panic(expected = "length is less than or equal to capacity: CapacityError")]
    fn vec_with_capacity_and_length_greater_than_capacity() {
        let _ = vec![i32; 10; 30];
    }
}
