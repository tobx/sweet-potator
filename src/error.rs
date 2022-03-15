use std::{io, path::PathBuf};

use crate::recipe::errors::ParseError;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("a recipe title must contain non-whitespace characters")]
    EmptyRecipeTitle,
    #[error("invalid image file extension: '{0}'")]
    InvalidImageFileExt(PathBuf),
    #[error("invalid language file format: {0}")]
    InvalidLanguageFileFormat(#[from] toml::de::Error),
    #[error(transparent)]
    Io(#[from] io::Error),
    #[error("invalid recipe format: '{0}'")]
    Parse(#[from] ParseError),
    #[error("missing image file extension in path: '{0}'")]
    MissingImageFileExt(PathBuf),
    #[error("missing template file: '{0}'")]
    MissingTemplateFile(String),
    #[error(transparent)]
    Tera(#[from] tera::Error),
}
