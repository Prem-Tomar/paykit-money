// Keep Serde completely optional for consumers that do not enable the `serde` feature.
#[cfg(feature = "serde")]
use serde::Serialize;
use std::error::Error;
use std::fmt;

/// A validated currency definition.
///
/// A currency consists of a three-letter uppercase ASCII code and the number of digits
/// used for its minor units. The type is intentionally data-driven: applications can
/// introduce new currency definitions without changing this crate.
#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Currency {
    code: String,
    minor_units: u8,
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for Currency {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        // First collect external data without claiming that it is a valid Currency.
        #[derive(serde::Deserialize)]
        struct CurrencyWire {
            code: String,
            minor_units: u8,
        }

        // Let the selected data format validate field names and primitive field types.
        let wire = CurrencyWire::deserialize(deserializer)?;

        // Route construction through the public validator so deserialization cannot bypass
        // currency-code or minor-unit-scale invariants.
        Currency::new(wire.code, wire.minor_units).map_err(serde::de::Error::custom)
    }
}

/// An error returned when a currency definition violates its structural invariants.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CurrencyError {
    /// The code is not exactly three uppercase ASCII letters.
    InvalidCode(String),
    /// The minor-unit scale exceeds [`Currency::MAX_MINOR_UNITS`].
    InvalidMinorUnits { code: String, minor_units: u8 },
}

impl Currency {
    /// The minimum supported minor-unit scale.
    pub const MIN_MINOR_UNITS: u8 = 0;

    /// The maximum supported minor-unit scale.
    pub const MAX_MINOR_UNITS: u8 = 18;

    /// Creates a validated currency definition.
    ///
    /// The code must contain exactly three uppercase ASCII letters. The minor-unit scale
    /// must be between [`Self::MIN_MINOR_UNITS`] and [`Self::MAX_MINOR_UNITS`], inclusive.
    pub fn new(code: impl Into<String>, minor_units: u8) -> Result<Self, CurrencyError> {
        let code = code.into();

        if !Self::is_valid_code(&code) {
            return Err(CurrencyError::InvalidCode(code));
        }

        if !Self::is_valid_minor_units(minor_units) {
            return Err(CurrencyError::InvalidMinorUnits { code, minor_units });
        }

        Ok(Self { code, minor_units })
    }

    /// Returns the three-letter currency code.
    #[must_use]
    pub fn code(&self) -> &str {
        &self.code
    }

    /// Returns the number of digits used for the currency's minor units.
    #[must_use]
    pub const fn minor_units(&self) -> u8 {
        self.minor_units
    }

    fn is_valid_minor_units(value: u8) -> bool {
        (Self::MIN_MINOR_UNITS..=Self::MAX_MINOR_UNITS).contains(&value)
    }

    fn is_valid_code(value: &str) -> bool {
        value.len() == 3 && value.bytes().all(|byte| byte.is_ascii_uppercase())
    }
}

impl fmt::Display for Currency {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.code)
    }
}

impl fmt::Display for CurrencyError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidCode(code) => {
                write!(
                    formatter,
                    "invalid currency code `{code}`: expected three uppercase ASCII letters"
                )
            }
            Self::InvalidMinorUnits { code, minor_units } => write!(
                formatter,
                "invalid minor-unit scale {minor_units} for currency `{code}`: expected {}..={}",
                Currency::MIN_MINOR_UNITS,
                Currency::MAX_MINOR_UNITS
            ),
        }
    }
}

impl Error for CurrencyError {}
