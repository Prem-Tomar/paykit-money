use crate::Currency;

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

    pub fn checked_add(&self, other: &Money) -> Result<Money, MoneyError> {
        if !Self::validated_currency(self, other) {
            return Err(MoneyError::CurrencyMismatchError(
                format!("Currency do not match for given values").to_owned(),
            ));
        }
        match self.minor_units().checked_add(other.minor_units) {
            Some(value) => Ok(Money::from_minor_units(value, self.currency.clone())),
            None => {
                return Err(MoneyError::MoneyAddError(
                    format!("Could not add the value {}", other.minor_units()).to_owned(),
                ));
            }
        }
    }

    pub fn checked_sub(&self, other: &Money) -> Result<Money, MoneyError> {
        if !Self::validated_currency(self, other) {
            return Err(MoneyError::CurrencyMismatchError(
                format!("Currency do not match for given values").to_owned(),
            ));
        }
        match self.minor_units().checked_sub(other.minor_units) {
            Some(value) => Ok(Money::from_minor_units(value, self.currency.clone())),
            None => {
                return Err(MoneyError::MoneySubError(
                    format!("Could not sub the value {}", other.minor_units()).to_owned(),
                ));
            }
        }
    }

    fn validated_currency(left: &Money, right: &Money) -> bool {
        left.currency() == right.currency()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]

pub enum MoneyError {
    CurrencyMismatchError(String),
    MoneyAddError(String),
    MoneySubError(String),
}
