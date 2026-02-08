# jitterbug
![rustc (with version)](https://img.shields.io/badge/rustc-1.56.0+-blue?style=for-the-badge&logo=rust) ![crate.rs (with version)](https://img.shields.io/crates/v/jitterbug?style=for-the-badge&logo=hackthebox&logoColor=white) ![docs.rs (with version)](https://img.shields.io/docsrs/jitterbug/latest?style=for-the-badge&logo=rust)

A true random number generator based on CPU jitter written in rust.

It allows true random number generation without seeding.

## Getting Started
1. Add the latest version of `jitterbug` to your `Cargo.toml` file
1. Use `Jitterbug::new()` to create a new jitterbug

## Examples
### Getting a true random number
You can get a true random number by creating a `new` `Jitterbug`, then using the `RngCore` `impl` of `Jitterbug`:
```rust
use jitterbug::Jitterbug;
use rand_core::Rng;

fn main() {
    // create a new jitterbug, and unwrap for direct
    // `Infallable` `Result`
    let mut jitter_rng = Jitterbug::new();

    // generate a new `u64` number
    let random_number = jitter_rng.next_u64();
    println!("random number: {random_number}");
}
```

## Dependencies
|Crate|Purpose|
|-|-|
|[rand_core v0.10.0](https://docs.rs/rand_core/0.10.0/rand_core/)|Used to satisfy the contract for rust random number generation|