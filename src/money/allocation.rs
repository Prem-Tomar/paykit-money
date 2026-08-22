use std::fmt;
use std::num::NonZeroU8;

use super::{Money, restore_sign};

impl Money {
    /// Splits this amount into equal parts while preserving the exact original total.
    #[must_use]
    pub fn allocate(&self, parts: NonZeroU8) -> Vec<Money> {
        let count = parts.get();
        let divisor = u128::from(count);
        let magnitude = self.minor_units().unsigned_abs();
        let quotient = magnitude / divisor;
        let remainder = magnitude % divisor;
        let is_negative = self.minor_units().is_negative();
        let mut allocations = Vec::with_capacity(usize::from(count));

        for _ in 0..usize::from(count - 1) {
            allocations.push(self.with_magnitude(quotient, is_negative));
        }

        allocations.push(self.with_magnitude(quotient + remainder, is_negative));
        allocations
    }

    /// Splits this amount according to non-negative integer weights.
    ///
    /// The full rounding remainder is assigned to the last output entry so the returned
    /// allocations always add back to the original amount exactly.
    ///
    /// # Errors
    ///
    /// Returns [`MoneyAllocationError::EmptyWeights`] when no weights are supplied.
    /// Returns [`MoneyAllocationError::ZeroTotalWeight`] when every supplied weight is zero.
    /// Returns [`MoneyAllocationError::WeightOverflow`] if checked internal allocation
    /// arithmetic cannot represent an intermediate value.
    pub fn allocate_weighted(&self, weights: &[u32]) -> Result<Vec<Money>, MoneyAllocationError> {
        if weights.is_empty() {
            return Err(MoneyAllocationError::EmptyWeights);
        }

        let total_weight = weights.iter().try_fold(0_u128, |total, &weight| {
            total
                .checked_add(u128::from(weight))
                .ok_or(MoneyAllocationError::WeightOverflow)
        })?;

        if total_weight == 0 {
            return Err(MoneyAllocationError::ZeroTotalWeight);
        }

        let magnitude = self.minor_units().unsigned_abs();
        let is_negative = self.minor_units().is_negative();
        let mut allocated_magnitude = 0_u128;
        let mut allocations = Vec::with_capacity(weights.len());

        for &weight in &weights[..weights.len() - 1] {
            let share = mul_div_floor(magnitude, u128::from(weight), total_weight)?;
            allocated_magnitude = allocated_magnitude
                .checked_add(share)
                .ok_or(MoneyAllocationError::WeightOverflow)?;
            allocations.push(self.with_magnitude(share, is_negative));
        }

        let last_share = magnitude
            .checked_sub(allocated_magnitude)
            .ok_or(MoneyAllocationError::WeightOverflow)?;
        allocations.push(self.with_magnitude(last_share, is_negative));

        Ok(allocations)
    }

    fn with_magnitude(&self, magnitude: u128, is_negative: bool) -> Money {
        Money::from_minor_units(
            restore_sign(magnitude, is_negative),
            self.currency().clone(),
        )
    }
}

fn mul_div_floor(
    value: u128,
    multiplier: u128,
    denominator: u128,
) -> Result<u128, MoneyAllocationError> {
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

    Ok(result_quotient)
}

fn add_quotient_remainder(
    target_quotient: &mut u128,
    target_remainder: &mut u128,
    addend_quotient: u128,
    addend_remainder: u128,
    denominator: u128,
) -> Result<(), MoneyAllocationError> {
    *target_quotient = target_quotient
        .checked_add(addend_quotient)
        .ok_or(MoneyAllocationError::WeightOverflow)?;

    if addend_remainder == 0 {
        return Ok(());
    }

    let carry_threshold = denominator - addend_remainder;
    if *target_remainder >= carry_threshold {
        *target_remainder -= carry_threshold;
        *target_quotient = target_quotient
            .checked_add(1)
            .ok_or(MoneyAllocationError::WeightOverflow)?;
    } else {
        *target_remainder += addend_remainder;
    }

    Ok(())
}

fn double_quotient_remainder(
    quotient: u128,
    remainder: u128,
    denominator: u128,
) -> Result<(u128, u128), MoneyAllocationError> {
    let mut doubled_quotient = quotient
        .checked_mul(2)
        .ok_or(MoneyAllocationError::WeightOverflow)?;

    let doubled_remainder = if remainder >= denominator - remainder {
        doubled_quotient = doubled_quotient
            .checked_add(1)
            .ok_or(MoneyAllocationError::WeightOverflow)?;
        remainder - (denominator - remainder)
    } else {
        remainder + remainder
    };

    Ok((doubled_quotient, doubled_remainder))
}

/// An error returned when weighted allocation input cannot produce valid allocations.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum MoneyAllocationError {
    /// No weights were supplied.
    EmptyWeights,
    /// Every supplied weight was zero.
    ZeroTotalWeight,
    /// Internal weighted allocation arithmetic exceeded the supported integer range.
    WeightOverflow,
}

impl fmt::Display for MoneyAllocationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyWeights => formatter.write_str("allocation weights are empty"),
            Self::ZeroTotalWeight => formatter.write_str("allocation weights sum to zero"),
            Self::WeightOverflow => formatter.write_str("allocation weight arithmetic overflowed"),
        }
    }
}

impl std::error::Error for MoneyAllocationError {}
