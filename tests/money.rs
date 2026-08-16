use paykit_money::{Currency, Money, MoneyError};

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

    assert!(matches!(
        usd.checked_add(&eur),
        Err(MoneyError::CurrencyMismatchError(_))
    ));
}

#[test]
fn checked_sub_rejects_cross_currency_amounts() {
    let usd = Money::from_minor_units(1_000, currency("USD", 2));
    let eur = Money::from_minor_units(250, currency("EUR", 2));

    assert!(matches!(
        usd.checked_sub(&eur),
        Err(MoneyError::CurrencyMismatchError(_))
    ));
}

#[test]
fn checked_add_rejects_integer_overflow() {
    let usd = currency("USD", 2);
    let left = Money::from_minor_units(i128::MAX, usd.clone());
    let right = Money::from_minor_units(1, usd);

    assert!(matches!(
        left.checked_add(&right),
        Err(MoneyError::MoneyAddError(_))
    ));
}

#[test]
fn checked_sub_rejects_integer_overflow() {
    let usd = currency("USD", 2);
    let left = Money::from_minor_units(i128::MIN, usd.clone());
    let right = Money::from_minor_units(1, usd);

    assert!(matches!(
        left.checked_sub(&right),
        Err(MoneyError::MoneySubError(_))
    ));
}

#[test]
fn displays_currency_mismatch_error() {
    let error = MoneyError::CurrencyMismatchError("currencies do not match".into());

    assert_eq!(error.to_string(), "currencies do not match");
}

#[test]
fn displays_addition_overflow_error() {
    let error = MoneyError::MoneyAddError("money addition overflowed".into());

    assert_eq!(error.to_string(), "money addition overflowed");
}

#[test]
fn displays_subtraction_overflow_error() {
    let error = MoneyError::MoneySubError("money subtraction overflowed".into());

    assert_eq!(error.to_string(), "money subtraction overflowed");
}
