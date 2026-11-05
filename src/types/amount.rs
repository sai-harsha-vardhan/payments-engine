use super::errors::DomainError;
use rust_decimal::Decimal;
use serde::Deserialize;
use std::fmt;
use std::str::FromStr;

const MAX_SCALE: u32 = 4;

#[derive(Debug, Default, Clone, Copy, PartialEq, PartialOrd, Deserialize)]
#[serde(try_from = "String")]
pub struct Amount(Decimal);

impl Amount {
    pub fn checked_add(self, rhs: Self) -> Option<Self> {
        self.0.checked_add(rhs.0).map(Self)
    }

    pub fn checked_sub(self, rhs: Self) -> Option<Self> {
        self.0.checked_sub(rhs.0).map(Self)
    }

    pub fn is_positive(self) -> bool {
        self.0.is_sign_positive() && !self.0.is_zero()
    }
}

impl FromStr for Amount {
    type Err = DomainError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Decimal::from_str also accepts '+', '_' separators and exponents, which are invalid in this input format.
        let unsigned = s.strip_prefix('-').unwrap_or(s);
        if unsigned.is_empty() || !unsigned.bytes().all(|b| b.is_ascii_digit() || b == b'.') {
            return Err(DomainError::InvalidAmount);
        }
        let value = Decimal::from_str(s).map_err(|_| DomainError::InvalidAmount)?;
        if value.scale() > MAX_SCALE {
            return Err(DomainError::InvalidAmount);
        }
        Ok(Self(value))
    }
}

impl TryFrom<String> for Amount {
    type Error = DomainError;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        s.parse()
    }
}

impl fmt::Display for Amount {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const MAX: &str = "79228162514264337593543950335";

    #[test]
    fn parse_invalid() {
        let cases = [
            "",
            "-",
            ".",
            "abc",
            " 1",
            "+1",
            "1_000",
            "1e2",
            "1.2.3",
            "0.00001",
            "1.00000",
            "79228162514264337593543950336",
        ];
        for input in cases {
            assert_eq!(
                input.parse::<Amount>(),
                Err(DomainError::InvalidAmount),
                "input {input:?}"
            );
        }
    }

    #[test]
    fn display_and_overflow_at_the_maximum() {
        let max: Amount = MAX.parse().unwrap();
        assert_eq!(max.to_string(), MAX);
        assert_eq!(max.checked_add("1".parse().unwrap()), None);
    }
}
