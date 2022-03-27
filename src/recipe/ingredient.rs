use std::{fmt, num::IntErrorKind, ops::Not};

use serde::Serialize;

use super::{
    errors::{ParseError, ParseResult},
    ParseFromStr,
};

#[derive(Clone, Copy, Debug, Serialize)]
pub struct Integer(u32);

impl Integer {
    fn try_parse_from_str(s: &str) -> ParseResult<Option<Self>> {
        match s.parse() {
            Ok(int) => Ok(Some(Self(int))),
            Err(error) if *error.kind() != IntErrorKind::PosOverflow => Ok(None),
            Err(_) => Err("integer is out of range".into()),
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize)]
pub struct Decimal {
    int: u16,
    frac: u16,
}

impl Decimal {
    fn try_parse_from_str(s: &str) -> ParseResult<Option<Self>> {
        let (int, frac) = match s.split_once(".") {
            Some(decimal) => decimal,
            None => return Ok(None),
        };
        match (int.parse(), frac.parse()) {
            (Ok(int), Ok(frac)) => Ok(Some(Self { int, frac })),
            (Err(error), _) if *error.kind() != IntErrorKind::PosOverflow => Ok(None),
            (_, Err(error)) if *error.kind() != IntErrorKind::PosOverflow => Ok(None),
            (Err(_), _) => Err("decimal integral part is out of range".into()),
            (_, Err(_)) => Err("decimal fractional part is out of range".into()),
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize)]
pub struct Fraction {
    numer: u8,
    denom: u8,
}

impl Fraction {
    fn add_integer(&self, int: u8) -> Option<Self> {
        Some(Self {
            numer: int
                .checked_mul(self.denom)
                .and_then(|product| product.checked_add(self.numer))?,
            denom: self.denom,
        })
    }

    fn try_parse_from_str(s: &str) -> ParseResult<Option<Self>> {
        let (numer, denom) = match s.split_once("/") {
            Some(fraction) => fraction,
            None => return Ok(None),
        };
        match (numer.parse(), denom.parse()) {
            (Ok(numer), Ok(denom)) => Ok(Some(Self { numer, denom })),
            (Err(error), _) if *error.kind() != IntErrorKind::PosOverflow => Ok(None),
            (_, Err(error)) if *error.kind() != IntErrorKind::PosOverflow => Ok(None),
            (Err(_), _) => Err("fraction numerator is out of range".into()),
            (_, Err(_)) => Err("fraction denominator is out of range".into()),
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum QuantityValue {
    Integer(Integer),
    Decimal(Decimal),
    Fraction(Fraction),
}

impl QuantityValue {
    pub fn parse_from_str(value: &str) -> ParseResult<(Self, &str)> {
        let (value, rest) = value.split_once(' ').unwrap_or((value, ""));
        if let Some(integer) = Integer::try_parse_from_str(value)? {
            match Self::parse_mixed_number(integer, rest)? {
                Some((fraction, rest)) => Ok((Self::Fraction(fraction), rest)),
                None => Ok((Self::Integer(integer), rest)),
            }
        } else if let Some(decimal) = Decimal::try_parse_from_str(value)? {
            Ok((Self::Decimal(decimal), rest))
        } else if let Some(fraction) = Fraction::try_parse_from_str(value)? {
            Ok((Self::Fraction(fraction), rest))
        } else {
            Err(format!("invalid ingredient quantity value: '{}'", value).into())
        }
    }

    fn parse_mixed_number(
        Integer(int): Integer,
        value: &str,
    ) -> ParseResult<Option<(Fraction, &str)>> {
        let (value, rest) = value.split_once(' ').unwrap_or((value, ""));
        let fraction = match Fraction::try_parse_from_str(value)? {
            Some(fraction) => fraction,
            None => return Ok(None),
        };
        let fraction = int
            .try_into()
            .ok()
            .and_then(|int| fraction.add_integer(int))
            .ok_or_else(|| format!("mixed number fraction '{}' is out of range", value))?;
        Ok(Some((fraction, rest)))
    }
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
            Self::Integer(Integer(value)) => {
                write!(f, "{}", value)
            }
            Self::Decimal(Decimal { int, frac }) => {
                write!(f, "{}.{}", int, frac)
            }
            Self::Fraction(Fraction { numer, denom }) => {
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
        let (value, rest) = QuantityValue::parse_from_str(s)?;
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
            value: QuantityValue::Decimal(Decimal { int: 0, frac: 5 }),
            unit: None,
            note: None,
        };
        assert_eq!(quantity.to_string(), "0.5");
        quantity.value = QuantityValue::Fraction(Fraction { numer: 1, denom: 2 });
        assert_eq!(quantity.to_string(), "1/2");
        quantity.value = QuantityValue::Integer(Integer(1));
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
        assert!(matches!(quantity.value, QuantityValue::Integer(Integer(1))));
        assert_eq!(quantity.unit, None);
        assert_eq!(quantity.note, None);
        let quantity = Quantity::parse_from_str("0.5").unwrap();
        assert!(matches!(
            quantity.value,
            QuantityValue::Decimal(Decimal { int: 0, frac: 5 })
        ));
        let quantity = Quantity::parse_from_str("1/2").unwrap();
        assert!(matches!(
            quantity.value,
            QuantityValue::Fraction(Fraction { numer: 1, denom: 2 })
        ));
        let quantity = Quantity::parse_from_str("1  a unit").unwrap();
        assert!(matches!(quantity.value, QuantityValue::Integer(Integer(1))));
        assert_eq!(quantity.unit, Some("a unit".into()));
        assert_eq!(quantity.note, None);
        let quantity = Quantity::parse_from_str("1  ( a note )").unwrap();
        assert!(matches!(quantity.value, QuantityValue::Integer(Integer(1))));
        assert_eq!(quantity.unit, None);
        assert_eq!(quantity.note, Some("a note".into()));
        let quantity = Quantity::parse_from_str("10 1 unit  ( a note )").unwrap();
        assert!(matches!(
            quantity.value,
            QuantityValue::Integer(Integer(10))
        ));
        assert_eq!(quantity.unit, Some("1 unit".into()));
        assert_eq!(quantity.note, Some("a note".into()));
    }

    #[test]
    fn test_display_ingredient() {
        let quantity = Quantity {
            value: QuantityValue::Integer(Integer(1)),
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
        let quantity = ingredient.quantity.unwrap();
        assert_eq!(quantity.value.to_string(), "1");
        assert_eq!(quantity.unit.unwrap(), "unit");
        assert_eq!(quantity.note.unwrap(), "note");
        let ingredient = Ingredient::parse_from_str("name: 0.5 bla (note)").unwrap();
        assert_eq!(ingredient.name, "name");
        assert_eq!(ingredient.kind, None);
        let quantity = ingredient.quantity.unwrap();
        assert_eq!(quantity.value.to_string(), "0.5");
        assert_eq!(quantity.unit.unwrap(), "bla");
        assert_eq!(quantity.note.unwrap(), "note");
        let ingredient = Ingredient::parse_from_str("name: 1/2").unwrap();
        assert_eq!(ingredient.name, "name");
        assert_eq!(ingredient.kind, None);
        assert_eq!(ingredient.quantity.unwrap().to_string(), "1/2");
        let ingredient = Ingredient::parse_from_str("name: 1 1/2").unwrap();
        assert_eq!(ingredient.name, "name");
        assert_eq!(ingredient.kind, None);
        assert_eq!(ingredient.quantity.unwrap().value.to_string(), "3/2");
    }
}
