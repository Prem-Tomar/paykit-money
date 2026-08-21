use std::cmp::Ordering;
use std::num::NonZeroU128;

/// Controls how a fractional minor-unit result is converted to a whole minor unit.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum RoundingMode {
    /// Discards the fractional remainder.
    TowardZero,
    /// Moves one minor unit farther from zero whenever a remainder exists.
    AwayFromZero,
    /// Rounds toward negative infinity.
    Floor,
    /// Rounds toward positive infinity.
    Ceiling,
    /// Rounds to the nearest minor unit, with exact halves moving away from zero.
    HalfAwayFromZero,
    /// Rounds to the nearest minor unit, with exact halves moving to the even integer.
    HalfEven,
}

impl RoundingMode {
    pub(super) fn round_magnitude(
        self,
        quotient: u128,
        remainder: u128,
        divisor: NonZeroU128,
        is_negative: bool,
    ) -> u128 {
        if remainder == 0 {
            return quotient;
        }

        let increment = match self {
            Self::TowardZero => false,
            Self::AwayFromZero => true,
            Self::Floor => is_negative,
            Self::Ceiling => !is_negative,
            Self::HalfAwayFromZero => {
                compare_remainder_to_half(remainder, divisor) != Ordering::Less
            }
            Self::HalfEven => match compare_remainder_to_half(remainder, divisor) {
                Ordering::Less => false,
                Ordering::Greater => true,
                Ordering::Equal => !quotient.is_multiple_of(2),
            },
        };

        if increment {
            // A nonzero remainder means the divisor is greater than one, so the quotient is
            // strictly smaller than the original i128 magnitude and can safely grow by one.
            quotient + 1
        } else {
            quotient
        }
    }
}

fn compare_remainder_to_half(remainder: u128, divisor: NonZeroU128) -> Ordering {
    // Comparing the remainder with the complementary part avoids overflowing remainder * 2.
    remainder.cmp(&(divisor.get() - remainder))
}
