use sweet_potator::recipe::directory::Directory;

use crate::{
    config::Config,
    error::Result,
    options,
    terminal::{color::Colorize, writeln},
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
    let mut titles: Vec<(String, Option<&str>)> = directories
        .iter()
        .map(|directory| {
            let title = directory.load()?.title;
            let suffix = directory.suffix(&title);
            Ok((title, suffix))
        })
        .collect::<Result<_>>()?;
    titles.sort();
    for (title, suffix) in titles {
        if let Some(suffix) = suffix {
            writeln(format!("{}{}", title, suffix.red()))?;
        } else {
            writeln(title)?;
        }
    }
    Ok(())
}
