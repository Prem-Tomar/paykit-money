mod allocation;
mod rate;
mod rounding_mode;

use crate::Currency;
#[cfg(feature = "serde")]
use serde::ser::SerializeStruct;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::fmt;
use std::fmt::Display;
use std::num::NonZeroU128;

pub use allocation::MoneyAllocationError;
pub use rate::{MoneyRateError, Rate};
pub use rounding_mode::RoundingMode;

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

#[cfg(feature = "serde")]
impl Serialize for Money {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        // Keep the public wire shape explicit instead of exposing the struct layout by derive.
        let mut state = serializer.serialize_struct("Money", 2)?;

        // Serialize the full i128 amount as a decimal string so JSON consumers cannot lose
        // precision when their native number type cannot represent every i128 value.
        state.serialize_field("minor_units", &MinorUnitsAsString(self.minor_units))?;
        state.serialize_field("currency", &self.currency)?;
        state.end()
    }
}

#[cfg(feature = "serde")]
impl<'de> Deserialize<'de> for Money {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        // Accept only the documented string amount and a Currency that validates itself.
        #[derive(serde::Deserialize)]
        struct MoneyWire {
            minor_units: String,
            currency: Currency,
        }

        // Let Serde validate the object shape and primitive field types first.
        let wire = MoneyWire::deserialize(deserializer)?;

        // Convert the exact decimal string into the complete signed i128 range.
        let minor_units = wire
            .minor_units
            .parse::<i128>()
            .map_err(serde::de::Error::custom)?;

        Ok(Money::from_minor_units(minor_units, wire.currency))
    }
}

#[cfg(feature = "serde")]
struct MinorUnitsAsString(i128);

#[cfg(feature = "serde")]
impl Serialize for MinorUnitsAsString {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        // collect_str writes the Display representation as a serialized string.
        serializer.collect_str(&self.0)
    }
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

    /// Parses an exact major-unit amount using the supplied currency's minor-unit scale.
    ///
    /// Surrounding whitespace and a leading minus sign are accepted. A leading plus sign is
    /// rejected. The fractional part may contain at most as many digits as the currency scale;
    /// shorter fractions are right-padded with zeroes. Parsing uses integer arithmetic and
    /// supports the complete `i128` minor-unit range without floating-point conversion.
    ///
    /// # Errors
    ///
    /// Returns:
    ///
    /// - [`MoneyParseError::Empty`] when the trimmed input is empty.
    /// - [`MoneyParseError::InvalidFormat`] when the input is not a supported decimal form.
    /// - [`MoneyParseError::TooManyFractionalDigits`] when its precision exceeds the currency
    ///   scale.
    /// - [`MoneyParseError::AmountOverflow`] when the scaled amount does not fit in `i128`.
    ///
    /// # Examples
    ///
    /// ```
    /// use paykit_money::{Currency, Money};
    ///
    /// let usd = Currency::new("USD", 2)?;
    /// let money = Money::from_major_units("10.5", usd)?;
    ///
    /// assert_eq!(money.minor_units(), 1_050);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    ///
    /// ```
    /// use paykit_money::{Currency, Money, MoneyParseError};
    ///
    /// let usd = Currency::new("USD", 2)?;
    /// let error = Money::from_major_units("10.001", usd)
    ///     .expect_err("three fractional digits should exceed the USD scale");
    ///
    /// assert_eq!(error, MoneyParseError::TooManyFractionalDigits);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
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
        let major_units = parse_unsigned_u128(major_part)?;
        let scale_factor = 10_u128.pow(scale as u32);
        let mut magnitude = major_units
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
            let fractional_units = parse_unsigned_u128(fractional_part)?;
            let padding = scale - fractional_part.len();
            let scaled_fractional_units = fractional_units
                .checked_mul(10_u128.pow(padding as u32))
                .ok_or(MoneyParseError::AmountOverflow)?;

            // Combine major and fractional minor units without permitting overflow.
            magnitude = magnitude
                .checked_add(scaled_fractional_units)
                .ok_or(MoneyParseError::AmountOverflow)?;
        }

        // Convert the unsigned magnitude only after validating the asymmetric signed range.
        let minor_units = if is_negative {
            if magnitude == i128::MIN.unsigned_abs() {
                i128::MIN
            } else {
                i128::try_from(magnitude)
                    .map_err(|_| MoneyParseError::AmountOverflow)?
                    .checked_neg()
                    .ok_or(MoneyParseError::AmountOverflow)?
            }
        } else {
            i128::try_from(magnitude).map_err(|_| MoneyParseError::AmountOverflow)?
        };

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

    /// Divides this amount by a positive integer and explicitly rounds any fractional
    /// minor-unit result.
    ///
    /// The operation divides the stored minor units, preserves the currency, and never uses
    /// floating-point arithmetic. A zero divisor cannot be passed because [`NonZeroU128`]
    /// enforces that invariant.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::num::NonZeroU128;
    ///
    /// use paykit_money::{Currency, Money, RoundingMode};
    ///
    /// let usd = Currency::new("USD", 2)?;
    /// let amount = Money::from_minor_units(1_000, usd);
    /// let divisor = NonZeroU128::new(3).expect("three is nonzero");
    ///
    /// let share = amount.div_rounded(divisor, RoundingMode::TowardZero);
    /// assert_eq!(share.to_string(), "USD 3.33");
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub fn div_rounded(&self, divisor: NonZeroU128, mode: RoundingMode) -> Money {
        let magnitude = self.minor_units.unsigned_abs();
        let divisor_value = divisor.get();
        let quotient = magnitude / divisor_value;
        let remainder = magnitude % divisor_value;
        let is_negative = self.minor_units.is_negative();
        let rounded_magnitude = mode.round_magnitude(quotient, remainder, divisor, is_negative);
        let rounded_minor_units = restore_sign(rounded_magnitude, is_negative);

        Money::from_minor_units(rounded_minor_units, self.currency.clone())
    }

    fn validated_currency(left: &Money, right: &Money) -> bool {
        left.currency() == right.currency()
    }
}

/// Formats money as its currency code followed by its exact major-unit amount.
///
/// Fractional digits always match the currency's minor-unit scale. This representation
/// is stable and non-localized: it does not use currency symbols or digit grouping.
///
/// # Examples
///
/// ```
/// use paykit_money::{Currency, Money};
///
/// let usd = Currency::new("USD", 2)?;
/// let money = Money::from_minor_units(-1_050, usd);
///
/// assert_eq!(money.to_string(), "USD -10.50");
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
impl Display for Money {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Derive the decimal divisor from the validated currency scale.
        let scale = u32::from(self.currency.minor_units());
        let scale_factor = 10_u128.pow(scale);
        // Use an unsigned magnitude so `i128::MIN` can be formatted without overflow.
        let magnitude = self.minor_units.unsigned_abs();
        // Split the exact integer amount without introducing floating-point arithmetic.
        let major_units = magnitude / scale_factor;
        let fractional_units = magnitude % scale_factor;
        let sign = if self.minor_units.is_negative() {
            "-"
        } else {
            ""
        };
        // Zero-scale currencies omit the decimal point; other scales retain trailing zeroes.
        if scale == 0 {
            write!(formatter, "{} {sign}{major_units}", self.currency)
        } else {
            write!(
                formatter,
                "{} {sign}{major_units}.{fractional_units:0width$}",
                self.currency,
                width = scale as usize
            )
        }
    }
}

fn parse_unsigned_u128(input: &str) -> Result<u128, MoneyParseError> {
    if input.is_empty() || !input.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(MoneyParseError::InvalidFormat);
    }

    let mut value = 0_u128;
    for byte in input.bytes() {
        let digit = u128::from(byte - b'0');
        value = value
            .checked_mul(10)
            .and_then(|value| value.checked_add(digit))
            .ok_or(MoneyParseError::AmountOverflow)?;
    }

    Ok(value)
}

pub(super) fn restore_sign(magnitude: u128, is_negative: bool) -> i128 {
    if is_negative {
        // `i128::MIN` has no positive `i128` counterpart, so restore it directly.
        if magnitude == i128::MIN.unsigned_abs() {
            i128::MIN
        } else {
            -i128::try_from(magnitude)
                .expect("a rounded negative magnitude smaller than i128::MIN must fit in i128")
        }
    } else {
        i128::try_from(magnitude)
            .expect("a rounded positive magnitude derived from i128 must fit in i128")
    }
}

/// An error returned by checked arithmetic on [`Money`].
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum MoneyError {
    /// The operands use different currency codes or minor-unit scales.
    CurrencyMismatch,
    /// The result cannot be represented by the signed `i128` minor-unit storage.
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

/// An error returned when parsing a major-unit amount into [`Money`].
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum MoneyParseError {
    /// The input contains no characters after surrounding whitespace is removed.
    Empty,
    /// The input is not a supported decimal amount.
    InvalidFormat,
    /// The fractional precision exceeds the currency's minor-unit scale.
    TooManyFractionalDigits,
    /// The scaled amount cannot be represented by the signed `i128` minor-unit storage.
    AmountOverflow,
}

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
