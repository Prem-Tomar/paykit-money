use paykit_money::{Currency, Money};

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
