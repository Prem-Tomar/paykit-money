use std::fmt;

use super::{Money, MoneyError, MoneyRateError, Rate, RoundingMode};

/// A fee rule composed of a variable basis-point rate plus a fixed monetary amount.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct FeeFormula {
    rate: Rate,
    fixed: Money,
}

#[cfg(feature = "serde")]
impl serde::Serialize for FeeFormula {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;

        let mut state = serializer.serialize_struct("FeeFormula", 2)?;
        state.serialize_field("rate", &self.rate)?;
        state.serialize_field("fixed", &self.fixed)?;
        state.end()
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for FeeFormula {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(serde::Deserialize)]
        struct FeeFormulaWire {
            rate: Rate,
            fixed: Money,
        }

        let wire = FeeFormulaWire::deserialize(deserializer)?;

        Ok(FeeFormula::new(wire.rate, wire.fixed))
    }
}

impl FeeFormula {
    /// Creates a fee formula from a variable rate and fixed amount.
    #[must_use]
    pub const fn new(rate: Rate, fixed: Money) -> Self {
        Self { rate, fixed }
    }

    /// Returns the variable fee rate.
    #[must_use]
    pub const fn rate(&self) -> Rate {
        self.rate
    }

    /// Returns the fixed fee amount.
    #[must_use]
    pub const fn fixed(&self) -> &Money {
        &self.fixed
    }

    /// Calculates the fee for an amount using explicit rounding for the variable component.
    ///
    /// # Errors
    ///
    /// Returns [`FeeFormulaError::CurrencyMismatch`] when the input amount and fixed fee
    /// use different currency definitions.
    ///
    /// Returns [`FeeFormulaError::AmountOverflow`] when either the variable component or
    /// the final variable-plus-fixed fee cannot fit in `i128` minor units.
    pub fn calculate(&self, amount: &Money, mode: RoundingMode) -> Result<Money, FeeFormulaError> {
        let variable = amount
            .apply_rate(self.rate, mode)
            .map_err(FeeFormulaError::from)?;

        variable
            .checked_add(&self.fixed)
            .map_err(FeeFormulaError::from)
    }
}

/// An error returned when calculating a [`FeeFormula`].
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum FeeFormulaError {
    /// The charged amount and fixed fee use different currency definitions.
    CurrencyMismatch,
    /// The fee amount cannot be represented by signed `i128` minor-unit storage.
    AmountOverflow,
}

impl From<MoneyRateError> for FeeFormulaError {
    fn from(error: MoneyRateError) -> Self {
        match error {
            MoneyRateError::AmountOverflow => Self::AmountOverflow,
        }
    }
}

impl From<MoneyError> for FeeFormulaError {
    fn from(error: MoneyError) -> Self {
        match error {
            MoneyError::CurrencyMismatch => Self::CurrencyMismatch,
            MoneyError::AmountOverflow => Self::AmountOverflow,
        }
    }
}

impl fmt::Display for FeeFormulaError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CurrencyMismatch => formatter.write_str("fee formula currency mismatch"),
            Self::AmountOverflow => formatter.write_str("fee formula amount overflowed"),
        }
    }
}

impl std::error::Error for FeeFormulaError {}

/// A fee formula with optional minimum and maximum caps.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct FeeSchedule {
    formula: FeeFormula,
    minimum: Option<Money>,
    maximum: Option<Money>,
}

#[cfg(feature = "serde")]
impl serde::Serialize for FeeSchedule {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;

        let mut state = serializer.serialize_struct("FeeSchedule", 3)?;
        state.serialize_field("formula", &self.formula)?;
        state.serialize_field("minimum", &self.minimum)?;
        state.serialize_field("maximum", &self.maximum)?;
        state.end()
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for FeeSchedule {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(serde::Deserialize)]
        struct FeeScheduleWire {
            formula: FeeFormula,
            minimum: Option<Money>,
            maximum: Option<Money>,
        }

        let wire = FeeScheduleWire::deserialize(deserializer)?;

        FeeSchedule::new(wire.formula, wire.minimum, wire.maximum).map_err(serde::de::Error::custom)
    }
}

impl FeeSchedule {
    /// Creates a fee schedule from a formula and optional caps.
    ///
    /// # Errors
    ///
    /// Returns [`FeeScheduleError::CurrencyMismatch`] when a configured cap does not use
    /// the same currency definition as the formula's fixed fee.
    ///
    /// Returns [`FeeScheduleError::InvalidCapRange`] when both caps are present and the
    /// minimum is greater than the maximum.
    pub fn new(
        formula: FeeFormula,
        minimum: Option<Money>,
        maximum: Option<Money>,
    ) -> Result<Self, FeeScheduleError> {
        validate_cap_currency(&formula, minimum.as_ref())?;
        validate_cap_currency(&formula, maximum.as_ref())?;

        if let (Some(minimum), Some(maximum)) = (&minimum, &maximum)
            && minimum.minor_units() > maximum.minor_units()
        {
            return Err(FeeScheduleError::InvalidCapRange);
        }

        Ok(Self {
            formula,
            minimum,
            maximum,
        })
    }

    /// Returns the underlying fee formula.
    #[must_use]
    pub const fn formula(&self) -> &FeeFormula {
        &self.formula
    }

    /// Returns the optional minimum fee cap.
    #[must_use]
    pub const fn minimum(&self) -> Option<&Money> {
        self.minimum.as_ref()
    }

    /// Returns the optional maximum fee cap.
    #[must_use]
    pub const fn maximum(&self) -> Option<&Money> {
        self.maximum.as_ref()
    }

    /// Calculates the fee and applies configured minimum and maximum caps.
    ///
    /// # Errors
    ///
    /// Returns [`FeeScheduleError::CurrencyMismatch`] when the amount cannot be calculated
    /// against the schedule's formula currency.
    ///
    /// Returns [`FeeScheduleError::AmountOverflow`] when the underlying formula calculation
    /// overflows.
    pub fn calculate(&self, amount: &Money, mode: RoundingMode) -> Result<Money, FeeScheduleError> {
        let mut fee = self
            .formula
            .calculate(amount, mode)
            .map_err(FeeScheduleError::from)?;

        if let Some(minimum) = &self.minimum
            && fee.minor_units() < minimum.minor_units()
        {
            fee = minimum.clone();
        }

        if let Some(maximum) = &self.maximum
            && fee.minor_units() > maximum.minor_units()
        {
            fee = maximum.clone();
        }

        Ok(fee)
    }
}

fn validate_cap_currency(
    formula: &FeeFormula,
    cap: Option<&Money>,
) -> Result<(), FeeScheduleError> {
    if let Some(cap) = cap
        && cap.currency() != formula.fixed().currency()
    {
        return Err(FeeScheduleError::CurrencyMismatch);
    }

    Ok(())
}

/// An error returned when configuring or calculating a [`FeeSchedule`].
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum FeeScheduleError {
    /// The schedule formula, caps, or input amount use different currency definitions.
    CurrencyMismatch,
    /// The configured minimum fee is greater than the configured maximum fee.
    InvalidCapRange,
    /// The fee amount cannot be represented by signed `i128` minor-unit storage.
    AmountOverflow,
}

impl From<FeeFormulaError> for FeeScheduleError {
    fn from(error: FeeFormulaError) -> Self {
        match error {
            FeeFormulaError::CurrencyMismatch => Self::CurrencyMismatch,
            FeeFormulaError::AmountOverflow => Self::AmountOverflow,
        }
    }
}

impl fmt::Display for FeeScheduleError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CurrencyMismatch => formatter.write_str("fee schedule currency mismatch"),
            Self::InvalidCapRange => formatter.write_str("fee schedule cap range is invalid"),
            Self::AmountOverflow => formatter.write_str("fee schedule amount overflowed"),
        }
    }
}

impl std::error::Error for FeeScheduleError {}
