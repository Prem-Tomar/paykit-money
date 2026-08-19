//! Exact money and currency primitives.
//!
//! Currency definitions are data-driven. This crate validates their structure but does
//! not maintain an application-specific catalogue of supported currencies.
//!
//! Amounts are stored as signed [`i128`] minor units. Parsing, checked arithmetic, and
//! formatting therefore avoid floating-point rounding.
//!
//! # Example
//!
//! ```
//! use paykit_money::{Currency, Money};
//!
//! let usd = Currency::new("USD", 2)?;
//! let subtotal = Money::from_major_units("10.50", usd.clone())?;
//! let fee = Money::from_minor_units(25, usd);
//! let total = subtotal.checked_add(&fee)?;
//!
//! assert_eq!(total.minor_units(), 1_075);
//! assert_eq!(total.to_string(), "USD 10.75");
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! # Policy boundaries
//!
//! This crate enforces structural invariants only. Applications decide:
//!
//! - which structurally valid currencies they support;
//! - whether a workflow permits zero or negative amounts;
//! - which rounding policy applies when an operation cannot remain exact;
//! - how monetary values are localized for people.

mod currency;
mod money;

pub use currency::{Currency, CurrencyError};
pub use money::{Money, MoneyError, MoneyParseError};
