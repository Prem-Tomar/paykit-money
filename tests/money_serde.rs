#![cfg(feature = "serde")]

use paykit_money::{Currency, Money};

fn currency(code: &str, minor_units: u8) -> Currency {
    Currency::new(code, minor_units).expect("test currency should be valid")
}

#[test]
fn serializes_minor_units_as_a_string_in_the_public_wire_shape() {
    let money = Money::from_minor_units(1_050, currency("USD", 2));

    let json = serde_json::to_string(&money).expect("valid money should serialize");

    assert_eq!(
        json,
        r#"{"minor_units":"1050","currency":{"code":"USD","minor_units":2}}"#
    );
}

#[test]
fn deserializes_valid_money_from_a_string_amount() {
    let json = r#"{"minor_units":"-1050","currency":{"code":"USD","minor_units":2}}"#;

    let money: Money = serde_json::from_str(json).expect("valid money should deserialize");

    assert_eq!(money.minor_units(), -1_050);
    assert_eq!(money.currency(), &currency("USD", 2));
}

#[test]
fn round_trip_preserves_zero_and_full_i128_range() {
    let usd = currency("USD", 2);

    for minor_units in [0, i128::MIN, i128::MAX] {
        let original = Money::from_minor_units(minor_units, usd.clone());
        let json = serde_json::to_string(&original).expect("money should serialize");
        let restored: Money =
            serde_json::from_str(&json).expect("serialized money should deserialize");

        assert_eq!(restored, original);
    }
}

#[test]
fn rejects_numeric_minor_units() {
    let json = r#"{"minor_units":1050,"currency":{"code":"USD","minor_units":2}}"#;

    let error = serde_json::from_str::<Money>(json)
        .expect_err("minor_units must use the documented string representation");

    assert!(error.to_string().contains("invalid type"));
}

#[test]
fn rejects_floating_point_minor_units() {
    let json = r#"{"minor_units":10.5,"currency":{"code":"USD","minor_units":2}}"#;

    let error = serde_json::from_str::<Money>(json)
        .expect_err("floating-point minor_units must not be accepted");

    assert!(error.to_string().contains("invalid type"));
}

#[test]
fn rejects_malformed_minor_unit_string() {
    let json = r#"{"minor_units":"10.50","currency":{"code":"USD","minor_units":2}}"#;

    let error = serde_json::from_str::<Money>(json)
        .expect_err("minor_units must contain a signed base-10 integer");

    assert!(error.to_string().contains("invalid digit"));
}

#[test]
fn rejects_minor_units_above_i128_maximum() {
    let json = r#"{"minor_units":"170141183460469231731687303715884105728","currency":{"code":"USD","minor_units":2}}"#;

    let error = serde_json::from_str::<Money>(json)
        .expect_err("minor_units above i128::MAX must be rejected");

    assert!(error.to_string().contains("number too large"));
}

#[test]
fn rejects_minor_units_below_i128_minimum() {
    let json = r#"{"minor_units":"-170141183460469231731687303715884105729","currency":{"code":"USD","minor_units":2}}"#;

    let error = serde_json::from_str::<Money>(json)
        .expect_err("minor_units below i128::MIN must be rejected");

    assert!(error.to_string().contains("number too small"));
}

#[test]
fn rejects_money_with_invalid_nested_currency() {
    let json = r#"{"minor_units":"1050","currency":{"code":"usd","minor_units":2}}"#;

    let error = serde_json::from_str::<Money>(json)
        .expect_err("nested Currency invariants must remain enforced");

    assert!(error.to_string().contains("three uppercase ASCII letters"));
}

#[test]
fn rejects_missing_money_fields() {
    let json = r#"{"minor_units":"1050"}"#;

    let error = serde_json::from_str::<Money>(json).expect_err("currency must be present");

    assert!(error.to_string().contains("missing field"));
}
