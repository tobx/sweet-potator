use std::{
    ffi::{OsStr, OsString},
    fs,
    io::{self, Write},
    path::{Path, PathBuf},
};

use crate::{
    error::{Error, Result},
    util::{append_os_file_ext, sanitize_file_name},
};

use super::Recipe;

pub const RECIPE_FILE_EXT: &str = "recipe";

#[derive(Debug)]
pub struct Directory {
    parent: PathBuf,
    name: OsString,
}

impl Directory {
    pub fn from_title(parent: &Path, title: &str) -> Result<Self> {
        Ok(Self {
            parent: parent.into(),
            name: title_to_name(title)?.into(),
        })
    }

    pub fn list_all(path: &Path) -> io::Result<Vec<Self>> {
        let mut directories = Vec::new();
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                directories.push(Directory {
                    parent: path.into(),
                    name: entry
                        .path()
                        .file_name()
                        .expect("invalid directory file name")
                        .into(),
                });
            }
        }
        Ok(directories)
    }

    pub fn base_name(&self) -> &OsStr {
        &self.name
    }

    pub fn copy_image_from(&self, path: &Path, file_exts: &[&OsStr]) -> Result<()> {
        let ext = path
            .extension()
            .ok_or_else(|| Error::MissingImageFileExt(path.into()))?;
        if !file_exts.contains(&ext) {
            return Err(Error::InvalidImageFileExt(ext.into()));
        }
        fs::copy(path, self.file_path(ext))?;
        Ok(())
    }

    pub fn delete(&self) -> io::Result<()> {
        fs::remove_dir_all(self.path())
    }

    pub fn image_file_name(&self, file_exts: &[OsString]) -> io::Result<Option<OsString>> {
        let path = self.path();
        for entry in fs::read_dir(&path)? {
            let entry = entry?;
            if entry.file_type()?.is_file() {
                let path = entry.path();
                if self.is_image_path(&path, file_exts) {
                    return Ok(Some(
                        path.file_name().expect("invalid image file name").into(),
                    ));
                }
            }
        }
        Ok(None)
    }

    pub fn load(&self) -> Result<Recipe> {
        let path = self.recipe_path();
        let file = fs::File::open(&path)?;
        Recipe::parse_from(file).map_err(|mut error| {
            if let Error::Parse(error) = &mut error {
                error.set_path(&path);
            }
            error
        })
    }

    pub fn path(&self) -> PathBuf {
        self.parent.join(&self.name)
    }

    pub fn recipe_path(&self) -> PathBuf {
        self.file_path(OsStr::new(RECIPE_FILE_EXT)).into()
    }

    pub fn store(&mut self, recipe: &Recipe) -> io::Result<()> {
        if let Some(name) = create(&self.parent, &self.name)? {
            self.name = name;
        }
        let recipe_path = self.recipe_path();
        let mut file = fs::File::options()
            .write(true)
            .create_new(true)
            .open(&recipe_path)?;
        write!(file, "{}", recipe)?;
        Ok(())
    }

    pub fn suffix(&self, title: &str) -> Option<&str> {
        let directory = Self::from_title(&self.parent, title).expect("empty recipe title");
        self.name
            .to_str()
            .and_then(|name| {
                directory
                    .name
                    .to_str()
                    .and_then(|test_name| name.strip_prefix(test_name))
            })
            .filter(|suffix| !suffix.is_empty())
    }

    pub fn update_from_title(&mut self, title: &str) -> Result<()> {
        let mut name: OsString = title_to_name(title)?.into();
        if name == self.name {
            return Ok(());
        }
        if self.parent.join(&name).exists() {
            name = find_available_name(&self.parent, &name, Some(&self.name))?;
        }
        self.rename(name)
    }

    fn file_path(&self, file_ext: &OsStr) -> OsString {
        append_os_file_ext(self.path().join(&self.name).into_os_string(), file_ext)
    }

    fn is_image_path(&self, path: &Path, file_exts: &[OsString]) -> bool {
        path.file_stem().map_or(false, |stem| stem == self.name)
            && matches!(
                path.extension(),
                Some(extension) if file_exts.iter().any(|ext|ext == extension)
            )
    }

    fn rename(&mut self, name: OsString) -> Result<()> {
        let path = self.parent.join(&name);
        fs::rename(&self.path(), &path)?;
        self.name = name;
        let mut files = Vec::new();
        for entry in fs::read_dir(&path)? {
            let entry = entry?;
            if entry.file_type()?.is_file() {
                files.push(entry.path());
            }
        }
        for file in files {
            let mut name = self.name.clone();
            if let Some(ext) = file.extension() {
                name = append_os_file_ext(name, ext);
            }
            fs::rename(file, path.join(name))?;
        }
        Ok(())
    }
}

fn create(parent: &Path, name: &OsStr) -> io::Result<Option<OsString>> {
    match fs::create_dir(parent.join(name)) {
        Ok(()) => Ok(None),
        Err(error) if matches!(error.kind(), io::ErrorKind::AlreadyExists) => {
            let name = find_available_name(parent, name, None)?;
            fs::create_dir(parent.join(&name))?;
            Ok(Some(name))
        }
        Err(error) => Err(error),
    }
}

// Renaming can end up with the same directory name:
// e.g. rename from `name (1)` to `name` while `name` already exists
//
// Provide some `current_name` in order to not skip the current name
// while iterating to find an available one.
fn find_available_name(
    parent: &Path,
    name: &OsStr,
    current_name: Option<&OsStr>,
) -> io::Result<OsString> {
    let mut i: usize = 2;
    loop {
        let mut name = name.to_owned();
        name.push(" (");
        name.push(i.to_string());
        name.push(")");
        if !parent.join(&name).exists() || current_name.map_or(false, |n| n == name) {
            return Ok(name);
        }
        i += 1;
    }
}

fn sanitize_title(title: &str) -> Result<&str> {
    let title = title.trim();
    if title.is_empty() {
        return Err(Error::EmptyRecipeTitle);
    }
    Ok(title)
}

fn title_to_name(title: &str) -> Result<String> {
    Ok(sanitize_file_name(sanitize_title(title)?))
}

#[cfg(test)]
mod tests {

    use tempfile::tempdir;

    use super::*;

    #[test]
    fn test_from_title() {
        let directory = Directory::from_title(Path::new("test"), " a/b ").unwrap();
        assert_eq!(directory.name, OsStr::new("a_b"));
        assert!(Directory::from_title(Path::new("test"), " ").is_err());
    }

    #[test]
    fn test_update_from_title() -> Result<()> {
        // create temp directories
        let temp_dir = tempdir()?;
        let temp_path = temp_dir.path();
        let recipe_path = temp_path.join("recipe");
        let new_recipe_path = temp_path.join("new recipe");
        let new_recipe_path_1 = temp_path.join("new recipe (1)");
        fs::create_dir_all(&recipe_path)?;
        fs::create_dir_all(&new_recipe_path)?;
        fs::create_dir_all(&new_recipe_path_1)?;
        fs::File::create(recipe_path.join("recipe.recipe"))?;
        fs::File::create(recipe_path.join("recipe.jpg"))?;

        // update directory
        let mut directory = Directory {
            parent: temp_path.into(),
            name: "recipe".into(),
        };
        directory.update_from_title("new recipe")?;

        // validate file structure
        assert!(!recipe_path.exists());
        let recipe_path = temp_path.join("new recipe (2)");
        assert!(recipe_path.exists());
        assert!(recipe_path.join("new recipe (2).recipe").exists());
        assert!(recipe_path.join("new recipe (2).jpg").exists());

        Ok(())
    }
}
