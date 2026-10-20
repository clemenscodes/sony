pub mod error;

use ddd::{DomainLayer, Layered, ValueObject};
use serde::de;
use serde::{Deserialize, Deserializer, Serialize};
use std::fmt;

pub use error::Error;

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize)]
#[serde(transparent)]
pub struct Name {
    text: String,
}

impl Layered for Name {
    type Layer = DomainLayer;
}

impl ValueObject for Name {}

impl Name {
    pub fn text(&self) -> &str {
        &self.text
    }
}

impl TryFrom<&str> for Name {
    type Error = Error;

    fn try_from(name: &str) -> Result<Self, Self::Error> {
        let letters = name
            .chars()
            .all(|letter| letter.is_ascii_alphanumeric() || letter == '_');
        let text = name.to_string();
        if name.is_empty() || !letters {
            return Err(Error::Name { text });
        }
        Ok(Self { text })
    }
}

impl fmt::Display for Name {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.text, formatter)
    }
}

impl<'letters> Deserialize<'letters> for Name {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'letters>,
    {
        let text = String::deserialize(deserializer)?;
        Self::try_from(text.as_str()).map_err(de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::de::IntoDeserializer;
    use serde::de::value;

    #[test]
    fn a_name_comes_back_from_the_letters_it_is_kept_as() {
        let name = Name::try_from("sendto").unwrap();

        let tokens = [serde_test::Token::Str("sendto")];

        serde_test::assert_de_tokens(&name, &tokens);
    }

    #[test]
    fn a_number_where_a_name_belongs_comes_back_as_no_name() {
        let tokens = [serde_test::Token::U32(7)];

        serde_test::assert_de_tokens_error::<Name>(
            &tokens,
            "invalid type: integer `7`, expected a string",
        );
    }

    #[test]
    fn a_name_that_is_no_text_at_all_comes_back_as_no_name() {
        let deserializer = IntoDeserializer::<value::Error>::into_deserializer(7);

        let name = Name::deserialize(deserializer);

        assert!(name.is_err());
    }

    #[test]
    fn a_name_is_kept_exactly_as_the_text_it_came_from() {
        let text = "cellHttpUtilParseUri";

        let name = Name::try_from(text).unwrap();

        let name_text = name.to_string();
        assert_eq!(name_text, "cellHttpUtilParseUri");
    }

    #[test]
    fn a_name_may_open_with_an_underscore() {
        let text = "_cellSpursWorkloadFlagReceiver";

        let name = Name::try_from(text).unwrap();

        let name_text = name.text();
        assert_eq!(name_text, "_cellSpursWorkloadFlagReceiver");
    }

    #[test]
    fn an_empty_writing_is_no_name() {
        let text = "";

        let name = Name::try_from(text);

        let error = Error::Name {
            text: String::new(),
        };
        assert_eq!(name.unwrap_err(), error);
    }

    #[test]
    fn a_writing_with_a_space_is_no_name() {
        let text = "cell Http";

        let name = Name::try_from(text);

        assert!(name.is_err());
    }

    #[test]
    fn two_names_of_the_same_letters_are_the_same_name() {
        let text = "sceNpLookupInit";

        let name = Name::try_from(text).unwrap();

        let same_name = Name::try_from(text).unwrap();
        assert_eq!(name, same_name);
    }

    #[test]
    fn a_name_in_another_case_is_another_name() {
        let camel_name = Name::try_from("sceNpLookupInit").unwrap();

        let lower_name = Name::try_from("scenplookupinit").unwrap();

        assert_ne!(camel_name, lower_name);
    }

    #[test]
    fn a_name_comes_back_from_the_text_it_goes_out_as() {
        let deserializer =
            IntoDeserializer::<value::Error>::into_deserializer("cellHttpUtilParseUri");

        let name = Name::deserialize(deserializer).unwrap();

        assert_eq!(name, Name::try_from("cellHttpUtilParseUri").unwrap());
    }

    #[test]
    fn an_empty_writing_is_no_name_on_the_way_back_either() {
        let deserializer = IntoDeserializer::<value::Error>::into_deserializer("");

        let name = Name::deserialize(deserializer);

        assert!(name.is_err());
    }
}
