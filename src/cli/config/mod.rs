use std::{
    collections::HashMap,
    path::{Path, PathBuf, StripPrefixError},
};

use config::{ConfigError, File, FileFormat};
use dirs_next::home_dir;
use serde::Deserialize;
use slug::slugify;
use sweet_potator::{
    generator::{self},
    util::sanitize_file_name,
};

pub const CONFIG_FILE_NAME: &str = "config.toml";

pub const DEFAULT_RECIPE_FILE_NAME: &str = "default.recipe";

pub const DEFAULT_CONFIG_FILE_CONTENT: &str = include_str!("default.toml");

pub const DEFAULT_LANGUAGE: fn() -> String = || "en".into();

pub const DEFAULT_RECIPE_FILE_CONTENT: &str = include_str!("default.recipe");

const TEMPLATE_DIR: &str = "templates";

const TRUE: fn() -> bool = || true;

#[derive(Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FileNameFilter {
    Sanitize,
    Slugify,
}

impl Default for FileNameFilter {
    fn default() -> Self {
        Self::Sanitize
    }
}

impl generator::TextFilter for FileNameFilter {
    fn filter<S: AsRef<str>>(&self, text: S) -> String {
        match self {
            Self::Sanitize => sanitize_file_name(text.as_ref().trim()),
            Self::Slugify => slugify(text),
        }
    }
}

#[derive(Deserialize)]
pub struct GeneratorOptions {
    #[serde(default = "TRUE")]
    pub escape: bool,
    pub extension: String,
    #[serde(default)]
    pub file_name_filter: FileNameFilter,
    #[serde(default = "DEFAULT_LANGUAGE")]
    pub language: String,
}

#[derive(Deserialize)]
pub struct Config {
    #[serde(skip)]
    pub dir: PathBuf,
    pub recipe_dir: PathBuf,
    pub editor: Vec<String>,
    #[serde(rename = "image_file_extensions")]
    pub image_file_exts: Vec<String>,
    pub templates: HashMap<String, GeneratorOptions>,
}

impl Config {
    pub fn load(path: &Path) -> std::result::Result<Self, ConfigError> {
        let config = config::Config::builder()
            .add_source(File::from_str(
                DEFAULT_CONFIG_FILE_CONTENT,
                FileFormat::Toml,
            ))
            .add_source(File::from(path).format(FileFormat::Toml))
            .build()?;
        let mut config: Config = config.try_deserialize()?;
        config.dir = path.parent().unwrap().into();
        let mut recipe_dir = config.recipe_dir;
        if let Ok(path) = expand_tilde(&recipe_dir) {
            recipe_dir = path;
        } else if !recipe_dir.is_absolute() {
            if let Ok(path) = recipe_dir.strip_prefix("./") {
                recipe_dir = path.into();
            }
            recipe_dir = path.parent().unwrap().join(recipe_dir);
        }
        config.recipe_dir = recipe_dir;
        Ok(config)
    }

    pub fn default_recipe_path(&self) -> PathBuf {
        self.dir.join(DEFAULT_RECIPE_FILE_NAME)
    }

    pub fn template_dir(&self) -> PathBuf {
        self.dir.join(TEMPLATE_DIR)
    }
}

fn expand_tilde(path: &Path) -> std::result::Result<PathBuf, StripPrefixError> {
    path.strip_prefix("~").map(|path| {
        home_dir()
            .expect("cannot retrieve home directory")
            .join(path)
    })
}
