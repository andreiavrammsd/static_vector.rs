# `static_vector`

[![build](https://github.com/andreiavrammsd/static_vector.rs/workflows/CI/badge.svg)](https://github.com/andreiavrammsd/static_vector.rs/actions/workflows/ci.yml)
[![codecov](https://codecov.io/gh/andreiavrammsd/static_vector.rs/graph/badge.svg?token=pCcpya0mZC)](https://codecov.io/gh/andreiavrammsd/static_vector.rs)
[![fuzz](https://github.com/andreiavrammsd/static_vector.rs/workflows/FUZZ/badge.svg)](https://github.com/andreiavrammsd/static_vector.rs/actions/workflows/fuzz.yml)
[![documentation](https://github.com/andreiavrammsd/static_vector.rs/workflows/DOC/badge.svg)](https://andreiavrammsd.github.io/static_vector.rs/)
[![license](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)

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
- `T: Default` only if [`Vec::set_len()`] is used

## Complexity

All operations are O(1) except:

| Method      | Time Complexity                            | Space Complexity                |
|-----------------------|----------------------------------|---------------------------------|
| `clear`               | O(current length)                | O(1)                            |
| `set_len`             | O(new length - current length)   | O(new length - current length)  |
| `extend_from_slice`   | O(slice length)                  | O(slice length)                 |
| `append`              | O(other vector length)           | O(other vector length)          |

## Add to project

```bash
cargo add static_vector --git https://github.com/andreiavrammsd/static_vector.rs
```

This crate is not published on [crates.io](https://crates.io/).

## Example

```rust
use static_vector::vec;

// Create a static vector with capacity 3, length 3, and elements 1, 2, 3.
let mut vec = vec![1, 2, 3];

vec.pop().unwrap();
assert_eq!(vec.len(), 2);

vec.push(4).unwrap();
assert_eq!(vec.len(), 3);
assert_eq!(vec.last(), Some(&4));

assert_eq!(vec.as_slice(), &[1, 2, 4]);

let sum_of_even_numbers = vec.iter().filter(|n| *n % 2 == 0).sum::<i32>();
assert_eq!(sum_of_even_numbers, 6);

vec.push(2).unwrap_err();
assert_eq!(vec.len(), 3);

match vec.set_len(1) {
    Ok(()) => assert_eq!(vec.len(), 1),
    Err(err) => eprintln!("{:?}", err),
}

vec.clear();
assert!(vec.is_empty());
```

See more examples in the [documentation](https://andreiavrammsd.github.io/static_vector.rs/) of [`Vec`] and in [examples](https://github.com/andreiavrammsd/static_vector.rs/tree/master/examples).

## Development on Linux

* Install [Rust](https://www.rust-lang.org/tools/install) (stable is used for main code, nightly is used only for code formatting and fuzz tests).
* If using VS Code, see [setup](https://github.com/andreiavrammsd/static_vector.rs/tree/master/.vscode).
* See [Makefile](https://github.com/andreiavrammsd/static_vector.rs/blob/master/Makefile).
