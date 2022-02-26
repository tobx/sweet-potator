use std::{fmt, slice};

use serde::Serialize;

use super::{
    errors::{ParseError, ParseResult},
    ParseFromStr,
};

#[derive(Debug, Serialize)]
pub struct Section<D> {
    pub name: String,
    pub items: Vec<D>,
}

impl<D> Section<D> {
    pub fn new(name: String, items: Vec<D>) -> Self {
        Self { name, items }
    }
}

impl<P: ParseFromStr> Section<P> {
    fn parse_items<'a>(
        iter: &'a mut slice::Iter<String>,
    ) -> ParseResult<(Vec<P>, Option<&'a String>)> {
        let mut items = Vec::new();
        for line in iter {
            if let Ok(line) = strip_prefix(line) {
                items.push(P::parse_from_str(line)?);
            } else {
                return Ok((items, Some(line)));
            }
        }
        Ok((items, None))
    }
}

impl<D: fmt::Display> Section<D> {
    fn format(&self, f: &mut fmt::Formatter<'_>, indentation: &str) -> fmt::Result {
        writeln!(f, "{}{}", indentation, self.name)?;
        format_items(&self.items, f, &format!("{0}{0}", indentation))
    }
}

#[derive(Debug, Serialize)]
pub enum List<D> {
    #[serde(rename = "items")]
    Basic(Vec<D>),
    #[serde(rename = "sections")]
    Sectioned(Vec<Section<D>>),
}

impl<D: fmt::Display> List<D> {
    pub fn format(&self, f: &mut fmt::Formatter<'_>, indent_size: usize) -> fmt::Result {
        let indentation = " ".repeat(indent_size);
        match self {
            Self::Basic(items) => {
                format_items(items, f, &indentation)?;
            }
            Self::Sectioned(sections) => {
                for section in sections {
                    section.format(f, &indentation)?;
                }
            }
        }
        Ok(())
    }
}

impl<P: ParseFromStr> List<P> {
    fn parse_basic<S: ParseFromStr>(lines: &[String]) -> ParseResult<Vec<S>> {
        lines
            .iter()
            .map(|line| strip_prefix(line).and_then(S::parse_from_str))
            .collect()
    }

    fn parse_sectioned(lines: &[String]) -> ParseResult<Vec<Section<P>>> {
        let mut sections = Vec::new();
        let mut lines = lines.iter();
        let mut section_name = if let Some(line) = lines.next() {
            line.to_string()
        } else {
            return Ok(sections);
        };
        loop {
            if section_name.is_empty() {
                return Err(ParseError::empty("list section name"));
            }
            let (items, new_section_name) = Section::parse_items(&mut lines)?;
            sections.push(Section::new(section_name, items));
            if let Some(name) = new_section_name {
                section_name = name.into();
            } else {
                break;
            }
        }
        Ok(sections)
    }
}

impl<D: ParseFromStr> TryFrom<Vec<String>> for List<D> {
    type Error = ParseError;

    fn try_from(lines: Vec<String>) -> Result<Self, Self::Error> {
        if lines.get(0).map_or(true, |line| line.starts_with("- ")) {
            Ok(Self::Basic(Self::parse_basic(&lines)?))
        } else {
            Ok(Self::Sectioned(Self::parse_sectioned(&lines)?))
        }
    }
}

fn format_items<D: fmt::Display>(
    items: &[D],
    f: &mut fmt::Formatter<'_>,
    indentation: &str,
) -> fmt::Result {
    for item in items {
        writeln!(f, "{}- {}", indentation, item)?;
    }
    Ok(())
}

fn strip_prefix(line: &str) -> ParseResult<&str> {
    line.strip_prefix("- ")
        .ok_or_else(|| "list item must start with '- '".into())
        .map(str::trim_start)
}

#[cfg(test)]
mod tests {

    use super::*;

    struct DisplayTest<'a, D> {
        list: &'a List<D>,
        indent_size: usize,
    }

    impl<'a, D> DisplayTest<'a, D> {
        pub fn new(list: &'a List<D>, indent_size: usize) -> Self {
            Self { list, indent_size }
        }
    }

    impl<'a, D: fmt::Display> fmt::Display for DisplayTest<'a, D> {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            self.list.format(f, self.indent_size)
        }
    }

    const SECTIONED_LIST: &str = concat!(
        "section 1\n",
        "- item 1.1\n",
        "- item 1.2\n",
        "section 2\n",
        "- item 2.1\n",
        "- item 2.2\n",
    );

    const TWO_INDENTED_SECTIONED_LIST: &str = concat!(
        "  section 1\n",
        "    - item 1.1\n",
        "    - item 1.2\n",
        "  section 2\n",
        "    - item 2.1\n",
        "    - item 2.2\n",
    );

    #[test]
    fn test_display_basic() {
        let list = List::Basic(vec!["item 1", "item 2"]);
        let test = DisplayTest::new(&list, 0);
        assert_eq!(test.to_string(), "- item 1\n- item 2\n");
        let test = DisplayTest::new(&list, 2);
        assert_eq!(test.to_string(), "  - item 1\n  - item 2\n");
    }

    #[test]
    fn test_display_sectioned() {
        let section1 = Section {
            name: "section 1".into(),
            items: vec!["item 1.1", "item 1.2"],
        };
        let section2 = Section {
            name: "section 2".into(),
            items: vec!["item 2.1", "item 2.2"],
        };
        let list = List::Sectioned(vec![section1, section2]);
        let test = DisplayTest::new(&list, 0);
        assert_eq!(test.to_string(), SECTIONED_LIST);
        let test = DisplayTest::new(&list, 2);
        assert_eq!(test.to_string(), TWO_INDENTED_SECTIONED_LIST);
    }

    #[test]
    fn test_parse_basic() {
        let list: List<String> = vec!["-  item 1".into(), "-  item 2".into()]
            .try_into()
            .unwrap();
        assert!(matches!(list, List::Basic(items) if items == vec!("item 1", "item 2")))
    }

    #[test]
    fn test_parse_basic_fail() {
        assert!(List::<String>::try_from(vec![
            "- item 1".into(),
            "- item 2".into(),
            String::new()
        ])
        .is_err());
    }

    #[test]
    fn test_parse_basic_empty() {
        assert!(List::<String>::try_from(vec![String::new()]).is_err());
    }

    #[test]
    fn test_parse_sectioned() {
        let list: List<String> = SECTIONED_LIST
            .trim()
            .split('\n')
            .map(Into::into)
            .collect::<Vec<_>>()
            .try_into()
            .unwrap();
        assert!(matches!(list, List::Sectioned(_)));
        if let List::Sectioned(sections) = list {
            assert_eq!(sections.len(), 2);
            let section = sections.get(0).unwrap();
            assert_eq!(section.name, "section 1");
            assert_eq!(section.items, vec!("item 1.1", "item 1.2"));
            let section = sections.get(1).unwrap();
            assert_eq!(section.name, "section 2");
            assert_eq!(section.items, vec!("item 2.1", "item 2.2"));
        }
    }
}
