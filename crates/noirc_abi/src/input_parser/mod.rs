mod json;
mod toml;

use std::{collections::BTreeMap, path::Path};

use acvm::FieldElement;
use serde::Serialize;

use crate::errors::InputParserError;
use crate::{Abi, AbiType};
/// This is what all formats eventually transform into
/// For example, a toml file will parse into TomlTypes
/// and those TomlTypes will be mapped to Value
#[cfg_attr(test, derive(PartialEq))]
#[derive(Debug, Clone, Serialize)]
pub enum InputValue {
    Field(FieldElement),
    Vec(Vec<FieldElement>),
    String(String),
    Struct(BTreeMap<String, InputValue>),
}

impl InputValue {
    /// Checks whether the ABI type matches the InputValue type
    /// and also their arity
    pub fn matches_abi(&self, abi_param: &AbiType) -> bool {
        match (self, abi_param) {
            (InputValue::Field(_), AbiType::Field) => true,
            (InputValue::Field(field_element), AbiType::Integer { width, .. }) => {
                field_element.num_bits() <= *width
            }
            (InputValue::Field(field_element), AbiType::Boolean) => {
                field_element.is_one() || field_element.is_zero()
            }

            (InputValue::Vec(field_elements), AbiType::Array { length, typ, .. }) => {
                if field_elements.len() != *length as usize {
                    return false;
                }
                // Check that all of the array's elements' values match the ABI as well.
                field_elements
                    .iter()
                    .all(|field_element| Self::Field(*field_element).matches_abi(typ))
            }

            (InputValue::String(string), AbiType::String { length }) => {
                string.len() == *length as usize
            }

            (InputValue::Struct(map), AbiType::Struct { fields, .. }) => {
                if map.len() != fields.len() {
                    return false;
                }
                // Check that all of the struct's fields' values match the ABI as well.
                map.iter().all(|(field_name, field_value)| {
                    if let Some(field_type) = fields.get(field_name) {
                        field_value.matches_abi(field_type)
                    } else {
                        false
                    }
                })
            }

            // All other InputValue-AbiType combinations are fundamentally incompatible.
            _ => false,
        }
    }
}

/// Parses the initial Witness Values that are needed to seed the
/// Partial Witness generator
pub trait InitialWitnessParser {
    fn parse_initial_witness<P: AsRef<Path>>(&self, path: P) -> BTreeMap<String, InputValue>;
}

/// The different formats that are supported when parsing
/// the initial witness values
pub enum Format {
    Toml,
    Json,
}

impl Format {
    pub fn ext(&self) -> &'static str {
        match self {
            Format::Toml => "toml",
            Format::Json => "json",
        }
    }
}

impl Format {
    pub fn parse(
        &self,
        input_string: &str,
        abi: &Abi,
    ) -> Result<BTreeMap<String, InputValue>, InputParserError> {
        match self {
            Format::Json => json::parse_json(input_string, abi),
            Format::Toml => toml::parse_toml(input_string, abi),
        }
    }

    pub fn serialize(
        &self,
        w_map: &BTreeMap<String, InputValue>,
    ) -> Result<String, InputParserError> {
        match self {
            Format::Json => json::serialize_to_json(w_map),
            Format::Toml => toml::serialize_to_toml(w_map),
        }
    }
}

pub(self) fn parse_str_to_field(value: &str) -> Result<FieldElement, InputParserError> {
    if value.starts_with("0x") {
        FieldElement::from_hex(value).ok_or_else(|| InputParserError::ParseHexStr(value.to_owned()))
    } else {
        value
            .parse::<i128>()
            .map_err(|err_msg| InputParserError::ParseStr(err_msg.to_string()))
            .map(FieldElement::from)
    }
}

#[cfg(test)]
mod test {
    use super::parse_str_to_field;

    #[test]
    fn parse_empty_str_fails() {
        // Check that this fails appropriately rather than being treated as 0, etc.
        assert!(parse_str_to_field("").is_err());
    }
}
