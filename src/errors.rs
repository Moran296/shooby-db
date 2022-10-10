use std::fmt::{Display, Formatter, Result as FmtResult};

#[derive(Debug, Clone)]
pub enum ShoobyError {
    Unknown,
    OutOfBounds,
    InvalidTypeConversion,
    InvalidSize,
    InvalidType,
}

impl Display for ShoobyError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            ShoobyError::Unknown => write!(f, "Unknown error"),
            ShoobyError::OutOfBounds => write!(f, "Out of bounds"),
            ShoobyError::InvalidTypeConversion => write!(f, "Invalid type conversion"),
            ShoobyError::InvalidSize => write!(f, "Invalid size"),
            ShoobyError::InvalidType => write!(f, "Invalid type"),
        }
    }
}

impl std::error::Error for ShoobyError {}
