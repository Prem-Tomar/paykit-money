use crate::Currency;
use std::fmt;

/// An exact monetary amount expressed in a currency's minor units.
///
/// The amount is stored as an integer, so constructing a `Money` value never introduces
/// floating-point rounding. Domain layers may decide whether a particular operation permits
/// zero or negative values.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Money {
    minor_units: i128,
    currency: Currency,
}

impl Money {
    /// Creates an exact monetary amount from minor units and a validated currency.
    #[must_use]
    pub const fn from_minor_units(minor_units: i128, currency: Currency) -> Self {
        Self {
            minor_units,
            currency,
        }
    }

    /// Returns the signed amount expressed in the currency's minor units.
    #[must_use]
    pub const fn minor_units(&self) -> i128 {
        self.minor_units
    }

    /// Returns the currency associated with this amount.
    #[must_use]
    pub const fn currency(&self) -> &Currency {
        &self.currency
    }

    pub 

    /// Adds another amount of money with the same currency.
    ///
    /// Returns a new `Money` value when both amounts use the exact same `Currency`
    /// definition and the resulting minor-unit amount fits in `i128`.
    ///
    /// # Errors
    ///
    /// Returns `MoneyError::CurrencyMismatch` when the two amounts do not share
    /// the same currency definition. The currency code and minor-unit scale must both
    /// match.
    ///
    /// Returns `MoneyError::AmountOverflow` when the addition would overflow the
    /// `i128` minor-unit storage.
    pub const fn checked_add(&self, other: &Money) -> Result<Money, MoneyError> {
        if !Self::validated_currency(self, other) {
            return Err(MoneyError::CurrencyMismatch);
        }
        match self.minor_units().checked_add(other.minor_units) {
            Some(value) => Ok(Money::from_minor_units(value, self.currency.clone())),
            None => Err(MoneyError::AmountOverflow),
        }
    }

    /// Subtracts another amount of money with the same currency.
    ///
    /// Returns a new `Money` value when both amounts use the exact same `Currency`
    /// definition and the resulting minor-unit amount fits in `i128`. Negative results
    /// are allowed by this foundational type.
    ///
    /// # Errors
    ///
    /// Returns `MoneyError::CurrencyMismatch` when the two amounts do not share
    /// the same currency definition. The currency code and minor-unit scale must both
    /// match.
    ///
    /// Returns `MoneyError::AmountOverflow` when the subtraction would overflow
    /// the `i128` minor-unit storage.
    pub const fn checked_sub(&self, other: &Money) -> Result<Money, MoneyError> {
        if !Self::validated_currency(self, other) {
            return Err(MoneyError::CurrencyMismatch);
        }
        match self.minor_units().checked_sub(other.minor_units) {
            Some(value) => Ok(Money::from_minor_units(value, self.currency.clone())),
            None => Err(MoneyError::AmountOverflow),
        }
    }

    fn validated_currency(left: &Money, right: &Money) -> bool {
        left.currency() == right.currency()
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum MoneyError {
    CurrencyMismatch,
    AmountOverflow,
}

impl fmt::Display for MoneyError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CurrencyMismatch => formatter.write_str("Currency validation failed"),
            Self::AmountOverflow => {
                formatter.write_str("Amount overflow while performing addition or subtraction")
            }
        }
    }
}

impl std::error::Error for MoneyError {}
