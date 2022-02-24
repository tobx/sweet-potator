use sweet_potator::recipe::directory::Directory;

use crate::{
    config::Config,
    error::Result,
    terminal::{color::Colorize, writeln},
    AppInfo,
};

pub fn info(config: &Config) -> Result<()> {
    let info = AppInfo::default();
    let recipe_count = Directory::list_all(&config.recipe_dir)?.len();
    let mappings = [
        ("Name", info.name.as_str()),
        ("Version", info.version),
        ("Homepage", info.homepage),
        ("Config dir", &config.dir.to_string_lossy()),
        ("Recipe dir", &config.recipe_dir.to_string_lossy()),
        ("Recipes", &recipe_count.to_string()),
    ];
    let width = mappings
        .iter()
        .fold(0, |acc, (name, _)| name.chars().count().max(acc));
    for (name, value) in mappings {
        writeln(format!(
            "{:>width$} {} {}",
            name,
            "·".green(),
            value,
            width = width
        ))?;
    }
    Ok(())
}
