# paykit-money

Exact money and currency primitives for Rust.

`paykit-money` represents amounts as signed `i128` minor units. It does not use
floating-point arithmetic, ship a currency catalogue, perform localization, or decide
whether a business operation may use zero or negative amounts.

## Usage

```rust
use paykit_money::{Currency, Money};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let usd = Currency::new("USD", 2)?;
    let price = Money::from_major_units("10.50", usd.clone())?;
    let fee = Money::from_minor_units(25, usd);
    let total = price.checked_add(&fee)?;

    assert_eq!(total.minor_units(), 1_075);
    assert_eq!(total.to_string(), "USD 10.75");
    Ok(())
}
```

## Invariants

A `Currency` is structurally valid when:

- its code is exactly three uppercase ASCII letters;
- its minor-unit scale is between 0 and 18, inclusive.

A `Money` value always contains a validated `Currency` and an exact signed `i128`
minor-unit amount. Arithmetic succeeds only when both the code and minor-unit scale
match, and only when the result fits in `i128`.

Applications remain responsible for supported-currency policy. For example, the crate
accepts `Currency::new("XYZ", 4)` even when an application does not support `XYZ`.

## Parsing and formatting

`Money::from_major_units` accepts surrounding whitespace and an optional leading minus
sign. It rejects a leading plus sign, malformed decimal input, excess fractional digits,
and values outside the complete `i128` minor-unit range. Short fractions are padded to
the currency scale; values are never rounded silently.

```rust
use paykit_money::{Currency, Money, MoneyParseError};

# fn main() -> Result<(), Box<dyn std::error::Error>> {
let usd = Currency::new("USD", 2)?;
let amount = Money::from_major_units("-10.5", usd.clone())?;
assert_eq!(amount.to_string(), "USD -10.50");

assert_eq!(
    Money::from_major_units("10.001", usd),
    Err(MoneyParseError::TooManyFractionalDigits),
);
# Ok(())
# }
```

`Display` is stable and non-localized. Its form is `<CODE> <AMOUNT>`, with exactly the
currency scale's number of fractional digits. It does not add symbols or digit grouping.

## Validation

```text
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test
cargo doc --no-deps
```

## Status

The crate is under active development. Its APIs may change before a stable release.
