Anyhow&ensp;¯\\\_(ツ)\_/¯
=========================

[![Build Status](https://api.travis-ci.com/dtolnay/anyhow.svg?branch=master)](https://travis-ci.com/dtolnay/anyhow)
[![Latest Version](https://img.shields.io/crates/v/anyhow.svg)](https://crates.io/crates/anyhow)
[![Rust Documentation](https://img.shields.io/badge/api-rustdoc-blue.svg)](https://docs.rs/anyhow)

This library provides [`anyhow::Error`][Error], a trait object based error type
for easy idiomatic error handling in Rust applications.

[Error]: https://docs.rs/anyhow/1.0.0-alpha.1/anyhow/struct.Error.html

```toml
[dependencies]
anyhow = "=1.0.0-alpha.1"
```

<br>

## Acknowledgements

The implementation of the `anyhow::Error` type is identical to
`fehler::Exception` (https://github.com/withoutboats/fehler). This library just
exposes it under the more standard `Error` / `Result` terminology rather than
the `throw!` / `#[throws]` / `Exception` language of exceptions.

<br>

#### License

<sup>
Licensed under either of <a href="LICENSE-APACHE">Apache License, Version
2.0</a> or <a href="LICENSE-MIT">MIT license</a> at your option.
</sup>

<br>

<sub>
Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this crate by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.
</sub>
