use std::io;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Config(#[from] config::ConfigError),
    #[error("recipe list is corrupt")]
    CorruptedRecipeList,
    #[error("editor command '{0}' not found")]
    EditorCommandNotFound(String),
    #[error("io error: {0}")]
    Io(#[from] io::Error),
    #[error("output directory '{0}' already exists")]
    OutputDirectoryAlreadyExists(String),
    #[error("recipe directory '{0}' not found")]
    RecipeDirNotFound(String),
    #[error("recipe file '{0}' not found")]
    RecipeFileNotFound(String),
    #[error(transparent)]
    SweetPotator(sweet_potator::error::Error),
    #[error("template name '{0}' not configured")]
    TemplateNameNotConfigured(String),
    #[error("template name '{0}' not found")]
    TemplateNameNotFound(String),
    #[error(transparent)]
    Tera(#[from] tera::Error),
}

impl From<sweet_potator::error::Error> for Error {
    fn from(error: sweet_potator::error::Error) -> Self {
        if let sweet_potator::error::Error::Io(error) = error {
            error.into()
        } else {
            Self::SweetPotator(error)
        }
    }
}
