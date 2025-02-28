use moon_common::Id;
use schematic::{derive_enum, ConfigEnum};
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use std::str::FromStr;

#[derive(Clone, ConfigEnum, Debug, Default, Eq, PartialEq)]
pub enum LanguageType {
    Bash,
    Batch,
    Go,
    #[variant(value = "javascript")]
    JavaScript,
    Php,
    Python,
    Ruby,
    Rust,
    #[variant(value = "typescript")]
    TypeScript,

    // Not explicitly set or detected
    #[default]
    Unknown,

    // An unsupported language
    #[variant(fallback)]
    Other(Id),
}

// Required to handle the other and unknown variants
impl<'de> Deserialize<'de> for LanguageType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        match String::deserialize(deserializer) {
            Ok(buffer) => LanguageType::from_str(&buffer).map_err(de::Error::custom),
            Err(error) => {
                // Not aware of another way to handle nulls/undefined
                if error.to_string().contains("invalid type: null") {
                    return Ok(LanguageType::Unknown);
                }

                Err(error)
            }
        }
    }
}

// Required to handle the other variant
impl Serialize for LanguageType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

derive_enum!(
    #[derive(ConfigEnum, Copy, Default, Hash)]
    pub enum PlatformType {
        Bun,
        Deno,
        Node,
        Rust,
        System,
        #[default]
        Unknown,
    }
);

impl PlatformType {
    pub fn is_system(&self) -> bool {
        matches!(self, PlatformType::System)
    }

    pub fn is_unknown(&self) -> bool {
        matches!(self, PlatformType::Unknown)
    }
}

impl From<LanguageType> for PlatformType {
    fn from(language: LanguageType) -> Self {
        match language {
            LanguageType::Unknown => PlatformType::Unknown,
            LanguageType::Bash | LanguageType::Batch => PlatformType::System,
            // Deno and Bun are not covered here!
            LanguageType::JavaScript | LanguageType::TypeScript => PlatformType::Node,
            LanguageType::Rust => PlatformType::Rust,
            // TODO: Move these to their own platform once it's been implemented!
            LanguageType::Go
            | LanguageType::Php
            | LanguageType::Python
            | LanguageType::Ruby
            | LanguageType::Other(_) => PlatformType::System,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converts_lang_to_string() {
        assert_eq!(LanguageType::Go.to_string(), "go");
        assert_eq!(LanguageType::JavaScript.to_string(), "javascript");
        assert_eq!(LanguageType::Ruby.to_string(), "ruby");
        assert_eq!(LanguageType::Unknown.to_string(), "unknown");
        assert_eq!(LanguageType::Other(Id::raw("dotnet")).to_string(), "dotnet");
    }

    #[test]
    fn serializes_lang_to_string() {
        assert_eq!(serde_json::to_string(&LanguageType::Go).unwrap(), "\"go\"");
        assert_eq!(
            serde_json::to_string(&LanguageType::JavaScript).unwrap(),
            "\"javascript\""
        );
        assert_eq!(
            serde_json::to_string(&LanguageType::Ruby).unwrap(),
            "\"ruby\""
        );
        assert_eq!(
            serde_json::to_string(&LanguageType::Unknown).unwrap(),
            "\"unknown\""
        );
        assert_eq!(
            serde_json::to_string(&LanguageType::Other(Id::raw("dotnet"))).unwrap(),
            "\"dotnet\""
        );
    }

    #[test]
    fn deserializes_lang_to_enum() {
        assert_eq!(
            serde_json::from_str::<LanguageType>("\"go\"").unwrap(),
            LanguageType::Go,
        );
        assert_eq!(
            serde_json::from_str::<LanguageType>("\"javascript\"").unwrap(),
            LanguageType::JavaScript,
        );
        assert_eq!(
            serde_json::from_str::<LanguageType>("\"ruby\"").unwrap(),
            LanguageType::Ruby,
        );
        assert_eq!(
            serde_json::from_str::<LanguageType>("\"unknown\"").unwrap(),
            LanguageType::Unknown,
        );
        assert_eq!(
            serde_json::from_str::<LanguageType>("\"dotnet\"").unwrap(),
            LanguageType::Other(Id::raw("dotnet")),
        );
    }
}
