use std::fmt;
use std::num::NonZeroU128;

use super::{Money, RoundingMode, restore_sign};

/// A non-negative rate represented in basis points.
///
/// One basis point is one hundredth of one percent, so `10_000` basis points equals
/// `100.00%`.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Rate {
    basis_points: u32,
}

impl Rate {
    const BASIS_POINT_DENOMINATOR: u128 = 10_000;

    /// Creates a rate from basis points.
    #[must_use]
    pub const fn from_basis_points(basis_points: u32) -> Self {
        Self { basis_points }
    }

    /// Returns the rate expressed as basis points.
    #[must_use]
    pub const fn basis_points(self) -> u32 {
        self.basis_points
    }

    fn denominator() -> NonZeroU128 {
        NonZeroU128::new(Self::BASIS_POINT_DENOMINATOR)
            .expect("basis point denominator must be nonzero")
    }
}

impl Money {
    /// Applies a basis-point rate using explicit rounding.
    ///
    /// This operation uses integer arithmetic only. It never converts the amount or rate
    /// to floating-point values.
    ///
    /// # Errors
    ///
    /// Returns [`MoneyRateError::AmountOverflow`] when the rated amount cannot fit in
    /// the signed `i128` minor-unit storage.
    pub fn apply_rate(&self, rate: Rate, mode: RoundingMode) -> Result<Money, MoneyRateError> {
        let magnitude = self.minor_units().unsigned_abs();
        let is_negative = self.minor_units().is_negative();
        let (quotient, remainder) = mul_div(magnitude, u128::from(rate.basis_points()), 10_000)?;
        let rounded_magnitude =
            mode.round_magnitude(quotient, remainder, Rate::denominator(), is_negative);

        if exceeds_signed_range(rounded_magnitude, is_negative) {
            return Err(MoneyRateError::AmountOverflow);
        }

        Ok(Money::from_minor_units(
            restore_sign(rounded_magnitude, is_negative),
            self.currency().clone(),
        ))
    }
}

fn mul_div(
    value: u128,
    multiplier: u128,
    denominator: u128,
) -> Result<(u128, u128), MoneyRateError> {
    let mut result_quotient = 0_u128;
    let mut result_remainder = 0_u128;
    let mut addend_quotient = value / denominator;
    let mut addend_remainder = value % denominator;
    let mut remaining_multiplier = multiplier;

    while remaining_multiplier > 0 {
        if remaining_multiplier & 1 == 1 {
            add_quotient_remainder(
                &mut result_quotient,
                &mut result_remainder,
                addend_quotient,
                addend_remainder,
                denominator,
            )?;
        }

        remaining_multiplier >>= 1;
        if remaining_multiplier > 0 {
            (addend_quotient, addend_remainder) =
                double_quotient_remainder(addend_quotient, addend_remainder, denominator)?;
        }
    }

    Ok((result_quotient, result_remainder))
}

fn add_quotient_remainder(
    target_quotient: &mut u128,
    target_remainder: &mut u128,
    addend_quotient: u128,
    addend_remainder: u128,
    denominator: u128,
) -> Result<(), MoneyRateError> {
    *target_quotient = target_quotient
        .checked_add(addend_quotient)
        .ok_or(MoneyRateError::AmountOverflow)?;

    if addend_remainder == 0 {
        return Ok(());
    }

    let carry_threshold = denominator - addend_remainder;
    if *target_remainder >= carry_threshold {
        *target_remainder -= carry_threshold;
        *target_quotient = target_quotient
            .checked_add(1)
            .ok_or(MoneyRateError::AmountOverflow)?;
    } else {
        *target_remainder += addend_remainder;
    }

    Ok(())
}

fn double_quotient_remainder(
    quotient: u128,
    remainder: u128,
    denominator: u128,
) -> Result<(u128, u128), MoneyRateError> {
    let mut doubled_quotient = quotient
        .checked_mul(2)
        .ok_or(MoneyRateError::AmountOverflow)?;

    let doubled_remainder = if remainder >= denominator - remainder {
        doubled_quotient = doubled_quotient
            .checked_add(1)
            .ok_or(MoneyRateError::AmountOverflow)?;
        remainder - (denominator - remainder)
    } else {
        remainder + remainder
    };

    Ok((doubled_quotient, doubled_remainder))
}

fn exceeds_signed_range(magnitude: u128, is_negative: bool) -> bool {
    if is_negative {
        magnitude > i128::MIN.unsigned_abs()
    } else {
        magnitude > i128::MAX as u128
    }
}

/// An error returned when applying a [`Rate`] to [`Money`].
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum MoneyRateError {
    /// The rated amount cannot be represented by the signed `i128` minor-unit storage.
    AmountOverflow,
}

impl fmt::Display for MoneyRateError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AmountOverflow => formatter.write_str("rated money amount overflowed"),
        }
    }
}

impl std::error::Error for MoneyRateError {}
