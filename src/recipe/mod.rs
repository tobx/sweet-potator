pub mod directory;
pub mod errors;
pub mod ingredient;
pub mod list;
pub mod metadata;
mod reader;

use std::{fmt, io};

use serde::Serialize;

use crate::error::Result;

use self::{
    errors::{ParseError, ParseResult},
    ingredient::Ingredient,
    list::List,
    metadata::Metadata,
    reader::Reader,
};

pub trait ParseFromStr: Sized {
    fn parse_from_str(s: &str) -> ParseResult<Self>;
}

impl ParseFromStr for String {
    fn parse_from_str(s: &str) -> ParseResult<Self> {
        Ok(s.into())
    }
}

#[derive(Debug, Serialize)]
pub struct Recipe {
    pub title: String,
    pub metadata: Metadata,
    pub ingredients: List<Ingredient>,
    pub instructions: List<String>,
    pub notes: Vec<String>,
}

impl Recipe {
    pub fn parse_from(reader: impl io::Read) -> Result<Self> {
        let mut reader = Reader::new(reader, true);
        Ok(Self {
            title: reader
                .next_block()?
                .ok_or_else(|| ParseError::from("missing title"))
                .and_then(Self::parse_title)?,
            metadata: reader
                .next_block()?
                .ok_or_else(|| ParseError::from("missing metadata"))
                .and_then(Metadata::try_from)?,
            ingredients: reader
                .next_block()?
                .ok_or_else(|| ParseError::from("missing ingredients"))
                .and_then(Self::parse_ingredients)?,
            instructions: reader
                .next_block()?
                .ok_or_else(|| ParseError::from("missing instructions"))
                .and_then(Self::parse_instructions)?,
            notes: reader
                .next_block()?
                .map_or_else(|| Ok(Vec::new()), Self::parse_notes)?,
        })
    }

    fn parse_title(mut lines: Vec<String>) -> ParseResult<String> {
        if lines.len() > 1 {
            return Err("missing empty line after title line".into());
        }
        lines.pop().ok_or_else(|| ParseError::empty("title line"))
    }

    fn parse_ingredients(mut lines: Vec<String>) -> ParseResult<List<Ingredient>> {
        if !matches!(lines.first(), Some(line) if line == "Ingredients") {
            return Err("missing headline 'Ingredients'".into());
        }
        lines.remove(0);
        lines.try_into()
    }

    fn parse_instructions(mut lines: Vec<String>) -> ParseResult<List<String>> {
        if !matches!(lines.first(), Some(line) if line == "Instructions") {
            return Err("missing headline 'Instructions'".into());
        }
        lines.remove(0);
        lines.try_into()
    }

    fn parse_notes(mut lines: Vec<String>) -> ParseResult<Vec<String>> {
        if !matches!(lines.first(), Some(line) if line == "Notes") {
            return Err("expected headline 'Notes'".into());
        }
        lines.remove(0);
        List::<String>::parse_basic(&lines)
    }
}

impl fmt::Display for Recipe {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let indentation = " ".repeat(2);
        writeln!(f, "{}\n", self.title)?;
        writeln!(f, "{}", self.metadata)?;
        writeln!(f, "Ingredients")?;
        self.ingredients.format(f, &indentation)?;
        writeln!(f, "\nInstructions")?;
        self.instructions.format(f, &indentation)?;
        if !self.notes.is_empty() {
            writeln!(f, "\nNotes")?;
            list::format_items(&self.notes, f, &indentation)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {

    use super::{
        list::Section,
        metadata::{Duration, Link, Source, Yield},
        *,
    };

    const RECIPE_TO_PARSE: &str = concat!(
        " \n",
        " the title \n",
        " \n",
        " Yield :  10  u n i t \n",
        " Time: 1h  30m\n",
        " Link: the name  >  the url\n",
        " Tags: tag 1 ,  tag 2\n",
        " \n",
        " Ingredients \n",
        " section 1 \n",
        " -  name 1 ,  kind 1 :  10  1 unit  ( note 1 ) \n",
        " -  name 2 \n",
        " section 2 \n",
        "   - name \n",
        " \n",
        " Instructions \n",
        " section 1 \n",
        " - instruction 1 \n",
        "   - instruction 2 \n",
        " \n",
        " Notes \n",
        "  - note",
        " \n"
    );

    const RECIPE_TO_DISPLAY: &str = concat!(
        "title\n",
        "\n",
        "Yield: 1 unit\n",
        "Time: 1h 30m\n",
        "Link: name > url\n",
        "Tags: tag1, tag2\n",
        "\n",
        "Ingredients\n",
        "  section\n",
        "    - name\n",
        "\n",
        "Instructions\n",
        "  section\n",
        "    - instruction\n",
        "\n",
        "Notes\n",
        "  - note\n"
    );

    #[test]
    fn test_display() {
        let recipe = Recipe {
            title: "title".into(),
            metadata: Metadata {
                duration: Some(Duration {
                    hours: 1,
                    minutes: 30,
                }),
                yields: Yield {
                    value: 1,
                    unit: Some("unit".into()),
                },
                source: Some(Source::Link(Link {
                    name: "name".into(),
                    url: "url".into(),
                })),
                tags: vec!["tag1".into(), "tag2".into()],
            },
            ingredients: List::Sectioned(vec![Section::new(
                "section".into(),
                vec![Ingredient {
                    name: "name".into(),
                    kind: None,
                    quantity: None,
                }],
            )]),
            instructions: List::Sectioned(vec![Section::new(
                "section".into(),
                vec!["instruction".into()],
            )]),
            notes: vec!["note".into()],
        };
        assert_eq!(recipe.to_string(), RECIPE_TO_DISPLAY)
    }

    #[test]
    fn test_parse() {
        let reader = io::Cursor::new(RECIPE_TO_PARSE);
        let recipe = Recipe::parse_from(reader).unwrap();

        // title
        assert_eq!(recipe.title, "the title");

        // metadata
        let metadata = recipe.metadata;

        assert_eq!(metadata.yields.value, 10);
        assert_eq!(metadata.yields.unit.as_deref(), Some("u n i t"));
        assert!(metadata.duration.is_some());
        if let Some(duration) = metadata.duration {
            assert_eq!(duration.hours, 1);
            assert_eq!(duration.minutes, 30);
        }
        assert!(matches!(metadata.source, Some(Source::Link(_))));
        if let Some(Source::Link(link)) = metadata.source {
            assert_eq!(link.name, "the name");
            assert_eq!(link.url, "the url");
        }
        assert_eq!(metadata.tags, ["tag 1", "tag 2"]);

        // ingredients
        assert!(matches!(recipe.ingredients, List::Sectioned(_)));
        if let List::Sectioned(sections) = recipe.ingredients {
            assert_eq!(sections.len(), 2);

            // section 1
            let section = sections.get(0).unwrap();
            assert_eq!(section.name, "section 1");
            assert_eq!(section.items.len(), 2);
            let item = section.items.get(0).unwrap();
            assert_eq!(item.name, "name 1");
            assert_eq!(item.kind, Some("kind 1".into()));
            let quantity = item.quantity.as_ref().unwrap();
            assert_eq!(quantity.value.to_string(), "10");
            assert_eq!(quantity.unit, Some("1 unit".into()));
            assert_eq!(quantity.note, Some("note 1".into()));
            let item = section.items.get(1).unwrap();
            assert_eq!(item.name, "name 2");
            assert_eq!(item.kind, None);
            assert!(item.quantity.is_none());

            // section 2
            let section = sections.get(1).unwrap();
            assert_eq!(section.name, "section 2");
            assert_eq!(section.items.len(), 1);
            let item = section.items.get(0).unwrap();
            assert_eq!(item.name, "name");
            assert_eq!(item.kind, None);
            assert!(item.quantity.is_none());
        }

        // instructions
        assert!(matches!(recipe.instructions, List::Sectioned(_)));
        if let List::Sectioned(sections) = recipe.instructions {
            assert_eq!(sections.len(), 1);

            // section 1
            let section = sections.get(0).unwrap();
            assert_eq!(section.name, "section 1");
            assert_eq!(section.items.len(), 2);
            assert_eq!(section.items.get(0).unwrap(), "instruction 1");
            assert_eq!(section.items.get(1).unwrap(), "instruction 2");
        }
    }

    #[test]
    fn test_parse_empty() {
        let reader = io::Cursor::new("");
        assert!(Recipe::parse_from(reader).is_err());
    }

    #[test]
    fn test_parse_missing() {
        let reader = io::Cursor::new("");
        assert!(Recipe::parse_from(reader).is_err());
        let reader = io::Cursor::new("title\n\nYield: 1\n\nIngredients\nNothing");
        assert!(Recipe::parse_from(reader).is_err());
    }
}
