# Sweet Potator

[![Latest Version](https://img.shields.io/crates/v/sweet-potator.svg)](https://crates.io/crates/sweet-potator)
[![Documentation](https://docs.rs/toml/badge.svg)](https://docs.rs/sweet-potator)

Sweet Potator is a static recipe site generator.
 
Create, edit and organize your cooking recipes in flat files via command line. Publish them to a web server, generate markdown files or create you own templates to generate any format you want.

Check out the [**demo page**](https://tobx.github.io/sweet-potator/).

## Table of contents
 
- [Features](#features)
- [Installation](#installation)
- [Configuration](#configuration)
- [CLI Usage](#cli-usage)
- [Recipe format](#recipe-format)
- [Search](#search)
- [Multi-language Support](#multi-language-support)
- [Library use (for developers)](#library-use-for-developers)

## Features

- Simple flat file [recipe format](#recipe-format)
- Tagging
- Adjust servings dynamically (HTML template)
- Multi-language support for included templates



## Installation
 
To install this application you need an [installation](https://www.rust-lang.org/tools/install) of [Rust](https://www.rust-lang.org/) and run:

```
cargo install sweet-potator
```

## Configuration

Display config directory location:  
`sweet-potator info`

On *nix systems this should be:  
`~/.config/sweet-potator`

The folder will be created upon first use. It includes the following data:

- `config.toml`: main configuration file
- `default.recipe`: default recipe file to use for new recipes
- `recipes`: recipe directory (includes all recipe files and images)
- `templates`: template directory (includes the content generation templates)

Your favorite editor to use to edit recipe files is probably the first thing you want to configure.

## CLI Usage

Create a new recipe:

```
sweet-potator new
```

Build recipe HTML page:

```
sweet-potator build <output-directory>
```

For more options check out the CLI help:

```bash
# Show help
sweet-potator help

# Show help of a subcommand (`new`, `list`, etc.)
sweet-potator help <subcommand> 
```

Note: `sweet-potator` respects the environment variables `FORCE_COLOR` and `NO_COLOR`.

## Recipe format

A recipe file looks like this:

```
title

Yield: 1
Time: 30m
Author: name
Tags: tag1, tag2

Ingredients
  - name, kind: 1 unit (note)

Instructions
  - instruction

Notes
  - note
```

### Specification (simplified)

1. First line: recipe title

2. Second block: metadata

   - `Yield`: e.g. `4` or `1 Cake`
   - `Time`: e.g. `30m`, `1h` or `1h 30m`
   - `Author` | `Book` | `Link` (recipe source):
     - `Author`: name of the recipe author
     - `Book`: name of a book
     - `Link`: e.g. `link name > https://example.com`
   - `Tags`: list of tags separated by "`, `" (comma + space)

3. Third block: ingredient list. `kind`, `unit` and `note` are optional. The quantity number can either be a number (e.g. `2` or `0.5`) or a fraction (e.g. `1/4`)

4. Forth block: list of plain text recipe instructions

5. Forth block: list of plain text additional notes

## Search

A search function is not included, but there are still ways to search through your recipes.

### Web browser (HTML)

Search through a generated HTML page is as simple as using your browsers search option.

### CLI

Search through your recipes via CLI gets a little tricky, but I find it very satisfying. I am using [skim](https://github.com/lotabout/skim) for it, because I am into Rust tools, but there is at least [fzf](https://github.com/junegunn/fzf) that has some pretty similar functionality. Those tools are super helpful in general for many reasons!

When using `skim` you can search through your recipes with the following commands:

```bash
# Search and edit:
FORCE_COLOR=1 sweet-potator list | sk --ansi | xargs -I{} sweet-potator edit {}

# Search and delete:
FORCE_COLOR=1 sweet-potator list | sk --ansi | xargs -I{} sweet-potator delete {}
```

You might want to create [aliases](https://en.wikipedia.org/wiki/Alias_%28command%29) for those commands (e.g. `edit-recipe` and `delete-recipe`).

CLI search example using [skim](https://github.com/lotabout/skim):

[![asciicast](https://asciinema.org/a/GjFTzUamKuajH0V1EcISeiUWH.svg)](https://asciinema.org/a/GjFTzUamKuajH0V1EcISeiUWH)

## Multi-language Support

The included templates (`html` and `markdown`) have multi-language support. Included languages are English (`en`) and German (`de`). You can configure which language to use in the template sections of the [config file](#configuration).

### Create a new translation

Within the included templates (see [configuration](#configuration)) is a folder `lang`. It includes language files in [TOML](https://toml.io) format. Just copy one and translate it to your language. If you do so, do not forget to share it.

## Library use (for developers)

This is mostly meant for future me, but if anyone else is interested, check out the minimalistic markdown example:

```
cargo run --example markdown
```
