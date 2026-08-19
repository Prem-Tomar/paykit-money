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

    pub fn from_major_units(input: &str, currency: Currency) -> Result<Self, MoneyParseError> {
        // Normalize API/user input before parsing the numeric content.
        let input = input.trim();
        if input.is_empty() {
            return Err(MoneyParseError::Empty);
        }

        // Capture an optional sign and keep the remaining parser unsigned.
        let (is_negative, unsigned_input) = match input.as_bytes()[0] {
            b'-' => (true, &input[1..]),
            b'+' => return Err(MoneyParseError::InvalidFormat),
            _ => (false, input),
        };

        if unsigned_input.is_empty() {
            return Err(MoneyParseError::InvalidFormat);
        }

        // Split once into major and fractional parts; multiple decimal points are invalid.
        let mut parts = unsigned_input.split('.');
        let major_part = parts.next().ok_or(MoneyParseError::InvalidFormat)?;
        let fractional_part = parts.next();

        if parts.next().is_some() || major_part.is_empty() {
            return Err(MoneyParseError::InvalidFormat);
        }

        // Convert whole major units into minor units using the currency scale.
        let scale = currency.minor_units() as usize;
        let major_units = parse_unsigned_i128(major_part)?;
        let scale_factor = 10_i128.pow(scale as u32);
        let mut minor_units = major_units
            .checked_mul(scale_factor)
            .ok_or(MoneyParseError::AmountOverflow)?;

        if let Some(fractional_part) = fractional_part {
            // Reject ambiguous or over-precise fractional input instead of rounding it.
            if fractional_part.is_empty() {
                return Err(MoneyParseError::InvalidFormat);
            }
            if fractional_part.len() > scale {
                return Err(MoneyParseError::TooManyFractionalDigits);
            }

            // Right-pad short fractions so "10.5" at scale 2 becomes 1050 minor units.
            let fractional_units = parse_unsigned_i128(fractional_part)?;
            let padding = scale - fractional_part.len();
            let scaled_fractional_units = fractional_units
                .checked_mul(10_i128.pow(padding as u32))
                .ok_or(MoneyParseError::AmountOverflow)?;

            // Combine major and fractional minor units without permitting overflow.
            minor_units = minor_units
                .checked_add(scaled_fractional_units)
                .ok_or(MoneyParseError::AmountOverflow)?;
        }

        // Apply the sign only after the absolute amount is known to fit.
        if is_negative {
            minor_units = minor_units
                .checked_neg()
                .ok_or(MoneyParseError::AmountOverflow)?;
        }

        // Store the exact integer amount; no floating-point conversion is involved.
        Ok(Money::from_minor_units(minor_units, currency))
    }

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
    pub fn checked_add(&self, other: &Money) -> Result<Money, MoneyError> {
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
    pub fn checked_sub(&self, other: &Money) -> Result<Money, MoneyError> {
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

fn parse_unsigned_i128(input: &str) -> Result<i128, MoneyParseError> {
    if input.is_empty() || !input.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(MoneyParseError::InvalidFormat);
    }

    let mut value = 0_i128;
    for byte in input.bytes() {
        let digit = i128::from(byte - b'0');
        value = value
            .checked_mul(10)
            .and_then(|value| value.checked_add(digit))
            .ok_or(MoneyParseError::AmountOverflow)?;
    }

    Ok(value)
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum MoneyError {
    CurrencyMismatch,
    AmountOverflow,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum MoneyParseError {
    Empty,
    InvalidFormat,
    TooManyFractionalDigits,
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

impl fmt::Display for MoneyParseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => formatter.write_str("money amount is empty"),
            Self::InvalidFormat => formatter.write_str("money amount format is invalid"),
            Self::TooManyFractionalDigits => {
                formatter.write_str("money amount has too many fractional digits")
            }
            Self::AmountOverflow => formatter.write_str("money amount overflowed"),
        }
    }
}

impl std::error::Error for MoneyParseError {}
