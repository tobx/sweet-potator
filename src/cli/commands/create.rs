use std::fs;

use sweet_potator::recipe::{directory::Directory, Recipe};

use crate::{
    config::Config,
    editor,
    error::Result,
    options,
    terminal::{color::Colorize, message::write},
    util::os_str_vec,
};

pub fn create(config: &Config, options: &options::Create) -> Result<()> {
    let title = options.title.as_deref().unwrap_or("Untitled");
    let mut directory = Directory::from_title(&config.recipe_dir, title)?;
    let file = fs::File::open(config.default_recipe_path())?;
    let mut recipe = Recipe::parse_from(file).expect("error in 'template.recipe'");
    if options.title.is_some() {
        recipe.title = title.into();
    }
    directory.store(&recipe)?;
    if let Some(path) = &options.image_path {
        let file_exts = os_str_vec(&config.image_file_exts);
        directory.copy_image_from(path, &file_exts)?;
    }
    editor::open(&config.editor, &directory.recipe_path())?;
    let recipe = directory.load()?;
    if recipe.title != title {
        directory.update_from_title(&recipe.title)?;
    }
    write::success(format!("created recipe '{}'", recipe.title.yellow()))?;
    Ok(())
}
