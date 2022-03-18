use std::{
    ffi::{OsStr, OsString},
    fs, io,
    path::{Path, PathBuf},
};

use serde::Serialize;
use sweet_potator::recipe::{directory::Directory, Recipe};

use crate::{
    config::Config,
    error::{Error, Result},
    options,
    terminal::{color::Colorize, message::write},
};

const RECIPE_JSON_FILE_NAME: &str = "recipes.json";
const RECIPE_IMAGE_DIR: &str = "images";

struct Image(PathBuf);

impl Image {
    fn file_name(&self) -> &OsStr {
        self.0.file_name().expect("invalid file name")
    }
}

impl Serialize for Image {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.file_name().to_str().expect("invalid image path"))
    }
}

#[derive(Serialize)]
struct Entry {
    recipe: Recipe,
    image: Option<Image>,
}

pub fn export(config: &Config, options: &options::Export) -> Result<()> {
    let image_file_exts: Vec<OsString> = config.image_file_exts.iter().map(Into::into).collect();
    let entries: Vec<_> = Directory::list_all(&config.recipe_dir)?
        .iter()
        .map(|directory| {
            let recipe = directory.load()?;
            let image = directory
                .image_file_name(&image_file_exts)?
                .map(|name| Image(Path::new(directory.base_name()).join(name)));
            Ok(Entry { recipe, image })
        })
        .collect::<Result<_>>()?;
    if options.output_dir.exists() {
        return Err(Error::OutputDirectoryAlreadyExists(
            options.output_dir.to_string_lossy().yellow(),
        ));
    }
    fs::create_dir_all(&options.output_dir.join(RECIPE_IMAGE_DIR))?;
    let file = fs::File::options()
        .write(true)
        .create_new(true)
        .open(&options.output_dir.join(RECIPE_JSON_FILE_NAME))?;
    serde_json::to_writer_pretty(&file, &entries).map_err(io::Error::from)?;
    let image_dir = options.output_dir.join(RECIPE_IMAGE_DIR);
    for image in entries.iter().filter_map(|entry| entry.image.as_ref()) {
        println!(
            "copy from {} to {}",
            config.recipe_dir.join(&image.0).to_string_lossy(),
            image_dir.join(image.file_name()).to_string_lossy()
        );
        fs::copy(
            config.recipe_dir.join(&image.0),
            image_dir.join(image.file_name()),
        )?;
    }
    write::success(format!(
        "{} recipes and {} images exported",
        entries.len(),
        entries.iter().filter(|entry| entry.image.is_some()).count(),
    ))?;
    Ok(())
}
