use paykit_money::{Rate, RateParseError};

#[test]
fn displays_rates_as_percentages_with_two_fractional_digits() {
    let cases = [
        (0, "0.00%"),
        (1, "0.01%"),
        (99, "0.99%"),
        (100, "1.00%"),
        (250, "2.50%"),
        (10_000, "100.00%"),
        (u32::MAX, "42949672.95%"),
    ];

    for (basis_points, expected) in cases {
        assert_eq!(Rate::from_basis_points(basis_points).to_string(), expected);
    }
}

#[test]
fn parses_whole_and_fractional_percentages_exactly() {
    let cases = [
        ("0%", 0),
        ("2%", 200),
        ("2.5%", 250),
        ("2.50%", 250),
        ("0.01%", 1),
        ("100.00%", 10_000),
    ];

    for (input, expected_basis_points) in cases {
        let rate = input.parse::<Rate>().expect("percentage should be valid");
        assert_eq!(rate.basis_points(), expected_basis_points);
    }
}

#[test]
fn parsing_accepts_surrounding_whitespace() {
    let rate = "  \t2.50%\n"
        .parse::<Rate>()
        .expect("whitespace should be valid");

    assert_eq!(rate, Rate::from_basis_points(250));
}

#[test]
fn parsing_accepts_the_maximum_basis_point_value() {
    let rate = "42949672.95%"
        .parse::<Rate>()
        .expect("u32::MAX basis points should be valid");

    assert_eq!(rate, Rate::from_basis_points(u32::MAX));
}

#[test]
fn display_and_parsing_round_trip_the_complete_boundary_set() {
    for basis_points in [0, 1, 99, 100, 250, 10_000, u32::MAX] {
        let original = Rate::from_basis_points(basis_points);
        let restored = original
            .to_string()
            .parse::<Rate>()
            .expect("displayed rate should parse");

        assert_eq!(restored, original);
    }
}

#[test]
fn rejects_empty_percentage_input() {
    assert_eq!("".parse::<Rate>(), Err(RateParseError::Empty));
    assert_eq!(" \t\n".parse::<Rate>(), Err(RateParseError::Empty));
}

#[test]
fn rejects_missing_percentage_symbol() {
    assert_eq!("2.50".parse::<Rate>(), Err(RateParseError::InvalidFormat));
}

#[test]
fn rejects_malformed_percentage_input() {
    for input in [
        "%",
        ".50%",
        "2.%",
        "2..50%",
        "+2.50%",
        "-2.50%",
        "2e2%",
        "2,50%",
        "2. 50%",
        "2.50 %%",
        "２.５０%",
    ] {
        assert_eq!(
            input.parse::<Rate>(),
            Err(RateParseError::InvalidFormat),
            "unexpected result for {input:?}"
        );
    }
}

#[test]
fn rejects_excess_fractional_precision_without_rounding() {
    assert_eq!(
        "2.501%".parse::<Rate>(),
        Err(RateParseError::TooManyFractionalDigits)
    );
}

#[test]
fn rejects_rate_above_u32_maximum() {
    assert_eq!(
        "42949672.96%".parse::<Rate>(),
        Err(RateParseError::RateOverflow)
    );
    assert_eq!(
        "999999999999999999999999999999999999999%".parse::<Rate>(),
        Err(RateParseError::RateOverflow)
    );
}

#[test]
fn rate_parse_errors_have_stable_messages() {
    assert_eq!(
        RateParseError::Empty.to_string(),
        "rate percentage is empty"
    );
    assert_eq!(
        RateParseError::InvalidFormat.to_string(),
        "rate percentage format is invalid"
    );
    assert_eq!(
        RateParseError::TooManyFractionalDigits.to_string(),
        "rate percentage has more than two fractional digits"
    );
    assert_eq!(
        RateParseError::RateOverflow.to_string(),
        "rate percentage exceeds u32 basis points"
    );
}

#[test]
fn rate_parse_error_implements_standard_error() {
    fn assert_error<T: std::error::Error>() {}

    assert_error::<RateParseError>();
}
