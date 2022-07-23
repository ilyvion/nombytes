# NomBytes

[![Crates.io](https://img.shields.io/crates/v/nombytes)](https://crates.io/crates/nombytes)
[![Crates.io](https://img.shields.io/crates/l/nombytes)](https://crates.io/crates/nombytes)
[![Crates.io](https://img.shields.io/crates/d/nombytes)](https://crates.io/crates/nombytes)
[![Docs.io](https://docs.rs/nombytes/badge.svg)](https://docs.rs/nombytes)
[![Docs master](https://img.shields.io/static/v1?label=docs&message=master&color=5479ab)](https://alexschrod.github.io/nombytes/)
[![Rust](https://github.com/alexschrod/nombytes/actions/workflows/CI.yml/badge.svg)](https://github.com/alexschrod/nombytes/actions/workflows/CI.yml)
[![codecov](https://codecov.io/gh/alexschrod/nombytes/branch/master/graph/badge.svg?token=C8UJJM7BVJ)](https://codecov.io/gh/alexschrod/nombytes)

`nombytes` is a library that provides a wrapper for the `bytes::Bytes` byte
container.

I originally made this so that I could have a function take a file name path
and return parsed values that still had references to the loaded file without
running into the lifetime issues associated with `&[u8]` and `&str` that
would prevent me from doing so. I decided to release it as a crate so that
others can make use of my efforts too.

This library has been tested to work with `bytes` down to v5.3.0 and `nom` down
to v6.0.0 and has been marked as such in its `Cargo.toml`.

## Usage

Put this in your `Cargo.toml`:

```toml
[dependencies]
nombytes = "0.1"
```

## Features

### `miette`

With the `miette` feature enabled, the `NomBytes` implements its
`SourceCode` trait so it can be used directly with `miette`'s
`#[source_code]` error attribute. This feature also enables the `std`
feature.

This library has been tested to work with `miette` down to v3.0.0 and
has been marked as such in its `Cargo.toml`.

## `std`

Enabled by default; allows creating `NomBytes` directly from `String`s
through a `From<String>` impl. With this feature turned off, this crate
is `#![no_std]` compatible.

## Example

Borrowed from the `nom` crate, using `NomBytes` instead of `&str`. Had to be
modified slightly because `NomBytes` acts as `&[u8]` rather than as `&str`.

```rust
use nom::{
  IResult,
  bytes::complete::{tag, take_while_m_n},
  combinator::map_res,
  sequence::tuple};
use nombytes::NomBytes;

#[derive(Debug,PartialEq)]
pub struct Color {
  pub red:     u8,
  pub green:   u8,
  pub blue:    u8,
}

fn from_hex(input: NomBytes) -> Result<u8, std::num::ParseIntError> {
  u8::from_str_radix(input.to_str(), 16)
}

fn is_hex_digit(c: u8) -> bool {
  (c as char).is_digit(16)
}

fn hex_primary(input: NomBytes) -> IResult<NomBytes, u8> {
  map_res(
    take_while_m_n(2, 2, is_hex_digit),
    from_hex
  )(input)
}

fn hex_color(input: NomBytes) -> IResult<NomBytes, Color> {
  let (input, output) = tag("#")(input)?;

  let (input, (red, green, blue)) = tuple((hex_primary, hex_primary, hex_primary))(input)?;

  Ok((input, Color { red, green, blue }))
}

fn main() {
  assert!(matches!(hex_color(NomBytes::from("#2F14DF")),
    Ok((r, Color {
      red: 47,
      green: 20,
      blue: 223,
    })) if r.to_str() == ""));
}
```

## License

Licensed under either of

* Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
