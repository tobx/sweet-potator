#![forbid(unsafe_code)]

pub mod error;
pub mod generator;
pub mod recipe;
pub mod template;
pub mod util;

use include_dir::{include_dir, Dir};

pub const APP_NAME: &str = env!("CARGO_PKG_NAME");

pub const TEMPLATE_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/src/templates");
