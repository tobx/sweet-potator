use sweet_potator::recipe::directory::Directory;

use crate::{
    config::Config,
    error::{Error, Result},
    options,
    terminal::{color::Colorize, message::write, writeln},
};

pub fn list(config: &Config, options: &options::List) -> Result<()> {
    let directories = Directory::list_all(&config.recipe_dir)?;
    if options.list_files {
        list_files(&directories)
    } else {
        list_titles(&directories)
    }
}

fn list_files(directories: &[Directory]) -> Result<()> {
    let mut names: Vec<String> = directories
        .iter()
        .map(|directory| {
            Ok(directory
                .recipe_path()
                .file_name()
                .unwrap()
                .to_string_lossy()
                .into())
        })
        .collect::<Result<_>>()?;
    names.sort();
    for name in names {
        writeln(name)?;
    }
    Ok(())
}

fn list_titles(directories: &[Directory]) -> Result<()> {
    let mut result = Ok(());
    let mut titles = Vec::new();
    for directory in directories {
        match directory.load() {
            Ok(recipe) => {
                let title = recipe.title;
                let suffix = directory.suffix(&title);
                titles.push((title, suffix));
            }
            Err(error) => {
                write::error(error)?;
                result = Err(Error::CorruptedRecipeList);
            }
        }
    }
    titles.sort();
    for (title, suffix) in titles {
        if let Some(suffix) = suffix {
            writeln(format!("{}{}", title, suffix.red()))?;
        } else {
            writeln(title)?;
        }
    }
    result
}
