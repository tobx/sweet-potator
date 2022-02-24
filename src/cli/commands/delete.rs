use std::io;

use sweet_potator::recipe::directory::Directory;

use crate::{
    config::Config,
    error::{Error, Result},
    options,
    terminal::{color::Colorize, message::write},
};

pub fn delete(config: &Config, options: &options::Delete) -> Result<()> {
    let directory = Directory::from_title(&config.recipe_dir, &options.title)?;
    directory.delete().map_err(|error| {
        if let io::ErrorKind::NotFound = error.kind() {
            Error::RecipeDirNotFound(directory.base_name().to_string_lossy().yellow())
        } else {
            error.into()
        }
    })?;
    write::success(format!(
        "deleted recipe '{}'",
        options.title.trim().yellow()
    ))?;
    Ok(())
}
