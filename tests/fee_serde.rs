#![cfg(feature = "serde")]

use paykit_money::{Currency, FeeFormula, FeeSchedule, Money, Rate};

fn currency(code: &str, minor_units: u8) -> Currency {
    Currency::new(code, minor_units).expect("test currency should be valid")
}

fn money(minor_units: i128, code: &str, scale: u8) -> Money {
    Money::from_minor_units(minor_units, currency(code, scale))
}

#[test]
fn serializes_rate_using_public_wire_shape() {
    let rate = Rate::from_basis_points(250);

    let json = serde_json::to_string(&rate).expect("rate should serialize");

    assert_eq!(json, r#"{"basis_points":250}"#);
}

#[test]
fn deserializes_rate_through_constructor() {
    let json = r#"{"basis_points":250}"#;

    let rate: Rate = serde_json::from_str(json).expect("rate should deserialize");

    assert_eq!(rate, Rate::from_basis_points(250));
}

#[test]
fn rejects_rate_with_wrong_field_type() {
    let json = r#"{"basis_points":"250"}"#;

    let error =
        serde_json::from_str::<Rate>(json).expect_err("basis_points must be an unsigned integer");

    assert!(error.to_string().contains("invalid type"));
}

#[test]
fn serializes_fee_formula_using_public_wire_shape() {
    let formula = FeeFormula::new(Rate::from_basis_points(250), money(30, "USD", 2));

    let json = serde_json::to_string(&formula).expect("fee formula should serialize");

    assert_eq!(
        json,
        r#"{"rate":{"basis_points":250},"fixed":{"minor_units":"30","currency":{"code":"USD","minor_units":2}}}"#
    );
}

#[test]
fn deserializes_fee_formula_through_constructor() {
    let json = r#"{"rate":{"basis_points":250},"fixed":{"minor_units":"30","currency":{"code":"USD","minor_units":2}}}"#;

    let formula: FeeFormula = serde_json::from_str(json).expect("fee formula should deserialize");

    assert_eq!(formula.rate(), Rate::from_basis_points(250));
    assert_eq!(formula.fixed(), &money(30, "USD", 2));
}

#[test]
fn rejects_fee_formula_with_invalid_fixed_money() {
    let json = r#"{"rate":{"basis_points":250},"fixed":{"minor_units":"30","currency":{"code":"usd","minor_units":2}}}"#;

    let error = serde_json::from_str::<FeeFormula>(json)
        .expect_err("nested fixed money invariants must remain enforced");

    assert!(error.to_string().contains("three uppercase ASCII letters"));
}

#[test]
fn rejects_fee_formula_with_missing_fields() {
    let json = r#"{"rate":{"basis_points":250}}"#;

    let error = serde_json::from_str::<FeeFormula>(json).expect_err("fixed fee must be present");

    assert!(error.to_string().contains("missing field"));
}

#[test]
fn serializes_fee_schedule_using_public_wire_shape() {
    let formula = FeeFormula::new(Rate::from_basis_points(250), money(30, "USD", 2));
    let schedule = FeeSchedule::new(
        formula,
        Some(money(50, "USD", 2)),
        Some(money(2_000, "USD", 2)),
    )
    .expect("valid schedule should construct");

    let json = serde_json::to_string(&schedule).expect("fee schedule should serialize");

    assert_eq!(
        json,
        r#"{"formula":{"rate":{"basis_points":250},"fixed":{"minor_units":"30","currency":{"code":"USD","minor_units":2}}},"minimum":{"minor_units":"50","currency":{"code":"USD","minor_units":2}},"maximum":{"minor_units":"2000","currency":{"code":"USD","minor_units":2}}}"#
    );
}

#[test]
fn serializes_fee_schedule_without_caps_as_null_caps() {
    let formula = FeeFormula::new(Rate::from_basis_points(250), money(30, "USD", 2));
    let schedule = FeeSchedule::new(formula, None, None).expect("valid schedule should construct");

    let json = serde_json::to_string(&schedule).expect("fee schedule should serialize");

    assert_eq!(
        json,
        r#"{"formula":{"rate":{"basis_points":250},"fixed":{"minor_units":"30","currency":{"code":"USD","minor_units":2}}},"minimum":null,"maximum":null}"#
    );
}

#[test]
fn deserializes_fee_schedule_through_constructor() {
    let json = r#"{"formula":{"rate":{"basis_points":250},"fixed":{"minor_units":"30","currency":{"code":"USD","minor_units":2}}},"minimum":{"minor_units":"50","currency":{"code":"USD","minor_units":2}},"maximum":{"minor_units":"2000","currency":{"code":"USD","minor_units":2}}}"#;

    let schedule: FeeSchedule =
        serde_json::from_str(json).expect("fee schedule should deserialize");

    assert_eq!(schedule.formula().rate(), Rate::from_basis_points(250));
    assert_eq!(schedule.formula().fixed(), &money(30, "USD", 2));
    assert_eq!(schedule.minimum(), Some(&money(50, "USD", 2)));
    assert_eq!(schedule.maximum(), Some(&money(2_000, "USD", 2)));
}

#[test]
fn deserializes_fee_schedule_with_null_caps() {
    let json = r#"{"formula":{"rate":{"basis_points":250},"fixed":{"minor_units":"30","currency":{"code":"USD","minor_units":2}}},"minimum":null,"maximum":null}"#;

    let schedule: FeeSchedule =
        serde_json::from_str(json).expect("fee schedule should deserialize");

    assert_eq!(schedule.minimum(), None);
    assert_eq!(schedule.maximum(), None);
}

#[test]
fn rejects_fee_schedule_with_cap_currency_mismatch() {
    let json = r#"{"formula":{"rate":{"basis_points":250},"fixed":{"minor_units":"30","currency":{"code":"USD","minor_units":2}}},"minimum":{"minor_units":"50","currency":{"code":"EUR","minor_units":2}},"maximum":null}"#;

    let error = serde_json::from_str::<FeeSchedule>(json)
        .expect_err("cap currency mismatch must remain invalid");

    assert!(error.to_string().contains("currency mismatch"));
}

#[test]
fn rejects_fee_schedule_with_invalid_cap_range() {
    let json = r#"{"formula":{"rate":{"basis_points":250},"fixed":{"minor_units":"30","currency":{"code":"USD","minor_units":2}}},"minimum":{"minor_units":"2001","currency":{"code":"USD","minor_units":2}},"maximum":{"minor_units":"2000","currency":{"code":"USD","minor_units":2}}}"#;

    let error = serde_json::from_str::<FeeSchedule>(json)
        .expect_err("minimum above maximum must remain invalid");

    assert!(error.to_string().contains("cap range is invalid"));
}

#[test]
fn rejects_fee_schedule_with_invalid_nested_formula() {
    let json = r#"{"formula":{"rate":{"basis_points":250},"fixed":{"minor_units":"30","currency":{"code":"usd","minor_units":2}}},"minimum":null,"maximum":null}"#;

    let error = serde_json::from_str::<FeeSchedule>(json)
        .expect_err("nested formula invariants must remain enforced");

    assert!(error.to_string().contains("three uppercase ASCII letters"));
}

#[test]
fn round_trip_preserves_fee_schedule_definition() {
    let formula = FeeFormula::new(Rate::from_basis_points(250), money(30, "USD", 2));
    let original = FeeSchedule::new(
        formula,
        Some(money(50, "USD", 2)),
        Some(money(2_000, "USD", 2)),
    )
    .expect("valid schedule should construct");

    let json = serde_json::to_string(&original).expect("fee schedule should serialize");
    let restored: FeeSchedule =
        serde_json::from_str(&json).expect("serialized fee schedule should deserialize");

    assert_eq!(restored, original);
}
