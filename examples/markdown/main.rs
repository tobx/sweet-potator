use std::{
    env, fs,
    path::{Path, PathBuf},
};

use slug::slugify;
use sweet_potator::{
    error::Result,
    generator::{Generator, TextFilter},
    template,
};
use tera::Context;

pub const TEMPLATE_DIR: &str = "$CARGO_MANIFEST_DIR/src/templates";

pub struct Slugifier;

impl TextFilter for Slugifier {
    fn filter<S: AsRef<str>>(&self, text: S) -> String {
        slugify(text)
    }
}

pub fn build(tpl_dir: &Path, recipe_dir: &Path, output_dir: &Path) -> Result<()> {
    if !output_dir.exists() {
        fs::create_dir_all(output_dir)?;
    }
    let escape = false;
    let mut engine = template::Engine::new(tpl_dir, escape, "md")?;
    let mut context = Context::new();
    context.insert("title", "Sweet Potator Example Document");
    engine.forced_context = Some(context);
    let generator = Generator::new(engine, vec!["jpg".into()], "md".into(), Slugifier);
    Ok(generator.generate(recipe_dir, output_dir)?)
}

fn main() {
    let example_dir =
        PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap()).join("examples/markdown");
    let tpl_dir = example_dir.join("template");
    let recipe_dir = example_dir.join("recipes");
    let output_dir = example_dir.join("generated_files");
    build(&tpl_dir, &recipe_dir, &output_dir).unwrap();
    println!("Markdown example files have been successfully created.");
    println!("Output directory: {}", output_dir.to_string_lossy());
}
