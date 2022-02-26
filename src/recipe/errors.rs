pub type ParseResult<T> = std::result::Result<T, ParseError>;

#[derive(thiserror::Error, Debug)]
#[error("{0}")]
pub struct ParseError(String);

impl ParseError {
    pub fn empty(name: &str) -> Self {
        format!("{} must contain non-whitespace characters", name).into()
    }
}

impl From<&str> for ParseError {
    fn from(message: &str) -> Self {
        Self(message.into())
    }
}

impl From<String> for ParseError {
    fn from(message: String) -> Self {
        Self(message)
    }
}
