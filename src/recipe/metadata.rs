use std::{collections::HashMap, fmt};

use serde::Serialize;

use super::{
    errors::{ParseError, ParseResult},
    ParseFromStr,
};

#[derive(Debug, Serialize)]
pub struct Yield {
    pub value: u32,
    pub unit: Option<String>,
}

impl fmt::Display for Yield {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value)?;
        if let Some(unit) = &self.unit {
            write!(f, " {unit}")?;
        }
        Ok(())
    }
}

impl ParseFromStr for Yield {
    fn parse_from_str(s: &str) -> ParseResult<Self> {
        let (value, unit) = s.split_once(' ').map_or((s, None), |(value, unit)| {
            (value, Some(unit.trim_start().into()))
        });
        if let Ok(value) = value.parse() {
            if value == 0 {
                Err(format!(
                    "metadata value for key '{}' must be greater than zero",
                    Metadata::YIELD_KEY,
                )
                .into())
            } else {
                Ok(Yield { value, unit })
            }
        } else {
            Err(format!(
                "metadata value for key '{}' must start with a number",
                Metadata::YIELD_KEY
            )
            .into())
        }
    }
}

#[derive(Debug, Serialize)]
pub struct Duration {
    pub hours: u32,
    pub minutes: u32,
}

impl Duration {
    fn parse_unit(text: &str, unit: &str) -> ParseResult<u32> {
        text.strip_suffix(unit)
            .and_then(|value| value.parse().ok())
            .ok_or_else(|| "invalid recipe duration".into())
    }
}

impl fmt::Display for Duration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut duration: Vec<String> = Vec::new();
        if self.hours > 0 {
            duration.push(self.hours.to_string() + "h");
        }
        if self.minutes > 0 {
            duration.push(self.minutes.to_string() + "m");
        }
        write!(f, "{}", duration.join(" "))
    }
}

impl ParseFromStr for Duration {
    fn parse_from_str(s: &str) -> ParseResult<Self> {
        let (mut hours, mut minutes) = if let Some((h, m)) = s.split_once(' ') {
            (
                Self::parse_unit(h, "h")?,
                Self::parse_unit(m.trim_start(), "m")?,
            )
        } else if let Ok(h) = Self::parse_unit(s, "h") {
            (h, 0)
        } else {
            (0, Self::parse_unit(s, "m")?)
        };
        if hours == 0 && minutes == 0 {
            return Err("recipe duration must be greater than zero".into());
        }
        hours += minutes / 60;
        minutes %= 60;
        Ok(Duration { hours, minutes })
    }
}

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
    Book(String),
    Link(Link),
}

impl Source {
    const AUTHOR_KEY: &'static str = "Author";
    const BOOK_KEY: &'static str = "Book";
    const LINK_KEY: &'static str = "Link";
}

impl fmt::Display for Source {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Author(author) => write!(f, "{}: {}", Self::AUTHOR_KEY, author),
            Self::Book(book) => write!(f, "{}: {}", Self::BOOK_KEY, book),
            Self::Link(link) => write!(f, "{}: {}", Self::LINK_KEY, link),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct Metadata {
    pub duration: Option<Duration>,
    #[serde(rename = "yield")]
    pub yields: Yield,
    pub source: Option<Source>,
    pub tags: Vec<String>,
}

impl Metadata {
    const DURATION_KEY: &'static str = "Time";
    const YIELD_KEY: &'static str = "Yield";
    const TAGS_KEY: &'static str = "Tags";
}

impl fmt::Display for Metadata {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{}: {}", Self::YIELD_KEY, self.yields)?;
        if let Some(duration) = &self.duration {
            writeln!(f, "{}: {}", Self::DURATION_KEY, duration)?;
        }
        if let Some(source) = &self.source {
            writeln!(f, "{source}")?;
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
                return Err(format!("duplicate metadata key '{key}'").into());
            }
        }
        map.try_into()
    }
}

impl TryFrom<HashMap<String, String>> for Metadata {
    type Error = ParseError;

    fn try_from(mut map: HashMap<String, String>) -> Result<Self, Self::Error> {
        let yields = map
            .remove(Self::YIELD_KEY)
            .ok_or_else(|| format!("missing metadata key '{}'", Self::YIELD_KEY))?;
        let yields = Yield::parse_from_str(&yields)?;
        let duration = map
            .remove(Self::DURATION_KEY)
            .as_deref()
            .map(Duration::parse_from_str)
            .transpose()?;
        let source = if let Some(value) = map.remove(Source::LINK_KEY) {
            Some(Source::Link(Link::parse_from_str(&value)?))
        } else {
            map.remove(Source::AUTHOR_KEY)
                .map(Source::Author)
                .or_else(|| map.remove(Source::BOOK_KEY).map(Source::Book))
        };
        let tags = map.remove(Self::TAGS_KEY).map_or_else(Vec::new, |value| {
            value.split(", ").map(|s| s.trim().into()).collect()
        });
        if let Some(key) = map.keys().next() {
            return Err(format!("unknown metadata key '{key}'").into());
        }
        let metadata = Self {
            duration,
            yields,
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
    fn test_display_duration() {
        let mut duration = Duration {
            hours: 1,
            minutes: 0,
        };
        assert_eq!(duration.to_string(), "1h");
        duration.minutes = 30;
        assert_eq!(duration.to_string(), "1h 30m");
        duration.hours = 0;
        assert_eq!(duration.to_string(), "30m");
    }

    #[test]
    fn test_parse_duration() {
        let duration = Duration::parse_from_str("1h").unwrap();
        assert_eq!(duration.hours, 1);
        assert_eq!(duration.minutes, 0);
        let duration = Duration::parse_from_str("1m").unwrap();
        assert_eq!(duration.hours, 0);
        assert_eq!(duration.minutes, 1);
        let duration = Duration::parse_from_str("2h  30m").unwrap();
        assert_eq!(duration.hours, 2);
        assert_eq!(duration.minutes, 30);
        let duration = Duration::parse_from_str("0h  60m").unwrap();
        assert_eq!(duration.hours, 1);
        assert_eq!(duration.minutes, 0);
    }

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
        let source = Source::Book("name".into());
        assert_eq!(source.to_string(), "Book: name");
    }

    #[test]
    fn test_metadata() {
        let mut map = HashMap::new();
        map.insert("Yield".into(), "1  unit".into());
        map.insert("Link".into(), "name> > >url".into());
        map.insert("Tags".into(), "tag1 ,  tag2".into());
        let metadata: Metadata = map.try_into().unwrap();
        assert_eq!(metadata.yields.value, 1);
        assert_eq!(metadata.yields.unit.as_deref(), Some("unit"));
        assert!(
            matches!(&metadata.source, Some(Source::Link(link)) if link.name =="name>" && link.url == ">url")
        );
        assert_eq!(&metadata.tags, &["tag1", "tag2"]);
        assert_eq!(
            metadata.to_string(),
            "Yield: 1 unit\nLink: name> > >url\nTags: tag1, tag2\n"
        );
    }

    #[test]

    fn test_metadata_empty() {
        assert!(Metadata::try_from(HashMap::new()).is_err());
    }

    #[test]
    fn test_metadata_missing_entries() {
        let mut map = HashMap::new();
        map.insert("Yield".into(), "1".into());
        let metadata: Metadata = map.try_into().unwrap();
        assert_eq!(metadata.yields.value, 1);
        assert_eq!(metadata.to_string(), "Yield: 1\n");
    }

    #[test]
    fn test_metadata_unkown_entries() {
        let mut map = HashMap::new();
        map.insert("Yield".into(), "1".into());
        map.insert("Unknown".into(), "unknown".into());
        assert!(Metadata::try_from(map).is_err());
    }
}
