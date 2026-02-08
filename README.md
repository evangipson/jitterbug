# rquant
![rustc (with version)](https://img.shields.io/badge/rustc-1.56.0+-blue?style=for-the-badge&logo=rust) ![crate.rs (with version)](https://img.shields.io/crates/v/jitterbug?style=for-the-badge&logo=hackthebox&logoColor=white) ![docs.rs (with version)](https://img.shields.io/docsrs/jitterbug/latest?style=for-the-badge&logo=rust)

A true random number generator based on CPU jitter written in rust.

It allows true random number generation without seeding.

## Getting Started
1. Add jitterbug to your `Cargo.toml` file
1. Use `Jitterbug::new()` to create a new jitterbug

## Examples
### Getting a true random number
You can get a true random number by creating a `new` `Jitterbug`, then using the `RngCore` `impl` of `Jitterbug`:
```rust
use jitterbug::Jitterbug;
use rand_core::Rng;

fn main() {
    // create a new jitterbug, and unwrap for direct `Infallable` `Result`
    let mut jitter_rng = Jitterbug::new().unwrap_err();

    // generate a new `u64` number
    println!("random number: {}", jitter_rng.next_u64());

    // generate a new number in a range
    println!("random number between 0 and 100: {}", jitter_rng.gen_range(0..=100));
}
```

## Dependencies
|Crate|Purpose|
|-|-|
|[rand_core v0.10.0](https://docs.rs/rand_core/0.10.0/rand_core/)|Used to satisfy the contract for rust random number generation|