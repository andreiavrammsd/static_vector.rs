#![no_main]
#![deny(warnings)]

use libfuzzer_sys::fuzz_target;
use static_vector::Vec;

fuzz_target!(|data: &[u8]| {
    let mut vec: Vec<u8, 125> = Vec::new();

    assert_eq!(vec.capacity(), 125);
    assert_eq!(vec.len(), 0);
    assert!(vec.is_empty());
    assert!(!vec.is_full());
    assert!(vec.first().is_none());
    assert!(vec.last().is_none());
    assert!(vec.pop().is_none());
    assert!(vec.pop_if(|_| true).is_none());

    let mut first_byte = None;
    let mut prev_byte = None;

    for (i, &byte) in data.iter().enumerate() {
        let is_full_before_push = vec.is_full();
        let result = vec.push(&byte);
        let is_full_after_push = vec.is_full();

        if first_byte.is_none() {
            first_byte = Some(byte);
        }
        assert_eq!(vec.first().unwrap(), &first_byte.unwrap());

        if is_full_before_push {
            assert_eq!(vec.len(), vec.capacity());
            assert!(result.is_err());
            assert!(!vec.is_empty());
            assert!(vec.is_full());
            assert_eq!(vec.last().unwrap(), &prev_byte.unwrap());
            assert!(vec.get(i).is_none());
            assert!(vec.get_mut(i).is_none());
        } else {
            assert_eq!(vec.len(), i + 1);
            assert!(result.is_ok());
            assert!(!vec.is_empty());

            if is_full_after_push {
                assert!(vec.is_full());
            } else {
                assert!(!vec.is_full());
            }

            assert_eq!(vec.last().unwrap(), &byte);
            assert_eq!(vec.get(i).unwrap(), &byte);
            assert_eq!(vec.get_mut(i).unwrap(), &byte);

            prev_byte = Some(byte);
        }
    }

    vec.as_mut_slice().fill_with(|| 1);
    assert_eq!(vec.as_slice().iter().sum::<u8>(), vec.len() as u8);

    vec.clear();
    for &byte in data {
        assert!(vec.push(&byte).is_ok());
        assert_eq!(vec.len(), 1);
        assert_eq!(vec.pop().unwrap(), byte);
        assert!(vec.is_empty());
    }

    vec.clear();
    for &byte in data {
        assert!(vec.push(&byte).is_ok());
        assert_eq!(vec.len(), 1);
        assert_eq!(vec.pop_if(|_b| true).unwrap(), byte);
        assert!(vec.is_empty());
    }
});
