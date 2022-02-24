use std::{collections::HashMap, fmt};

use serde::Serialize;

use super::errors::{ParseError, ParseFromStr, ParseResult};

#[derive(Debug, Serialize)]
pub struct Link {
    pub name: String,
    pub url: String,
}

impl fmt::Display for Link {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} > {}", self.name, self.url)
    }
}

impl ParseFromStr for Link {
    fn parse_from_str(s: &str) -> ParseResult<Self> {
        let (name, url) = s.split_once(" > ").ok_or("missing link separator ' > '")?;
        let name = name.trim_end().to_string();
        let url = url.trim_start().to_string();
        if name.is_empty() {
            return Err(ParseError::empty("link name"));
        }
        if url.is_empty() {
            return Err(ParseError::empty("link url"));
        }
        Ok(Self { name, url })
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Source {
    Author(String),
    Link(Link),
}

impl Source {
    const AUTHOR_KEY: &'static str = "Author";
    const LINK_KEY: &'static str = "Link";
}

impl fmt::Display for Source {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Author(author) => write!(f, "{}: {}", Self::AUTHOR_KEY, author),
            Self::Link(link) => write!(f, "{}: {}", Self::LINK_KEY, link),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct Metadata {
    pub servings: usize,
    pub source: Option<Source>,
    pub tags: Vec<String>,
}

impl Metadata {
    const SERVINGS_KEY: &'static str = "Servings";
    const TAGS_KEY: &'static str = "Tags";
}

impl fmt::Display for Metadata {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{}: {}", Self::SERVINGS_KEY, self.servings)?;
        if let Some(source) = &self.source {
            writeln!(f, "{}", source)?;
        }
        if !self.tags.is_empty() {
            writeln!(f, "{}: {}", Self::TAGS_KEY, self.tags.join(", "))?;
        }
        Ok(())
    }
}

impl TryFrom<Vec<String>> for Metadata {
    type Error = ParseError;

    fn try_from(lines: Vec<String>) -> Result<Self, Self::Error> {
        let mut map = HashMap::new();
        for line in lines {
            let (key, value) = parse_mapping(&line)?;
            if map.insert(key.into(), value.into()).is_some() {
                return Err(format!("duplicate metadata key '{}'", key).into());
            }
        }
        map.try_into()
    }
}

impl TryFrom<HashMap<String, String>> for Metadata {
    type Error = ParseError;

    fn try_from(mut map: HashMap<String, String>) -> Result<Self, Self::Error> {
        let servings = map
            .remove(Self::SERVINGS_KEY)
            .ok_or_else(|| format!("missing metadata key '{}'", Self::SERVINGS_KEY))?
            .parse()
            .map_err(|_| {
                format!(
                    "metadata value for key '{}' must be a number",
                    Self::SERVINGS_KEY
                )
            })?;
        let source = if let Some(value) = map.remove(Source::LINK_KEY) {
            Some(Source::Link(Link::parse_from_str(&value)?))
        } else {
            map.remove(Source::AUTHOR_KEY).map(Source::Author)
        };
        let tags = map.remove(Self::TAGS_KEY).map_or_else(Vec::new, |value| {
            value.split(", ").map(|s| s.trim().into()).collect()
        });
        if let Some(key) = map.keys().next() {
            return Err(format!("unknown metadata key '{}'", key).into());
        }
        let metadata = Self {
            servings,
            source,
            tags,
        };
        Ok(metadata)
    }
}

fn parse_mapping(line: &str) -> ParseResult<(&str, &str)> {
    let (key, value) = line
        .trim()
        .split_once(": ")
        .ok_or("missing key-value separator ': ' in metadata")?;
    let key = key.trim_end();
    let value = value.trim_start();
    let mapping = if key.is_empty() {
        Err(ParseError::empty("metadata key"))
    } else if value.is_empty() {
        Err(ParseError::empty("metadata value"))
    } else {
        Ok((key, value))
    }?;
    Ok(mapping)
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_display_link() {
        let link = Link {
            name: "name".into(),
            url: "url".into(),
        };
        assert_eq!(link.to_string(), "name > url");
    }

    #[test]
    fn test_parse_link() {
        let link = Link::parse_from_str("name > url").unwrap();
        assert_eq!(link.name, "name");
        assert_eq!(link.url, "url");
        let link = Link::parse_from_str("name with space  >  url with space").unwrap();
        assert_eq!(link.name, "name with space");
        assert_eq!(link.url, "url with space");
    }

    #[test]
    fn test_display_source() {
        let link = Link {
            name: "name".into(),
            url: "url".into(),
        };
        let source = Source::Link(link);
        assert_eq!(source.to_string(), "Link: name > url");
        let source = Source::Author("name".into());
        assert_eq!(source.to_string(), "Author: name");
    }

    #[test]
    fn test_metadata() {
        let mut map = HashMap::new();
        map.insert("Servings".into(), "1".into());
        map.insert("Link".into(), "name > url".into());
        map.insert("Tags".into(), "tag1 ,  tag2".into());
        let metadata: Metadata = map.try_into().unwrap();
        assert_eq!(metadata.servings, 1);
        assert!(
            matches!(&metadata.source, Some(Source::Link(link)) if link.name =="name" && link.url == "url")
        );
        assert_eq!(&metadata.tags, &["tag1", "tag2"]);
        assert_eq!(
            metadata.to_string(),
            "Servings: 1\nLink: name > url\nTags: tag1, tag2\n"
        );
    }

    #[test]

    fn test_metadata_empty() {
        assert!(Metadata::try_from(HashMap::new()).is_err());
    }

    #[test]
    fn test_metadata_missing_entries() {
        let mut map = HashMap::new();
        map.insert("Servings".into(), "1".into());
        let metadata: Metadata = map.try_into().unwrap();
        assert_eq!(metadata.servings, 1);
        assert_eq!(metadata.to_string(), "Servings: 1\n");
    }

    #[test]
    fn test_metadata_unkown_entries() {
        let mut map = HashMap::new();
        map.insert("Servings".into(), "1".into());
        map.insert("Unknown".into(), "unknown".into());
        assert!(Metadata::try_from(map).is_err());
    }
}
