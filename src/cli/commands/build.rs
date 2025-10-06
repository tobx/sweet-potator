use std::fs;

use sweet_potator::{generator::Generator, template};
use tera::Context;

use crate::{
    AppInfo,
    config::Config,
    error::{Error, Result},
    options,
    terminal::color::Colorize,
};

pub fn build(config: &Config, options: &options::Build) -> Result<()> {
    if !options.output_dir.exists() {
        fs::create_dir_all(&options.output_dir)?;
    }
    let tpl_name = &options.template_name;
    let tpl_options = config
        .templates
        .get(tpl_name)
        .ok_or_else(|| Error::TemplateNameNotConfigured(tpl_name.yellow()))?;
    let tpl_dir = if let Some(dir) = &options.template_dir {
        dir.join(tpl_name)
    } else {
        config.template_dir().join(tpl_name)
    };
    if !tpl_dir.exists() {
        return Err(Error::TemplateNameNotFound(options.template_name.yellow()));
    }
    let mut engine = template::Engine::new(
        &tpl_dir,
        tpl_options.escape,
        &tpl_options.extension,
        Some(&tpl_options.language),
    )?;
    let mut context = Context::new();
    context.insert("app", &AppInfo::default());
    context.insert("lf", "\n");
    engine.forced_context = Some(context);
    let generator = Generator::new(
        engine,
        config.image_file_exts.iter().map(Into::into).collect(),
        tpl_options.extension.as_str().into(),
        tpl_options.file_name_filter,
    );
    Ok(generator.generate(&config.recipe_dir, &options.output_dir)?)
}
