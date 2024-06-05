use std::{
    fs, io,
    path::{Path, PathBuf},
};

use tera::Tera;
use toml::Value;

use crate::error::{Error, Result};

pub type Context = tera::Context;

pub const LANGUAGE_DIR: &str = "lang";
pub const LANGUAGE_FILE_EXTENSION: &str = "toml";
pub const STATIC_DIR: &str = "static";
pub const TERA_DIR: &str = "tera";

pub const INDEX_NAME: &str = "index";
pub const RECIPE_NAME: &str = "recipe";

pub struct Engine {
    tera: Tera,
    file_ext: String,
    language_path: PathBuf,
    static_path: PathBuf,
    pub language: Option<String>,
    pub forced_context: Option<Context>,
}

impl Engine {
    /// # Panics
    ///
    /// Will panic if `PathBuf::to_str` returns `None`
    pub fn new<E: Into<String>>(
        path: &Path,
        escape: bool,
        file_ext: E,
        language: Option<&str>,
    ) -> Result<Self> {
        let file_ext = file_ext.into();
        let mut glob_path = path.join(TERA_DIR).join("**/*");
        glob_path.set_extension(&file_ext);
        let mut tera = Tera::new(glob_path.to_str().expect("invalid template path"))?;
        if escape {
            tera.autoescape_on(vec![""]);
        } else {
            tera.autoescape_on(Vec::new());
        }
        let engine = Self {
            tera,
            file_ext,
            language_path: path.join(LANGUAGE_DIR),
            static_path: path.join(STATIC_DIR),
            language: language.map(Into::into),
            forced_context: None,
        };
        if !engine.has_template(RECIPE_NAME) {
            return Err(Error::MissingTemplateFile(
                engine.template_path(RECIPE_NAME),
            ));
        }
        Ok(engine)
    }

    pub(crate) fn has_index_template(&self) -> bool {
        self.has_template(INDEX_NAME)
    }

    pub(crate) fn render_index(&self, context: Context, writer: impl io::Write) -> Result<()> {
        self.render(INDEX_NAME, context, writer)
    }

    pub(crate) fn render_recipe(&self, context: Context, writer: impl io::Write) -> Result<()> {
        self.render(RECIPE_NAME, context, writer)
    }

    pub(crate) fn static_path(&self) -> &Path {
        &self.static_path
    }

    fn has_template(&self, name: &str) -> bool {
        let path = self.template_path(name);
        self.tera.get_template_names().any(|name| name == path)
    }

    fn load_language_file(&self) -> Result<Option<Value>> {
        if let Some(language) = &self.language {
            let path = self
                .language_path
                .join(language)
                .with_extension(LANGUAGE_FILE_EXTENSION);
            match fs::read_to_string(path) {
                Ok(data) => Ok(Some(data.parse()?)),
                Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
                Err(error) => Err(error.into()),
            }
        } else {
            Ok(None)
        }
    }

    fn render(
        &self,
        template_name: &str,
        mut context: Context,
        writer: impl io::Write,
    ) -> Result<()> {
        if let Some(data) = self.load_language_file()? {
            context.insert("lang", &data);
        }
        if let Some(fc) = &self.forced_context {
            context.extend(fc.clone());
        }
        Ok(self
            .tera
            .render_to(&self.template_path(template_name), &context, writer)?)
    }

    fn template_path(&self, template_name: &str) -> String {
        format!("{}.{}", template_name, self.file_ext)
    }
}
