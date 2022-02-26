use std::{fmt, ops::Not};

use serde::Serialize;

use super::{
    errors::{ParseError, ParseResult},
    ParseFromStr,
};

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum QuantityValue {
    Integer(u32),
    Decimal { int: u32, frac: u32 },
    Fraction { numer: u8, denom: u8 },
}

#[derive(Debug, Serialize)]
pub struct Quantity {
    pub value: QuantityValue,
    pub unit: Option<String>,
    pub note: Option<String>,
}

impl fmt::Display for QuantityValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Integer(value) => {
                write!(f, "{}", value)
            }
            Self::Decimal { int, frac } => {
                write!(f, "{}.{}", int, frac)
            }
            Self::Fraction { numer, denom } => {
                write!(f, "{}/{}", numer, denom)
            }
        }
    }
}

impl fmt::Display for Quantity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value)?;
        if let Some(unit) = &self.unit {
            write!(f, " {}", unit)?;
        }
        if let Some(note) = &self.note {
            write!(f, " ({})", note)?;
        }
        Ok(())
    }
}

impl ParseFromStr for Quantity {
    fn parse_from_str(s: &str) -> ParseResult<Self> {
        let (value, rest) = s
            .split_once(" ")
            .map_or((s, ""), |(value, rest)| (value, rest));
        let value = if let Some((int, frac)) = value.split_once('.') {
            if let (Ok(int), Ok(frac)) = (int.parse(), frac.parse()) {
                Some(QuantityValue::Decimal { int, frac })
            } else {
                None
            }
        } else if let Some((numer, denom)) = value.split_once('/') {
            if let (Ok(numer), Ok(denom)) = (numer.parse(), denom.parse()) {
                Some(QuantityValue::Fraction { numer, denom })
            } else {
                None
            }
        } else if let Ok(value) = value.parse() {
            Some(QuantityValue::Integer(value))
        } else {
            None
        }
        .ok_or_else(|| {
            ParseError::from("ingredient quantity must start with a number or fraction")
        })?;
        let (unit, note) = if let Some((unit, note)) = rest.split_once(" (") {
            let unit = unit.trim().to_string();
            let note = note
                .strip_suffix(')')
                .ok_or("missing closing parenthesis of quantity note")?
                .trim();
            (unit, note.is_empty().not().then(|| note.into()))
        } else {
            (rest.trim_start().into(), None)
        };
        let unit = unit.is_empty().not().then(|| unit);
        Ok(Self { value, unit, note })
    }
}

#[derive(Debug, Serialize)]
pub struct Ingredient {
    pub name: String,
    pub kind: Option<String>,
    pub quantity: Option<Quantity>,
}

impl fmt::Display for Ingredient {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)?;
        if let Some(kind) = &self.kind {
            write!(f, ", {}", kind)?;
        }
        if let Some(quantity) = &self.quantity {
            write!(f, ": {}", quantity)?;
        }
        Ok(())
    }
}

impl ParseFromStr for Ingredient {
    fn parse_from_str(s: &str) -> ParseResult<Self> {
        let (name, quantity) = s.split_once(": ").map_or((s, ""), |(name, quantity)| {
            (name.trim_end(), quantity.trim_start())
        });
        let (name, kind) = name.split_once(", ").map_or((name, ""), |(name, kind)| {
            (name.trim_end(), kind.trim_start())
        });
        if name.is_empty() {
            return Err(ParseError::empty("ingredient name"));
        }
        let kind = kind.is_empty().not().then(|| kind.into());
        let quantity = if quantity.is_empty() {
            None
        } else {
            Some(Quantity::parse_from_str(quantity)?)
        };
        Ok(Self {
            name: name.into(),
            kind,
            quantity,
        })
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_display_quantity() {
        let mut quantity = Quantity {
            value: QuantityValue::Decimal { int: 0, frac: 5 },
            unit: None,
            note: None,
        };
        assert_eq!(quantity.to_string(), "0.5");
        quantity.value = QuantityValue::Fraction { numer: 1, denom: 2 };
        assert_eq!(quantity.to_string(), "1/2");
        quantity.value = QuantityValue::Integer(1);
        assert_eq!(quantity.to_string(), "1");
        quantity.unit = Some("unit".into());
        assert_eq!(quantity.to_string(), "1 unit");
        quantity.note = Some("note".into());
        assert_eq!(quantity.to_string(), "1 unit (note)");
        quantity.unit = None;
        assert_eq!(quantity.to_string(), "1 (note)");
    }

    #[test]
    fn test_parse_quantity() {
        let quantity = Quantity::parse_from_str("1").unwrap();
        assert!(matches!(quantity.value, QuantityValue::Integer(1)));
        assert_eq!(quantity.unit, None);
        assert_eq!(quantity.note, None);
        let quantity = Quantity::parse_from_str("0.5").unwrap();
        assert!(matches!(
            quantity.value,
            QuantityValue::Decimal { int: 0, frac: 5 }
        ));
        let quantity = Quantity::parse_from_str("1/2").unwrap();
        assert!(matches!(
            quantity.value,
            QuantityValue::Fraction { numer: 1, denom: 2 }
        ));
        let quantity = Quantity::parse_from_str("1  a unit").unwrap();
        assert!(matches!(quantity.value, QuantityValue::Integer(1)));
        assert_eq!(quantity.unit, Some("a unit".into()));
        assert_eq!(quantity.note, None);
        let quantity = Quantity::parse_from_str("1  ( a note )").unwrap();
        assert!(matches!(quantity.value, QuantityValue::Integer(1)));
        assert_eq!(quantity.unit, None);
        assert_eq!(quantity.note, Some("a note".into()));
        let quantity = Quantity::parse_from_str("10 1 unit  ( a note )").unwrap();
        assert!(matches!(quantity.value, QuantityValue::Integer(10)));
        assert_eq!(quantity.unit, Some("1 unit".into()));
        assert_eq!(quantity.note, Some("a note".into()));
    }

    #[test]
    fn test_display_ingredient() {
        let quantity = Quantity {
            value: QuantityValue::Integer(1),
            unit: None,
            note: None,
        };
        let mut ingredient = Ingredient {
            name: "name".into(),
            kind: None,
            quantity: None,
        };
        assert_eq!(ingredient.to_string(), "name");
        ingredient.kind = Some("kind".into());
        assert_eq!(ingredient.to_string(), "name, kind");
        ingredient.quantity = Some(quantity);
        assert_eq!(ingredient.to_string(), "name, kind: 1");
        ingredient.kind = None;
        assert_eq!(ingredient.to_string(), "name: 1");
    }

    #[test]
    fn test_parse_ingredient() {
        let ingredient = Ingredient::parse_from_str("a name").unwrap();
        assert_eq!(ingredient.name, "a name");
        assert_eq!(ingredient.kind, None);
        assert!(ingredient.quantity.is_none());
        let ingredient = Ingredient::parse_from_str("a name ,  a kind").unwrap();
        assert_eq!(ingredient.name, "a name");
        assert_eq!(ingredient.kind, Some("a kind".into()));
        assert!(ingredient.quantity.is_none());
        let ingredient = Ingredient::parse_from_str("a name :  1 unit (note)").unwrap();
        assert_eq!(ingredient.name, "a name");
        assert_eq!(ingredient.kind, None);
        assert_eq!(ingredient.quantity.unwrap().to_string(), "1 unit (note)");
        let ingredient = Ingredient::parse_from_str("a name ,  a kind :  1 unit (note)").unwrap();
        assert_eq!(ingredient.name, "a name");
        assert_eq!(ingredient.kind, Some("a kind".into()));
        assert_eq!(ingredient.quantity.unwrap().to_string(), "1 unit (note)");
    }
}
