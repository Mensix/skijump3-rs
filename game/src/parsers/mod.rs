use std::fmt;

pub mod anim;
pub mod langbase;
pub mod pcx;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    pub message: String,
    pub byte_offset: Option<usize>,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.byte_offset {
            Some(offset) => write!(f, "{} at byte {}", self.message, offset),
            None => write!(f, "{}", self.message),
        }
    }
}

impl std::error::Error for ParseError {}

pub trait AssetParser<T> {
    fn parse(data: &[u8]) -> Result<T, ParseError>;
    fn validate(data: &[u8]) -> bool;
}