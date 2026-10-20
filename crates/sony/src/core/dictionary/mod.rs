mod entry;
pub mod error;

use crate::core::dictionary::entry::Entry;
use crate::core::identifier::Identifier;
use crate::core::name::Name;
use ddd::{DomainLayer, Layered, ValueObject};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fmt;

pub use error::Error;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(into = "Vec<Entry>", try_from = "Vec<Entry>")]
pub struct Dictionary {
    entries: BTreeMap<Identifier, Name>,
}

impl Layered for Dictionary {
    type Layer = DomainLayer;
}

impl ValueObject for Dictionary {}

impl Dictionary {
    const NAMES: &'static str = include_str!("../../../assets/names.txt");

    pub fn try_resolve(&self, identifier: &Identifier) -> Result<&Name, Error> {
        let Some(name) = self.entries.get(identifier) else {
            let identifier = identifier.to_string();
            return Err(Error::Unknown { identifier });
        };
        Ok(name)
    }
}

impl TryFrom<()> for Dictionary {
    type Error = Error;

    fn try_from(_: ()) -> Result<Self, Self::Error> {
        Self::try_from(Self::NAMES)
    }
}

impl TryFrom<&str> for Dictionary {
    type Error = Error;

    fn try_from(text: &str) -> Result<Self, Self::Error> {
        let mut read_entries = Vec::new();
        for line in text.lines() {
            let text = line.trim();
            if text.is_empty() {
                continue;
            }
            let entry = Entry::try_from(text)?;
            read_entries.push(entry);
        }
        Self::try_from(read_entries)
    }
}

impl TryFrom<Vec<Entry>> for Dictionary {
    type Error = Error;

    fn try_from(read_entries: Vec<Entry>) -> Result<Self, Self::Error> {
        let mut entries: BTreeMap<Identifier, Name> = BTreeMap::new();
        for entry in read_entries {
            let Some(name) = entries.insert(entry.identifier(), entry.name().clone()) else {
                continue;
            };
            if &name == entry.name() {
                continue;
            }
            let identifier = entry.identifier().to_string();
            let name = name.to_string();
            let entry_name = entry.name().to_string();
            return Err(Error::Shared {
                identifier,
                name,
                entry_name,
            });
        }
        if entries.is_empty() {
            return Err(Error::Nameless);
        }
        Ok(Self { entries })
    }
}

impl From<Dictionary> for Vec<Entry> {
    fn from(dictionary: Dictionary) -> Self {
        dictionary.entries.into_values().map(Entry::from).collect()
    }
}

impl fmt::Display for Dictionary {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut text = String::new();
        for (identifier, name) in &self.entries {
            let line = format!("{identifier} {name}\n");
            text.push_str(&line);
        }
        fmt::Display::fmt(&text, formatter)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::name;
    use serde_test::{Token, assert_de_tokens_error, assert_tokens};

    #[test]
    fn two_names_of_one_identifier_are_an_error_and_both_are_named() {
        let text = "eraz\ngqnr";

        let dictionary = Dictionary::try_from(text);

        let identifier = String::from("E088B021");
        let name = String::from("eraz");
        let entry_name = String::from("gqnr");
        assert_eq!(
            dictionary.unwrap_err(),
            Error::Shared {
                identifier,
                name,
                entry_name
            }
        );
    }

    #[test]
    fn a_line_that_is_no_name_yields_no_dictionary() {
        let text = "sendto\ncell Http";

        let dictionary = Dictionary::try_from(text);

        let text = String::from("cell Http");
        let name = name::Error::Name { text };
        assert_eq!(dictionary.unwrap_err(), Error::Name { name });
    }

    #[test]
    fn a_dictionary_shows_one_identifier_and_one_name_to_a_line() {
        let text = "sendto\nrecvfrom";

        let dictionary = Dictionary::try_from(text).unwrap();

        let dictionary_text = dictionary.to_string();
        assert_eq!(dictionary_text, "1F953B9F recvfrom\n9647570B sendto\n");
    }

    #[test]
    fn a_dictionary_writes_one_line_for_every_name_it_keeps() {
        let dictionary = Dictionary::try_from(()).unwrap();

        let dictionary_text = dictionary.to_string();

        assert_eq!(dictionary_text.lines().count(), 3792);
    }

    #[test]
    fn the_dictionary_is_the_names_beside_it() {
        let dictionary = Dictionary::try_from(()).unwrap();

        let count = dictionary.entries.len();

        assert_eq!(count, 3792);
    }

    #[test]
    fn a_text_without_a_name_yields_no_dictionary() {
        let text = "";

        let dictionary = Dictionary::try_from(text);

        assert_eq!(dictionary.unwrap_err(), Error::Nameless);
    }

    #[test]
    fn a_text_of_blank_lines_alone_yields_no_dictionary() {
        let text = "\n   \n\n";

        let dictionary = Dictionary::try_from(text);

        assert_eq!(dictionary.unwrap_err(), Error::Nameless);
    }

    #[test]
    fn one_name_that_comes_twice_is_one_entry_and_no_conflict() {
        let text = "sendto\nsendto";

        let dictionary = Dictionary::try_from(text).unwrap();

        let count = dictionary.entries.len();
        assert_eq!(count, 1);
    }

    #[test]
    fn no_two_names_yield_one_identifier() {
        let dictionary = Dictionary::try_from(());

        let fit = dictionary.is_ok();

        assert!(fit);
    }

    #[test]
    fn an_identifier_the_disc_writes_is_resolved_to_the_name_it_means() {
        let dictionary = Dictionary::try_from(()).unwrap();
        let identifier = Identifier::try_from("32FAAF58").unwrap();

        let name = dictionary.try_resolve(&identifier).unwrap();

        let name_text = name.to_string();
        assert_eq!(name_text, "cellHttpUtilParseUri");
    }

    #[test]
    fn a_socket_call_is_resolved_from_the_identifier_the_disc_writes() {
        let dictionary = Dictionary::try_from(()).unwrap();
        let identifier = Identifier::try_from("9647570B").unwrap();

        let name = dictionary.try_resolve(&identifier).unwrap();

        let name_text = name.to_string();
        assert_eq!(name_text, "sendto");
    }

    #[test]
    fn an_identifier_no_name_yields_is_an_error_and_named() {
        let dictionary = Dictionary::try_from(()).unwrap();
        let identifier = Identifier::try_from("DEADBEEF").unwrap();

        let name = dictionary.try_resolve(&identifier);

        let error = Error::Unknown {
            identifier: String::from("DEADBEEF"),
        };
        assert_eq!(name.unwrap_err(), error);
    }

    #[test]
    fn every_name_in_the_dictionary_yields_the_identifier_it_is_kept_under() {
        let dictionary = Dictionary::try_from(()).unwrap();

        let entries = dictionary.entries.clone();

        for (identifier, name) in entries {
            assert_eq!(Identifier::from(&name), identifier);
        }
    }

    #[test]
    fn the_dictionary_is_the_same_every_time() {
        let dictionary = Dictionary::try_from(()).unwrap();

        let same_dictionary = Dictionary::try_from(()).unwrap();

        assert_eq!(dictionary, same_dictionary);
    }

    #[test]
    fn a_dictionary_goes_out_and_comes_back_as_the_names_it_keeps() {
        let dictionary = Dictionary::try_from("sendto\nrecvfrom").unwrap();

        let tokens = [
            Token::Seq { len: Some(2) },
            Token::Str("recvfrom"),
            Token::Str("sendto"),
            Token::SeqEnd,
        ];

        assert_tokens(&dictionary, &tokens);
    }

    #[test]
    fn a_text_without_a_name_yields_no_dictionary_on_the_way_back_either() {
        let tokens = [Token::Seq { len: Some(0) }, Token::SeqEnd];

        let error_text = Error::Nameless.to_string();

        assert_de_tokens_error::<Dictionary>(&tokens, &error_text);
    }

    #[test]
    fn a_writing_that_is_no_name_yields_no_dictionary_on_the_way_back_either() {
        let text = String::from("cell Http");
        let tokens = [Token::Seq { len: Some(1) }, Token::Str("cell Http")];

        let error_text = name::Error::Name { text }.to_string();

        assert_de_tokens_error::<Dictionary>(&tokens, &error_text);
    }
}
