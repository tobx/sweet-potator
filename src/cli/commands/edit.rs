use std::io;

use sweet_potator::recipe::directory::Directory;

use crate::{
    config::Config,
    editor,
    error::{Error, Result},
    options,
    terminal::{color::Colorize, message::write},
    util::os_str_vec,
};

pub fn edit(config: &Config, options: &options::Edit) -> Result<()> {
    if options.image_path.is_some() {
        set_image(config, options)
    } else {
        edit_recipe(config, options)
    }
}

pub fn edit_recipe(config: &Config, options: &options::Edit) -> Result<()> {
    let mut directory = Directory::from_title(&config.recipe_dir, &options.title)?;
    let title = directory
        .load()
        .map_err(Error::from)
        .map_err(|error| match error {
            Error::Io(error) if error.kind() == io::ErrorKind::NotFound => {
                let path = directory.recipe_path();
                let file_name = path.file_name().unwrap();
                Error::RecipeFileNotFound(file_name.to_string_lossy().yellow())
            }
            _ => error,
        })?
        .title;
    editor::open(&config.editor, &directory.recipe_path())?;
    let recipe = directory.load()?;
    if recipe.title != title {
        directory.update_from_title(&recipe.title)?;
    }
    write::success(format!("edited recipe '{}'", recipe.title.yellow()))?;
    Ok(())
}

pub fn set_image(config: &Config, options: &options::Edit) -> Result<()> {
    let directory = Directory::from_title(&config.recipe_dir, &options.title)?;
    if let Some(path) = &options.image_path {
        let file_exts = os_str_vec(&config.image_file_exts);
        directory.copy_image_from(path, &file_exts)?;
    }
    write::success(format!(
        "copied image into recipe directory '{}'",
        directory
            .base_name()
            .to_str()
            .expect("invalid directory name")
            .yellow()
    ))?;
    Ok(())
}
