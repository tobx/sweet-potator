use std::{
    collections::HashSet,
    ffi::OsString,
    fs,
    path::{Path, PathBuf},
};

use serde::Serialize;
use tera::Context;

use crate::{
    error::Result,
    recipe::directory::Directory,
    template,
    util::{append_os_file_ext, copy_dir, UniqueNameFinder},
};

pub trait TextFilter {
    fn filter<S: AsRef<str>>(&self, text: S) -> String;
}

const INDEX_NAME: &str = "index";

const IMAGE_DIR: &str = "images";
const RECIPE_DIR: &str = "recipes";
const STATIC_DIR: &str = "static";

#[derive(Debug, Serialize)]
struct IndexEntry {
    pub title: String,
    pub path: PathBuf,
    pub tags: Vec<String>,
    pub image_path: Option<PathBuf>,
}

pub struct Generator<F> {
    engine: template::Engine,
    image_file_exts: Vec<OsString>,
    output_file_ext: OsString,
    file_name_filter: F,
}

impl<F> Generator<F> {
    pub fn new(
        engine: template::Engine,
        image_file_exts: Vec<OsString>,
        output_file_ext: OsString,
        file_name_filter: F,
    ) -> Self {
        Self {
            engine,
            image_file_exts,
            output_file_ext,
            file_name_filter,
        }
    }
}

impl<F: TextFilter> Generator<F> {
    pub fn generate(&self, recipe_dir: &Path, output_dir: &Path) -> Result<()> {
        let index = self.render_recipes(recipe_dir, output_dir)?;
        if self.engine.has_index_template() {
            self.render_index(&index, output_dir)?;
        }
        let static_path = self.engine.static_path();
        if static_path.exists() {
            copy_dir(static_path, output_dir.join(STATIC_DIR))?;
        }
        Ok(())
    }

    fn render_index(&self, entries: &[IndexEntry], output_dir: &Path) -> Result<()> {
        let tags = get_distinct_tags(entries);
        let mut entries: Vec<_> = entries.iter().collect();
        entries.sort_by_key(|entry| &entry.title);
        let mut context = Context::new();
        context.insert("recipes", &entries);
        context.insert("tags", &tags);
        let path = output_dir.join(append_os_file_ext(INDEX_NAME, &self.output_file_ext));
        fs::create_dir_all(&path.parent().expect("invalid index template path"))?;
        let file = fs::File::create(path)?;
        self.engine.render_index(context, file)
    }

    fn copy_image(
        &self,
        directory: &Directory,
        slug: &str,
        output_dir: &Path,
    ) -> Result<Option<OsString>> {
        directory
            .image_file_name(&self.image_file_exts)?
            .map(|name| {
                let ext = Path::new(&name).extension().unwrap();
                let new_name = append_os_file_ext(slug, ext);
                fs::copy(directory.path().join(&name), output_dir.join(&new_name))?;
                Ok(new_name)
            })
            .transpose()
    }

    fn render_recipes(&self, recipe_dir: &Path, output_dir: &Path) -> Result<Vec<IndexEntry>> {
        let recipe_output_dir = output_dir.join(Path::new(RECIPE_DIR));
        fs::create_dir_all(&recipe_output_dir)?;
        let image_output_dir = output_dir.join(Path::new(IMAGE_DIR));
        fs::create_dir_all(&image_output_dir)?;
        let mut name_finder = UniqueNameFinder::new(" (", ")");
        let mut index_entries = Vec::new();
        for directory in Directory::list_all(recipe_dir)? {
            let recipe = directory.load()?;
            let name = name_finder.find(self.file_name_filter.filter(&recipe.title));
            let recipe_file_name = append_os_file_ext(&name, &self.output_file_ext);
            let relative_recipe_path = Path::new(RECIPE_DIR).join(&recipe_file_name);
            let image_file_name = self.copy_image(&directory, &name, &image_output_dir)?;
            let relative_image_path = image_file_name.map(|name| Path::new(IMAGE_DIR).join(name));
            let mut context = Context::new();
            context.insert("recipe", &recipe);
            context.insert("path", relative_recipe_path.to_str().unwrap());
            context.insert(
                "image_path",
                &relative_image_path
                    .as_deref()
                    .map(|path| path.to_str().unwrap()),
            );
            let mut tags = recipe.metadata.tags;
            tags.sort();
            let index_entry = IndexEntry {
                title: recipe.title,
                path: relative_recipe_path,
                tags,
                image_path: relative_image_path,
            };
            let file = fs::File::create(recipe_output_dir.join(recipe_file_name))?;
            self.engine.render_recipe(context, file)?;
            index_entries.push(index_entry);
        }
        Ok(index_entries)
    }
}

fn get_distinct_tags<'a>(entries: &'a [IndexEntry]) -> Vec<&'a String> {
    let mut tags = HashSet::new();
    for entry in entries {
        for tag in &entry.tags {
            if tags.contains(tag) {
            } else {
                tags.insert(tag);
            }
        }
    }
    let mut tags: Vec<_> = tags.into_iter().collect();
    tags.sort();
    tags
}

#[cfg(test)]
mod tests {
    use std::{
        collections::HashMap,
        fs::File,
        io::{Cursor, Write},
    };

    use tempfile::tempdir;

    use crate::recipe::Recipe;

    use super::*;

    struct FileNameFilter;

    impl TextFilter for FileNameFilter {
        fn filter<S: AsRef<str>>(&self, text: S) -> String {
            text.as_ref().to_uppercase()
        }
    }

    #[test]
    fn test() -> Result<()> {
        // create temp directories
        let temp_dir = tempdir()?;
        let temp_path = temp_dir.path();
        let tpl_dir = temp_path.join("template");
        let static_dir = tpl_dir.join(STATIC_DIR);
        let tera_dir = tpl_dir.join("tera");
        let recipe_dir = temp_path.join(RECIPE_DIR);
        let output_dir = temp_path.join("output");
        fs::create_dir_all(&tpl_dir)?;
        fs::create_dir_all(&static_dir)?;
        fs::create_dir_all(&tera_dir)?;
        fs::create_dir_all(&recipe_dir)?;
        fs::create_dir_all(&output_dir)?;

        // create recipe template
        let mut file = File::create(tera_dir.join("recipe.html"))?;
        writeln!(file, "title: {{{{ recipe.title }}}}")?;

        // create index template
        let mut file = File::create(tera_dir.join("index.html"))?;
        writeln!(
            file,
            "{{% for r in recipes -%}}{{{{ r.title }}}}{{%- endfor %}}"
        )?;

        // default recipe
        let recipe_str = "title\n\nServings: 1\n\nIngredients\n- nothing\n\nInstructions\n- none";

        // create and store recipe 1
        let mut recipe1 = Recipe::parse_from(Cursor::new(recipe_str))?;
        recipe1.title = "recipe 1".into();
        fs::create_dir(recipe_dir.join("recipe 1"))?;
        let mut file = File::create(recipe_dir.join("recipe 1/recipe 1.recipe"))?;
        writeln!(file, "{}", recipe1)?;

        // create and store recipe 1 (2)
        let mut recipe1_2 = Recipe::parse_from(Cursor::new(recipe_str))?;
        recipe1_2.title = "recipe 1".into();
        fs::create_dir(recipe_dir.join("recipe 1 (2)"))?;
        let mut file = File::create(recipe_dir.join("recipe 1 (2)/recipe 1 (2).recipe"))?;
        writeln!(file, "{}", recipe1_2)?;

        // create and store recipe 2
        let mut recipe2 = Recipe::parse_from(Cursor::new(recipe_str))?;
        recipe2.title = "recipe 2".into();
        fs::create_dir(recipe_dir.join("recipe 2"))?;
        let mut file = File::create(recipe_dir.join("recipe 2/recipe 2.recipe"))?;
        writeln!(file, "{}", recipe2)?;

        // add image to recipe 2
        File::create(recipe_dir.join("recipe 2/recipe 2.jpg"))?;

        // create static content
        File::create(static_dir.join("test.txt"))?;

        // generate html
        let mut context = Context::new();
        context.insert(
            "app",
            &HashMap::from([("name", "name"), ("homepage", "homepage")]),
        );
        let mut engine = template::Engine::new(&tpl_dir, true, "html")?;
        engine.forced_context = Some(context);
        let image_file_exts = vec!["jpg".into()];

        let generator = Generator::new(engine, image_file_exts, "html".into(), FileNameFilter);
        generator.generate(&recipe_dir, &output_dir)?;

        // validate html
        let recipe_output_dir = output_dir.join(RECIPE_DIR);
        let image_output_dir = output_dir.join(IMAGE_DIR);
        let static_output_dir = output_dir.join(STATIC_DIR);
        let recipe1 = fs::read_to_string(recipe_output_dir.join("RECIPE 1.html"))?;
        assert_eq!(recipe1, "title: recipe 1\n");
        let recipe1_2 = fs::read_to_string(recipe_output_dir.join("RECIPE 1 (2).html"))?;
        assert_eq!(recipe1_2, "title: recipe 1\n");
        let recipe2 = fs::read_to_string(recipe_output_dir.join("RECIPE 2.html"))?;
        assert_eq!(recipe2, "title: recipe 2\n");
        assert!(image_output_dir.join("RECIPE 2.jpg").exists());
        let index = fs::read_to_string(output_dir.join("index.html"))?;
        assert_eq!(index, concat!("recipe 1", "recipe 1", "recipe 2", "\n"));
        assert!(static_output_dir.join("test.txt").exists());

        Ok(())
    }
}
