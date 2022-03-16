use std::{
    fmt,
    path::{Path, PathBuf},
};

pub type ParseResult<T> = std::result::Result<T, ParseError>;

#[derive(thiserror::Error, Debug)]
pub struct ParseError {
    message: String,
    path: Option<PathBuf>,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(path) = &self.path {
            let recipe_dir = path
                .parent()
                .and_then(|path| path.parent())
                .expect("invalid recipe path");
            write!(
                f,
                "invalid recipe format in file '{}': {}",
                path.strip_prefix(recipe_dir).unwrap().to_string_lossy(),
                self.message
            )
        } else {
            write!(f, "invalid recipe format: {}", self.message)
        }
    }
}

impl ParseError {
    pub fn empty(name: &str) -> Self {
        format!("{} must contain non-whitespace characters", name).into()
    }

    pub fn set_path(&mut self, path: &Path) {
        self.path = Some(path.into())
    }
}

impl From<&str> for ParseError {
    fn from(message: &str) -> Self {
        Self {
            message: message.into(),
            path: None,
        }
    }
}

impl From<String> for ParseError {
    fn from(message: String) -> Self {
        Self {
            message,
            path: None,
        }
    }
}
