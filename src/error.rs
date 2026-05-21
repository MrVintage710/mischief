use nom::error::Error;
use thiserror::Error;

//==============================================================================================
//        Mischief Error
//==============================================================================================

pub type MischiefResult<T> = Result<T, MischiefError>;
pub type MischiefException = Result<(), MischiefError>;

#[derive(Debug, Error)]
pub enum MischiefError {
    #[error("Error while reading unit value: `{0}`")]
    UnitParseError(String),

    #[error("There was an error while parsing tailwind: `{0}`")]
    TailwindParsingError(String),

    #[error("Error: {0}")]
    ParseStringError(#[from] std::num::ParseFloatError),

    #[error("Regex Error: `{0}`")]
    RegexError(#[from] regex::Error),

    #[error("No value returned.")]
    NoValue
}