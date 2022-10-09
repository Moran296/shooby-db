#[derive(Debug, Clone)]
pub enum ShoobyError {
    Unknown,
    OutOfBounds,
    InvalidTypeConversion,
    InvalidSize,
    InvalidType,
}
