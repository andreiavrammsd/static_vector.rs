#![deny(warnings)]

use static_vector::Vec;

fn main() {
    let mut vec = Vec::<i32, 1>::new();
    vec.is_empty();
    vec.is_full();
    vec.capacity();
    vec.len();
    vec.push(&1);
    vec.set_len(1);
    vec.first();
    vec.first_mut();
    vec.last();
    vec.last_mut();
    vec.get(0);
    vec.get_mut(0);
    vec.pop();
    vec.pop_if(|&_i32| true);
    vec.iter();
    vec.iter_mut();

    Vec::<i32, 1>::new();
}
