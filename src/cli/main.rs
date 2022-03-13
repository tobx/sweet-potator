#![forbid(unsafe_code)]

mod commands;
mod config;
mod editor;
mod error;
mod options;
mod terminal;
mod util;

use std::fs;

use clap::Parser;
use dirs_next::home_dir;

use serde::Serialize;
use sweet_potator::{APP_NAME, TEMPLATE_DIR};

use crate::{
    config::{Config, CONFIG_FILE_NAME, DEFAULT_CONFIG_FILE_CONTENT, DEFAULT_RECIPE_FILE_CONTENT},
    error::Result,
    options::{Options, SubCommand},
    terminal::message::write,
};

#[derive(Serialize)]
struct AppInfo {
    name: String,
    homepage: &'static str,
    version: &'static str,
}

impl AppInfo {
    fn full_name() -> String {
        let mut name = String::new();
        let mut next_is_capital = true;
        for c in APP_NAME.chars() {
            if next_is_capital {
                for c in c.to_uppercase() {
                    name.push(c);
                }
                next_is_capital = false;
            } else if c == '_' || c == '-' {
                name.push(' ');
                next_is_capital = true;
            } else {
                name.push(c);
            }
        }
        name
    }
}

impl Default for AppInfo {
    fn default() -> Self {
        Self {
            name: Self::full_name(),
            homepage: env!("CARGO_PKG_HOMEPAGE"),
            version: env!("CARGO_PKG_VERSION"),
        }
    }
}

fn route(config: &Config, options: Options) -> Result<()> {
    use SubCommand::{Build, Create, Delete, Edit, Info, List};

    match options.subcommand {
        Build(options) => commands::build(config, &options),
        Create(options) => commands::create(config, &options),
        Delete(options) => commands::delete(config, &options),
        Edit(options) => commands::edit(config, &options),
        Info => commands::info(config),
        List(options) => commands::list(config, &options),
    }
}

fn run() -> Result<()> {
    let mut options = Options::parse();
    let config_dir = options.config_dir.get_or_insert_with(|| {
        let home_dir = home_dir().expect("cannot retrieve home directory");
        home_dir.join(".config").join(APP_NAME)
    });
    let config_file = config_dir.join(CONFIG_FILE_NAME);
    if !config_file.exists() {
        fs::create_dir_all(config_dir)?;
        fs::write(&config_file, DEFAULT_CONFIG_FILE_CONTENT)?;
    }
    let mut config = Config::load(&config_file)?;
    let default_recipe_file = config.default_recipe_path();
    if !default_recipe_file.exists() {
        fs::write(&default_recipe_file, DEFAULT_RECIPE_FILE_CONTENT)?;
    }
    let template_dir = config.template_dir();
    if !template_dir.exists() {
        TEMPLATE_DIR.extract(&config.template_dir())?;
    }
    if let Some(path) = &options.recipe_dir {
        config.recipe_dir = path.into();
    }
    if !config.recipe_dir.exists() {
        fs::create_dir_all(&config.recipe_dir)?;
    }
    route(&config, options)
}

fn main() {
    if let Err(error) = run() {
        write::error(error).expect("cannot write error to stderr");
        std::process::exit(1);
    }
}
