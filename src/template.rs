use std::{
    io,
    path::{Path, PathBuf},
};

use serde::Serialize;
use tera::Tera;

use crate::error::{Error, Result};

pub type Context = tera::Context;

pub const STATIC_DIR: &str = "static";
pub const TERA_DIR: &str = "tera";

pub const INDEX_NAME: &str = "index";
pub const RECIPE_NAME: &str = "recipe";

#[derive(Debug, Serialize)]
struct IndexEntry {
    pub title: String,
    pub path: PathBuf,
    pub tags: Vec<String>,
    pub image_path: Option<PathBuf>,
}

pub struct Engine {
    tera: Tera,
    file_ext: String,
    static_path: PathBuf,
    pub forced_context: Option<Context>,
}

impl Engine {
    pub fn new<E: Into<String>>(path: &Path, escape: bool, file_ext: E) -> Result<Self> {
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
            static_path: path.join(STATIC_DIR),
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

    fn render(
        &self,
        template_name: &str,
        mut context: Context,
        writer: impl io::Write,
    ) -> Result<()> {
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
