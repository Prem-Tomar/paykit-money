use paykit_money::{Currency, CurrencyError};

fn expect_valid_currency(code: &str, minor_units: u8) -> Currency {
    Currency::new(code, minor_units)
        .unwrap_or_else(|error| panic!("expected a valid currency definition: {error}"))
}

#[test]
fn creates_currency_from_valid_definition() {
    let currency = expect_valid_currency("USD", 2);

    assert_eq!(currency.code(), "USD");
    assert_eq!(currency.minor_units(), 2);
}

#[test]
fn accepts_zero_minor_units() {
    let currency = expect_valid_currency("JPY", 0);

    assert_eq!(currency.code(), "JPY");
    assert_eq!(currency.minor_units(), 0);
}

#[test]
fn accepts_currency_not_built_into_the_crate() {
    let currency = expect_valid_currency("XYZ", 4);

    assert_eq!(currency.code(), "XYZ");
    assert_eq!(currency.minor_units(), 4);
}

#[test]
fn accepts_owned_currency_code() {
    let code = String::from("EUR");
    let currency = Currency::new(code, 2).expect("EUR should be valid");

    assert_eq!(currency.code(), "EUR");
}

#[test]
fn rejects_lowercase_code() {
    let result = Currency::new("usd", 2);

    assert_eq!(result, Err(CurrencyError::InvalidCode("usd".into())));
}

#[test]
fn rejects_code_with_invalid_length() {
    let result = Currency::new("US", 2);

    assert_eq!(result, Err(CurrencyError::InvalidCode("US".into())));
}

#[test]
fn rejects_non_alphabetic_code() {
    let result = Currency::new("US1", 2);

    assert_eq!(result, Err(CurrencyError::InvalidCode("US1".into())));
}

#[test]
fn rejects_non_ascii_code() {
    let result = Currency::new("ÜSD", 2);

    assert_eq!(result, Err(CurrencyError::InvalidCode("ÜSD".into())));
}

#[test]
fn accepts_maximum_minor_unit_scale() {
    let currency = expect_valid_currency("MAX", Currency::MAX_MINOR_UNITS);

    assert_eq!(currency.minor_units(), Currency::MAX_MINOR_UNITS);
}

#[test]
fn rejects_minor_unit_scale_above_maximum() {
    let minor_units = Currency::MAX_MINOR_UNITS + 1;
    let result = Currency::new("USD", minor_units);

    assert_eq!(
        result,
        Err(CurrencyError::InvalidMinorUnits {
            code: "USD".into(),
            minor_units,
        })
    );
}

#[test]
fn displays_currency_code() {
    let currency = expect_valid_currency("INR", 2);

    assert_eq!(currency.to_string(), "INR");
}

#[test]
fn error_messages_include_invalid_input() {
    let error = Currency::new("usd", 2).expect_err("lowercase code should be rejected");

    assert!(error.to_string().contains("usd"));
}
