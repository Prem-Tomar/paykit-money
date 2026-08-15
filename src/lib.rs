struct Currency {
    code: String,
    minor_units: u8,
}

#[allow(unused)]
enum CurrencyError {
    CurrencyNotSupported(String),
    InvalidCode(String),
    InvalidUnit(String),
}

impl Currency {
    pub const MIN: u8 = 0;
    pub const MAX: u8 = 18;

    #[allow(unused)]
    pub fn new(code: &str, minor_units: u8) -> Result<Self, CurrencyError> {
        // Reject invalid code input
        if !Self::validate_code(code) {
            return Err(CurrencyError::InvalidCode(format!(
                "Provided code is not supported {}",
                code
            )));
        }
        // Reject invalid unit range
        if !Self::validate_units(minor_units) {
            return Err(CurrencyError::InvalidUnit(format!(
                "Provided values are not supported code: {}, units: {}",
                code, minor_units
            )));
        }
        // Returning the Currency after validations
        Ok(Currency {
            code: String::from(code),
            minor_units,
        })
    }
    #[allow(unused)]
    pub fn get_minor_units(&self) -> u8 {
        self.minor_units
    }
    #[allow(unused)]
    pub fn get_code(&self) -> &str {
        &self.code
    }

    fn validate_units(value: u8) -> bool {
        (Self::MIN..=Self::MAX).contains(&value)
    }

    fn validate_code(value: &str) -> bool {
        value.len() == 3 && value.chars().all(|c| c.is_ascii_uppercase())
    }
}
