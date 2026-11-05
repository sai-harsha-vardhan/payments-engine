use std::fmt;

/// A row-level failure: logged to stderr, the row is skipped
#[derive(Debug, PartialEq)]
pub enum DomainError {
    MissingAmount,
    InvalidAmount,
    DuplicateTransaction,
    UnknownTransactionId,
    ClientMismatch,
    NotDisputable,
    NotUnderDispute,
    InsufficientBalance,
    AccountLocked,
    Overflow,
}

impl fmt::Display for DomainError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::MissingAmount => "missing amount",
            Self::InvalidAmount => "invalid amount",
            Self::DuplicateTransaction => "duplicate transaction id",
            Self::UnknownTransactionId => "unknown transaction id",
            Self::ClientMismatch => "transaction belongs to another client",
            Self::NotDisputable => "transaction cannot be disputed",
            Self::NotUnderDispute => "transaction is not under dispute",
            Self::InsufficientBalance => "insufficient available funds",
            Self::AccountLocked => "account is locked",
            Self::Overflow => "arithmetic overflow",
        })
    }
}

impl std::error::Error for DomainError {}

/// A fatal failure: printed once, process exits non-zero.
#[derive(Debug)]
pub enum AppError {
    MissingArgument,
    MissingColumn(&'static str),
    Io(std::io::Error),
    Csv(csv::Error),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingArgument => f.write_str(concat!(
                "usage: ",
                env!("CARGO_PKG_NAME"),
                " <transactions.csv>"
            )),
            Self::MissingColumn(name) => write!(f, "input is missing the `{name}` column"),
            Self::Io(error) => write!(f, "i/o error: {error}"),
            Self::Csv(error) => write!(f, "csv error: {error}"),
        }
    }
}

impl std::error::Error for AppError {}

impl From<std::io::Error> for AppError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<csv::Error> for AppError {
    fn from(error: csv::Error) -> Self {
        Self::Csv(error)
    }
}
