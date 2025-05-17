# StaticVector

[![license](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)
[![build](https://github.com/andreiavrammsd/static_vector.rs/workflows/CI/badge.svg)](https://github.com/andreiavrammsd/static_vector.rs/actions/workflows/ci.yml)
[![codecov](https://codecov.io/gh/andreiavrammsd/static_vector.rs/graph/badge.svg?token=pCcpya0mZC)](https://codecov.io/gh/andreiavrammsd/static_vector.rs)

A no-std, stack-allocated vector with fixed capacity and dynamic length.

`StaticVector` stores elements on the stack using a fixed-size array without heap allocations.

Aims to be suitable for low-level projects and to have an API as safe and explicit as possible.
The goal is to allocate only when needed. When first constructed, the vector will not allocate.

> **Note:** It's a learning project, so there are no guarantees.

## Features

- No heap allocation (`#![no_std]` compatible)
- Constant-time indexed access
- Supports iteration, mutable access, clearing, resizing
- Compile-time enforced capacity

## Requirements

- `T: Clone` for insertion: `push`
- `T: Default` only if `set_len` is used
- `CAPACITY > 0`

## Example

```rust
use static_vector::StaticVector;

let mut vec = StaticVector::<i32, 4>::new();
vec.push(&1).unwrap();
vec.push(&2).unwrap();
assert_eq!(vec.len(), 2);
```

## Development on Linux

See [Makefile](Makefile).
