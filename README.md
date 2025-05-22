# `static_vector`

[![license](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)
[![build](https://github.com/andreiavrammsd/static_vector.rs/workflows/CI/badge.svg)](https://github.com/andreiavrammsd/static_vector.rs/actions/workflows/ci.yml)
[![codecov](https://codecov.io/gh/andreiavrammsd/static_vector.rs/graph/badge.svg?token=pCcpya0mZC)](https://codecov.io/gh/andreiavrammsd/static_vector.rs)
[![documentation](https://img.shields.io/badge/Documentation-static_vector-4EC820.svg)](https://andreiavrammsd.github.io/static_vector.rs/)

A no-std, stack-allocated vector with fixed capacity and dynamic length: `static_vector::Vec::<T, CAPACITY>`.

[`Vec`] stores elements on the stack using a fixed-size array without heap allocations.

Aims to be suitable for low-level projects and to have an API as safe and explicit as possible.
The goal is to allocate only when needed. When first constructed, the vector will not allocate.

> **Note:** It's a learning project, so there are no guarantees.

## Features

- No heap allocation (`#![no_std]` compatible)
- Supports iteration, mutable access, clearing, resizing
- Compile-time enforced capacity

## Requirements
- `CAPACITY` > 0, otherwise [`Vec::new()`] panics 
- `T: Clone` for insertion: [`Vec::push()`]
- `T: Default` only if [`Vec::set_len()`] is used

## Complexity

All operations are O(1) except:

| Method      | Time Complexity                  | Space Complexity                |
|-------------|----------------------------------|---------------------------------|
| `clear`     | O(current length)                | O(1)                            |
| `set_len`   | O(new length - current length)   | O(new length - current length)  |

## Add to project

```bash
cargo add static_vector --git https://github.com/andreiavrammsd/static_vector.rs
```

This crate is not published on [crates.io](https://crates.io/).

## Example

```rust
use static_vector::Vec;

let mut vec = Vec::<i32, 3>::new();

vec.push(&4).unwrap();
vec.push(&5).unwrap();
vec.push(&6).unwrap();
assert_eq!(vec.len(), 3);
assert_eq!(vec.first(), Some(&4));

let sum_of_even_numbers = vec.iter().filter(|n| *n % 2 == 0).sum::<i32>();
assert_eq!(sum_of_even_numbers, 10);

vec.push(&2).unwrap_err();
assert_eq!(vec.len(), 3);

match vec.set_len(1) {
    Ok(()) => assert_eq!(vec.len(), 1),
    Err(err) => eprintln!("{:?}", err),
}

vec.clear();
assert!(vec.is_empty());
```

## Development on Linux

See [Makefile](https://github.com/andreiavrammsd/static_vector.rs/blob/master/Makefile).
