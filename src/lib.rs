//! Exact money and currency primitives.
//!
//! Currency definitions are data-driven. This crate validates their structure but does
//! not maintain an application-specific catalogue of supported currencies.

mod currency;
mod money;

pub use currency::{Currency, CurrencyError};
pub use money::{Money, MoneyError};
