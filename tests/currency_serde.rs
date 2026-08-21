#![cfg(feature = "serde")]

use paykit_money::Currency;

#[test]
fn serializes_currency_using_the_public_wire_shape() {
    let currency = Currency::new("USD", 2).expect("USD should be valid");

    let json = serde_json::to_string(&currency).expect("valid currency should serialize");

    assert_eq!(json, r#"{"code":"USD","minor_units":2}"#);
}

#[test]
fn deserializes_valid_currency_through_its_invariants() {
    let json = r#"{"code":"INR","minor_units":2}"#;

    let currency: Currency =
        serde_json::from_str(json).expect("valid currency data should deserialize");

    assert_eq!(currency.code(), "INR");
    assert_eq!(currency.minor_units(), 2);
}

#[test]
fn round_trip_preserves_custom_currency_definition() {
    let original = Currency::new("XYZ", 4).expect("custom currency should be valid");

    let json = serde_json::to_string(&original).expect("currency should serialize");
    let restored: Currency =
        serde_json::from_str(&json).expect("serialized currency should deserialize");

    assert_eq!(restored, original);
}

#[test]
fn rejects_invalid_currency_code_during_deserialization() {
    let json = r#"{"code":"usd","minor_units":2}"#;

    let error = serde_json::from_str::<Currency>(json)
        .expect_err("lowercase currency code must remain invalid");

    assert!(error.to_string().contains("three uppercase ASCII letters"));
}

#[test]
fn rejects_invalid_minor_unit_scale_during_deserialization() {
    let json = r#"{"code":"USD","minor_units":19}"#;

    let error = serde_json::from_str::<Currency>(json)
        .expect_err("minor-unit scale above the maximum must remain invalid");

    assert!(error.to_string().contains("expected 0..=18"));
}

#[test]
fn rejects_missing_currency_fields() {
    let json = r#"{"code":"USD"}"#;

    let error = serde_json::from_str::<Currency>(json).expect_err("minor_units must be present");

    assert!(error.to_string().contains("missing field"));
}

#[test]
fn rejects_wrong_currency_field_types() {
    let json = r#"{"code":"USD","minor_units":"2"}"#;

    let error = serde_json::from_str::<Currency>(json)
        .expect_err("minor_units must be an unsigned integer");

    assert!(error.to_string().contains("invalid type"));
}
