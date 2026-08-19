use paykit_money::{Currency, Money, MoneyError, MoneyParseError};

fn currency(code: &str, minor_units: u8) -> Currency {
    Currency::new(code, minor_units)
        .unwrap_or_else(|error| panic!("expected a valid currency definition: {error}"))
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
