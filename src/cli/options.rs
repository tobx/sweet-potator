use std::path::PathBuf;

use clap::Parser;

#[derive(Parser)]
#[clap(version, about)]
pub struct Options {
    /// Config directory
    #[clap(long)]
    pub config_dir: Option<PathBuf>,

    /// Recipe directory
    #[clap(long)]
    pub recipe_dir: Option<PathBuf>,

    #[clap(subcommand)]
    pub subcommand: SubCommand,
}

#[derive(Parser)]
pub enum SubCommand {
    Build(Build),
    #[clap(name = "new")]
    Create(Create),
    Delete(Delete),
    Edit(Edit),
    Export(Export),
    Info,
    List(List),
}

/// Build recipe page
#[derive(Default, Parser)]
pub struct Build {
    /// Template name to use
    #[clap(long = "template", value_name = "NAME", default_value = "html")]
    pub template_name: String,

    /// Use a custom template directory
    #[clap(long, value_name = "DIR")]
    pub template_dir: Option<PathBuf>,

    /// Output (build) directory
    pub output_dir: PathBuf,
}

/// Create new recipe
#[derive(Default, Parser)]
pub struct Create {
    /// Recipe image file path
    #[clap(long = "image", value_name = "FILE")]
    pub image_path: Option<PathBuf>,

    /// Recipe title
    pub title: Option<String>,
}

/// Delete a recipe
#[derive(Default, Parser)]
pub struct Delete {
    /// Recipe title
    pub title: String,
}

/// Edit a recipe
#[derive(Default, Parser)]
pub struct Edit {
    /// Set recipe image
    #[clap(long = "set-image", value_name = "FILE")]
    pub image_path: Option<PathBuf>,

    /// Recipe title
    pub title: String,
}

/// Export recipes as JSON file
#[derive(Default, Parser)]
pub struct Export {
    /// Output directory
    pub output_dir: PathBuf,
}

/// List recipes
#[derive(Default, Parser)]
pub struct List {
    /// List recipe file names instead of titles
    #[clap(long = "files")]
    pub list_files: bool,

    /// Filter recipes by tags
    #[clap(long)]
    pub tags: Option<Vec<String>>,
}

/// Show application info
#[derive(Default, Parser)]
pub struct Info;
