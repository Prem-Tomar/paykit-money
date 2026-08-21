use std::num::{NonZeroU8, NonZeroU128};

use paykit_money::{Currency, Money, MoneyError, MoneyParseError, RoundingMode};

fn currency(code: &str, minor_units: u8) -> Currency {
    Currency::new(code, minor_units)
        .unwrap_or_else(|error| panic!("expected a valid currency definition: {error}"))
}

fn divisor(value: u128) -> NonZeroU128 {
    NonZeroU128::new(value).expect("test divisor must be nonzero")
}

fn parts(value: u8) -> NonZeroU8 {
    NonZeroU8::new(value).expect("test allocation parts must be nonzero")
}

fn minor_units(values: &[Money]) -> Vec<i128> {
    values.iter().map(Money::minor_units).collect()
}

#[test]
fn represents_fractional_major_units_exactly() {
    let money = Money::from_minor_units(12_345, currency("INR", 2));

    assert_eq!(money.minor_units(), 12_345);
    assert_eq!(money.currency().code(), "INR");
    assert_eq!(money.currency().minor_units(), 2);
}

#[test]
fn represents_currency_without_fractional_minor_units() {
    let money = Money::from_minor_units(500, currency("JPY", 0));

    assert_eq!(money.minor_units(), 500);
    assert_eq!(money.currency().code(), "JPY");
}

#[test]
fn preserves_zero_amount() {
    let money = Money::from_minor_units(0, currency("USD", 2));

    assert_eq!(money.minor_units(), 0);
}

#[test]
fn preserves_negative_amount() {
    let money = Money::from_minor_units(-250, currency("USD", 2));

    assert_eq!(money.minor_units(), -250);
}

#[test]
fn displays_positive_money_with_fixed_fractional_precision() {
    let money = Money::from_minor_units(1_050, currency("USD", 2));

    assert_eq!(money.to_string(), "USD 10.50");
}

#[test]
fn displays_negative_money_with_sign_before_amount() {
    let money = Money::from_minor_units(-1_050, currency("USD", 2));

    assert_eq!(money.to_string(), "USD -10.50");
}

#[test]
fn displays_zero_with_currency_precision() {
    let money = Money::from_minor_units(0, currency("USD", 2));

    assert_eq!(money.to_string(), "USD 0.00");
}

#[test]
fn displays_money_without_fractional_minor_units() {
    let money = Money::from_minor_units(500, currency("JPY", 0));

    assert_eq!(money.to_string(), "JPY 500");
}

#[test]
fn displays_sub_major_unit_amount_with_leading_zeroes() {
    let money = Money::from_minor_units(5, currency("USD", 2));

    assert_eq!(money.to_string(), "USD 0.05");
}

#[test]
fn displays_custom_currency_scale() {
    let money = Money::from_minor_units(12_030, currency("XYZ", 4));

    assert_eq!(money.to_string(), "XYZ 1.2030");
}

#[test]
fn displays_one_minor_unit_at_maximum_supported_scale() {
    let money = Money::from_minor_units(1, currency("MAX", Currency::MAX_MINOR_UNITS));

    assert_eq!(money.to_string(), "MAX 0.000000000000000001");
}

#[test]
fn displays_minimum_minor_unit_amount_without_overflow() {
    let money = Money::from_minor_units(i128::MIN, currency("JPY", 0));

    assert_eq!(
        money.to_string(),
        "JPY -170141183460469231731687303715884105728"
    );
}

#[test]
fn parses_decimal_major_units_exactly() {
    let money = Money::from_major_units("10.50", currency("USD", 2))
        .expect("valid decimal amount should parse");

    assert_eq!(money.minor_units(), 1_050);
    assert_eq!(money.currency().code(), "USD");
}

#[test]
fn parses_short_fraction_by_padding_to_currency_scale() {
    let money = Money::from_major_units("10.5", currency("USD", 2))
        .expect("short decimal amount should parse");

    assert_eq!(money.minor_units(), 1_050);
}

#[test]
fn parses_whole_major_units_using_currency_scale() {
    let money =
        Money::from_major_units("10", currency("USD", 2)).expect("whole amount should parse");

    assert_eq!(money.minor_units(), 1_000);
}

#[test]
fn parses_currency_without_fractional_minor_units() {
    let money =
        Money::from_major_units("500", currency("JPY", 0)).expect("whole JPY amount should parse");

    assert_eq!(money.minor_units(), 500);
    assert_eq!(money.currency().code(), "JPY");
}

#[test]
fn parses_negative_major_units() {
    let money = Money::from_major_units("-10.50", currency("USD", 2))
        .expect("negative amount should parse");

    assert_eq!(money.minor_units(), -1_050);
}

#[test]
fn parses_amount_with_surrounding_whitespace() {
    let money = Money::from_major_units("  10.50  ", currency("USD", 2))
        .expect("trimmed amount should parse");

    assert_eq!(money.minor_units(), 1_050);
}

#[test]
fn rejects_explicit_plus_sign() {
    let result = Money::from_major_units("+10.50", currency("USD", 2));

    assert_eq!(result, Err(MoneyParseError::InvalidFormat));
}

#[test]
fn rejects_excess_fractional_precision() {
    let result = Money::from_major_units("10.001", currency("USD", 2));

    assert_eq!(result, Err(MoneyParseError::TooManyFractionalDigits));
}

#[test]
fn rejects_fractional_digits_for_zero_scale_currency() {
    let result = Money::from_major_units("500.1", currency("JPY", 0));

    assert_eq!(result, Err(MoneyParseError::TooManyFractionalDigits));
}

#[test]
fn rejects_empty_major_unit_input() {
    let result = Money::from_major_units("", currency("USD", 2));

    assert_eq!(result, Err(MoneyParseError::Empty));
}

#[test]
fn rejects_malformed_major_unit_input() {
    let usd = currency("USD", 2);

    for input in ["abc", "10.2.3", "10.", ".50", "+", "-"] {
        assert_eq!(
            Money::from_major_units(input, usd.clone()),
            Err(MoneyParseError::InvalidFormat)
        );
    }
}

#[test]
fn rejects_major_unit_amount_overflow() {
    let result = Money::from_major_units(
        "170141183460469231731687303715884105728",
        currency("USD", 2),
    );

    assert_eq!(result, Err(MoneyParseError::AmountOverflow));
}

#[test]
fn parses_maximum_i128_minor_unit_amount() {
    let money = Money::from_major_units(
        "170141183460469231731687303715884105727",
        currency("JPY", 0),
    )
    .expect("maximum i128 amount should parse");

    assert_eq!(money.minor_units(), i128::MAX);
}

#[test]
fn parses_minimum_i128_minor_unit_amount() {
    let money = Money::from_major_units(
        "-170141183460469231731687303715884105728",
        currency("JPY", 0),
    )
    .expect("minimum i128 amount should parse");

    assert_eq!(money.minor_units(), i128::MIN);
}

#[test]
fn parses_minimum_i128_minor_unit_amount_with_fractional_scale() {
    let money = Money::from_major_units(
        "-1701411834604692317316873037158841057.28",
        currency("USD", 2),
    )
    .expect("minimum i128 amount should parse at a nonzero scale");

    assert_eq!(money.minor_units(), i128::MIN);
    assert_eq!(
        money.to_string(),
        "USD -1701411834604692317316873037158841057.28"
    );
}

#[test]
fn rejects_amount_above_maximum_i128() {
    let result = Money::from_major_units(
        "170141183460469231731687303715884105728",
        currency("JPY", 0),
    );

    assert_eq!(result, Err(MoneyParseError::AmountOverflow));
}

#[test]
fn rejects_amount_below_minimum_i128() {
    let result = Money::from_major_units(
        "-170141183460469231731687303715884105729",
        currency("JPY", 0),
    );

    assert_eq!(result, Err(MoneyParseError::AmountOverflow));
}

#[test]
fn equal_values_compare_equal() {
    let left = Money::from_minor_units(1_000, currency("USD", 2));
    let right = Money::from_minor_units(1_000, currency("USD", 2));

    assert_eq!(left, right);
}

#[test]
fn different_amounts_compare_unequal() {
    let left = Money::from_minor_units(1_000, currency("USD", 2));
    let right = Money::from_minor_units(2_000, currency("USD", 2));

    assert_ne!(left, right);
}

#[test]
fn different_currencies_compare_unequal() {
    let usd = Money::from_minor_units(1_000, currency("USD", 2));
    let eur = Money::from_minor_units(1_000, currency("EUR", 2));

    assert_ne!(usd, eur);
}

#[test]
fn checked_add_combines_same_currency_amounts() {
    let usd = currency("USD", 2);
    let left = Money::from_minor_units(1_000, usd.clone());
    let right = Money::from_minor_units(250, usd);

    let result = left
        .checked_add(&right)
        .expect("same-currency addition should succeed");

    assert_eq!(result.minor_units(), 1_250);
    assert_eq!(result.currency().code(), "USD");
}

#[test]
fn checked_sub_subtracts_same_currency_amounts() {
    let usd = currency("USD", 2);
    let left = Money::from_minor_units(1_000, usd.clone());
    let right = Money::from_minor_units(250, usd);

    let result = left
        .checked_sub(&right)
        .expect("same-currency subtraction should succeed");

    assert_eq!(result.minor_units(), 750);
    assert_eq!(result.currency().code(), "USD");
}

#[test]
fn checked_sub_allows_negative_result() {
    let usd = currency("USD", 2);
    let left = Money::from_minor_units(250, usd.clone());
    let right = Money::from_minor_units(1_000, usd);

    let result = left
        .checked_sub(&right)
        .expect("negative money values are allowed by this foundational type");

    assert_eq!(result.minor_units(), -750);
    assert_eq!(result.currency().code(), "USD");
}

#[test]
fn checked_add_rejects_cross_currency_amounts() {
    let usd = Money::from_minor_units(1_000, currency("USD", 2));
    let eur = Money::from_minor_units(250, currency("EUR", 2));

    assert_eq!(usd.checked_add(&eur), Err(MoneyError::CurrencyMismatch));
}

#[test]
fn checked_sub_rejects_cross_currency_amounts() {
    let usd = Money::from_minor_units(1_000, currency("USD", 2));
    let eur = Money::from_minor_units(250, currency("EUR", 2));

    assert_eq!(usd.checked_sub(&eur), Err(MoneyError::CurrencyMismatch));
}

#[test]
fn checked_add_rejects_same_code_with_different_minor_unit_scale() {
    let usd_cents = Money::from_minor_units(1_000, currency("USD", 2));
    let usd_mills = Money::from_minor_units(250, currency("USD", 3));

    assert_eq!(
        usd_cents.checked_add(&usd_mills),
        Err(MoneyError::CurrencyMismatch)
    );
}

#[test]
fn checked_sub_rejects_same_code_with_different_minor_unit_scale() {
    let usd_cents = Money::from_minor_units(1_000, currency("USD", 2));
    let usd_mills = Money::from_minor_units(250, currency("USD", 3));

    assert_eq!(
        usd_cents.checked_sub(&usd_mills),
        Err(MoneyError::CurrencyMismatch)
    );
}

#[test]
fn checked_add_rejects_integer_overflow() {
    let usd = currency("USD", 2);
    let left = Money::from_minor_units(i128::MAX, usd.clone());
    let right = Money::from_minor_units(1, usd);

    assert_eq!(left.checked_add(&right), Err(MoneyError::AmountOverflow));
}

#[test]
fn checked_sub_rejects_integer_overflow() {
    let usd = currency("USD", 2);
    let left = Money::from_minor_units(i128::MIN, usd.clone());
    let right = Money::from_minor_units(1, usd);

    assert_eq!(left.checked_sub(&right), Err(MoneyError::AmountOverflow));
}

#[test]
fn displays_currency_mismatch_error() {
    let error = MoneyError::CurrencyMismatch;

    assert_eq!(error.to_string(), "Currency validation failed");
}

#[test]
fn displays_amount_overflow_error() {
    let error = MoneyError::AmountOverflow;

    assert_eq!(
        error.to_string(),
        "Amount overflow while performing addition or subtraction"
    );
}

#[test]
fn displays_empty_parse_error() {
    assert_eq!(MoneyParseError::Empty.to_string(), "money amount is empty");
}

#[test]
fn displays_invalid_format_parse_error() {
    assert_eq!(
        MoneyParseError::InvalidFormat.to_string(),
        "money amount format is invalid"
    );
}

#[test]
fn displays_excess_precision_parse_error() {
    assert_eq!(
        MoneyParseError::TooManyFractionalDigits.to_string(),
        "money amount has too many fractional digits"
    );
}

#[test]
fn displays_parse_overflow_error() {
    assert_eq!(
        MoneyParseError::AmountOverflow.to_string(),
        "money amount overflowed"
    );
}

#[test]
fn rounded_division_returns_exact_quotient_for_every_mode() {
    let usd = currency("USD", 2);
    let positive = Money::from_minor_units(1_200, usd.clone());
    let negative = Money::from_minor_units(-1_200, usd);
    let modes = [
        RoundingMode::TowardZero,
        RoundingMode::AwayFromZero,
        RoundingMode::Floor,
        RoundingMode::Ceiling,
        RoundingMode::HalfAwayFromZero,
        RoundingMode::HalfEven,
    ];

    for mode in modes {
        assert_eq!(positive.div_rounded(divisor(3), mode).minor_units(), 400);
        assert_eq!(negative.div_rounded(divisor(3), mode).minor_units(), -400);
    }
}

#[test]
fn rounds_toward_zero_for_positive_and_negative_amounts() {
    let usd = currency("USD", 2);
    let positive = Money::from_minor_units(1_000, usd.clone());
    let negative = Money::from_minor_units(-1_000, usd);

    assert_eq!(
        positive
            .div_rounded(divisor(3), RoundingMode::TowardZero)
            .minor_units(),
        333
    );
    assert_eq!(
        negative
            .div_rounded(divisor(3), RoundingMode::TowardZero)
            .minor_units(),
        -333
    );
}

#[test]
fn rounds_away_from_zero_for_positive_and_negative_amounts() {
    let usd = currency("USD", 2);
    let positive = Money::from_minor_units(1_000, usd.clone());
    let negative = Money::from_minor_units(-1_000, usd);

    assert_eq!(
        positive
            .div_rounded(divisor(3), RoundingMode::AwayFromZero)
            .minor_units(),
        334
    );
    assert_eq!(
        negative
            .div_rounded(divisor(3), RoundingMode::AwayFromZero)
            .minor_units(),
        -334
    );
}

#[test]
fn floor_and_ceiling_follow_mathematical_direction() {
    let usd = currency("USD", 2);
    let positive = Money::from_minor_units(1_000, usd.clone());
    let negative = Money::from_minor_units(-1_000, usd);

    assert_eq!(
        positive
            .div_rounded(divisor(3), RoundingMode::Floor)
            .minor_units(),
        333
    );
    assert_eq!(
        positive
            .div_rounded(divisor(3), RoundingMode::Ceiling)
            .minor_units(),
        334
    );
    assert_eq!(
        negative
            .div_rounded(divisor(3), RoundingMode::Floor)
            .minor_units(),
        -334
    );
    assert_eq!(
        negative
            .div_rounded(divisor(3), RoundingMode::Ceiling)
            .minor_units(),
        -333
    );
}

#[test]
fn half_away_from_zero_uses_nearest_value_and_breaks_ties_away() {
    let usd = currency("USD", 2);
    let cases = [
        (7, 3, 2),
        (8, 3, 3),
        (5, 2, 3),
        (-7, 3, -2),
        (-8, 3, -3),
        (-5, 2, -3),
    ];

    for (minor_units, divisor_value, expected) in cases {
        let amount = Money::from_minor_units(minor_units, usd.clone());
        assert_eq!(
            amount
                .div_rounded(divisor(divisor_value), RoundingMode::HalfAwayFromZero)
                .minor_units(),
            expected
        );
    }
}

#[test]
fn half_even_breaks_exact_ties_toward_the_even_integer() {
    let usd = currency("USD", 2);
    let cases = [(5, 2), (7, 4), (9, 4), (-5, -2), (-7, -4), (-9, -4)];

    for (minor_units, expected) in cases {
        let amount = Money::from_minor_units(minor_units, usd.clone());
        assert_eq!(
            amount
                .div_rounded(divisor(2), RoundingMode::HalfEven)
                .minor_units(),
            expected
        );
    }
}

#[test]
fn half_even_uses_the_nearest_integer_when_result_is_not_tied() {
    let usd = currency("USD", 2);
    let cases = [(7, 2), (8, 3), (-7, -2), (-8, -3)];

    for (minor_units, expected) in cases {
        let amount = Money::from_minor_units(minor_units, usd.clone());
        assert_eq!(
            amount
                .div_rounded(divisor(3), RoundingMode::HalfEven)
                .minor_units(),
            expected
        );
    }
}

#[test]
fn rounded_division_preserves_currency_definition() {
    let amount = Money::from_minor_units(1_000, currency("XYZ", 4));

    let result = amount.div_rounded(divisor(3), RoundingMode::HalfEven);

    assert_eq!(result.currency(), amount.currency());
    assert_eq!(result.currency().code(), "XYZ");
    assert_eq!(result.currency().minor_units(), 4);
}

#[test]
fn rounded_division_handles_zero_and_full_i128_range() {
    let usd = currency("USD", 2);
    let zero = Money::from_minor_units(0, usd.clone());
    let minimum = Money::from_minor_units(i128::MIN, usd.clone());
    let maximum = Money::from_minor_units(i128::MAX, usd);

    assert_eq!(
        zero.div_rounded(divisor(u128::MAX), RoundingMode::AwayFromZero)
            .minor_units(),
        0
    );
    assert_eq!(
        minimum
            .div_rounded(divisor(1), RoundingMode::HalfEven)
            .minor_units(),
        i128::MIN
    );
    assert_eq!(
        maximum
            .div_rounded(divisor(1), RoundingMode::HalfEven)
            .minor_units(),
        i128::MAX
    );
}

#[test]
fn halfway_comparison_supports_u128_max_without_overflow() {
    let positive = Money::from_minor_units(i128::MAX, currency("USD", 2));
    let negative = Money::from_minor_units(i128::MIN, currency("USD", 2));

    assert_eq!(
        positive
            .div_rounded(divisor(u128::MAX), RoundingMode::HalfEven)
            .minor_units(),
        0
    );
    assert_eq!(
        negative
            .div_rounded(divisor(u128::MAX), RoundingMode::HalfEven)
            .minor_units(),
        -1
    );
}

#[test]
fn allocate_splits_even_positive_amounts() {
    let money = Money::from_minor_units(1_002, currency("USD", 2));

    let allocations = money.allocate(parts(3));

    assert_eq!(minor_units(&allocations), vec![334, 334, 334]);
}

#[test]
fn allocate_adds_positive_remainder_to_last_part() {
    let money = Money::from_minor_units(1_000, currency("USD", 2));

    let allocations = money.allocate(parts(3));

    assert_eq!(minor_units(&allocations), vec![333, 333, 334]);
}

#[test]
fn allocate_adds_larger_positive_remainder_to_last_part() {
    let money = Money::from_minor_units(1_001, currency("USD", 2));

    let allocations = money.allocate(parts(3));

    assert_eq!(minor_units(&allocations), vec![333, 333, 335]);
}

#[test]
fn allocate_adds_negative_remainder_to_last_part() {
    let money = Money::from_minor_units(-1_000, currency("USD", 2));

    let allocations = money.allocate(parts(3));

    assert_eq!(minor_units(&allocations), vec![-333, -333, -334]);
}

#[test]
fn allocate_preserves_exact_total() {
    let money = Money::from_minor_units(1_001, currency("USD", 2));

    let allocations = money.allocate(parts(3));
    let allocated_total: i128 = allocations.iter().map(Money::minor_units).sum();

    assert_eq!(allocated_total, money.minor_units());
}

#[test]
fn allocate_preserves_currency_definition() {
    let money = Money::from_minor_units(1_000, currency("XYZ", 4));

    let allocations = money.allocate(parts(3));

    assert!(
        allocations
            .iter()
            .all(|allocation| allocation.currency() == money.currency())
    );
}

#[test]
fn allocate_single_part_returns_original_money() {
    let money = Money::from_minor_units(-1_000, currency("USD", 2));

    let allocations = money.allocate(parts(1));

    assert_eq!(allocations, vec![money]);
}

#[test]
fn allocate_handles_zero_amount() {
    let money = Money::from_minor_units(0, currency("USD", 2));

    let allocations = money.allocate(parts(3));

    assert_eq!(minor_units(&allocations), vec![0, 0, 0]);
}

#[test]
fn allocate_handles_i128_min_without_overflow() {
    let money = Money::from_minor_units(i128::MIN, currency("USD", 2));

    let allocations = money.allocate(parts(1));

    assert_eq!(allocations, vec![money]);
}
