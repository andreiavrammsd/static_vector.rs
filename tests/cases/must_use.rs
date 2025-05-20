#![deny(warnings)]

use static_vector::Vec;

fn main() {
    let mut vec = Vec::<i32, 1>::new();
    vec.first();
    vec.last();
    vec.get(0);
    vec.get_mut(0);
    vec.pop();
    vec.pop_if(|&_i32| true);
    vec.iter();
    vec.iter_mut();

    Vec::<i32, 1>::new();
}
