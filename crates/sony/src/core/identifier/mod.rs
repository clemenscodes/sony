pub mod error;

use crate::core::name::Name;
use bytes::Bytes;
use ddd::{DomainLayer, Layered, ValueObject};
use serde::de;
use serde::{Deserialize, Deserializer, Serialize};
use sha1::{Digest, Sha1};
use std::fmt;

pub use error::Error;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize)]
#[serde(transparent)]
pub struct Identifier {
    checksum: Bytes<{ Identifier::BYTES }>,
}

impl Layered for Identifier {
    type Layer = DomainLayer;
}

impl ValueObject for Identifier {}

impl Identifier {
    const SUFFIX: [u8; 16] = [
        0x67, 0x59, 0x65, 0x99, 0x04, 0x25, 0x04, 0x90, 0x56, 0x64, 0x27, 0x49, 0x94, 0x89, 0x74,
        0x1A,
    ];
    const LETTERS: usize = 8;
    const BYTES: usize = 4;
}

impl From<&Name> for Identifier {
    fn from(name: &Name) -> Self {
        let mut sha1 = Sha1::new();
        sha1.update(name.text().as_bytes());
        sha1.update(Self::SUFFIX);
        let digest = sha1.finalize();
        let mut bytes = [0; Self::BYTES];
        bytes.copy_from_slice(&digest[..Self::BYTES]);
        let checksum = Bytes::from(u32::from_le_bytes(bytes).to_be_bytes());
        Self { checksum }
    }
}

impl TryFrom<&str> for Identifier {
    type Error = Error;

    fn try_from(identifier: &str) -> Result<Self, Self::Error> {
        let length_fit = identifier.len() == Self::LETTERS;
        let hexadecimal_fit = identifier.chars().all(|letter| letter.is_ascii_hexdigit());
        let uppercase_fit = identifier
            .chars()
            .all(|letter| !letter.is_ascii_lowercase());
        let text = identifier.to_string();
        if !length_fit || !hexadecimal_fit || !uppercase_fit {
            return Err(Error::Identifier { text });
        }
        let mut checksum: u32 = 0;
        for letter in identifier.bytes() {
            let hexadecimal = match letter < 0x3A {
                true => u32::from(letter) - 0x30,
                false => u32::from(letter) - 0x37,
            };
            checksum = (checksum << 4) | hexadecimal;
        }
        let checksum = Bytes::from(checksum.to_be_bytes());
        Ok(Self { checksum })
    }
}

impl fmt::Display for Identifier {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.checksum, formatter)
    }
}

impl<'letters> Deserialize<'letters> for Identifier {
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
    fn letters_that_are_no_identifier_are_shown_as_the_letters_they_were() {
        let letters = "zzzzzzzz";

        let identifier = Identifier::try_from(letters);

        let text = String::from("zzzzzzzz");
        assert_eq!(identifier.unwrap_err(), Error::Identifier { text });
    }

    #[test]
    fn an_identifier_comes_back_from_the_letters_it_is_kept_as() {
        let identifier = Identifier::try_from("32FAAF58").unwrap();

        let tokens = [serde_test::Token::Str("32FAAF58")];

        serde_test::assert_de_tokens(&identifier, &tokens);
    }

    #[test]
    fn a_number_where_an_identifier_belongs_comes_back_as_no_identifier() {
        let tokens = [serde_test::Token::U32(7)];

        serde_test::assert_de_tokens_error::<Identifier>(
            &tokens,
            "invalid type: integer `7`, expected a string",
        );
    }

    #[test]
    fn an_identifier_that_is_no_text_at_all_comes_back_as_no_identifier() {
        let deserializer = IntoDeserializer::<value::Error>::into_deserializer(7);

        let identifier = Identifier::deserialize(deserializer);

        assert!(identifier.is_err());
    }

    #[test]
    fn a_name_yields_the_checksum_the_disc_writes_for_it() {
        let name = Name::try_from("cellHttpUtilParseUri").unwrap();

        let identifier = Identifier::from(&name);

        let identifier_text = identifier.to_string();
        assert_eq!(identifier_text, "32FAAF58");
    }

    #[test]
    fn a_second_name_yields_the_second_checksum_the_disc_writes() {
        let name = Name::try_from("cellHttpUtilBase64Encoder").unwrap();

        let identifier = Identifier::from(&name);

        let identifier_text = identifier.to_string();
        assert_eq!(identifier_text, "83FAA354");
    }

    #[test]
    fn a_socket_call_yields_its_checksum_from_the_bare_name() {
        let name = Name::try_from("sendto").unwrap();

        let identifier = Identifier::from(&name);

        let identifier_text = identifier.to_string();
        assert_eq!(identifier_text, "9647570B");
    }

    #[test]
    fn one_name_yields_the_same_checksum_every_time() {
        let name = Name::try_from("sceNpMatching2SearchRoom").unwrap();

        let identifier = Identifier::from(&name);

        let same_identifier = Identifier::from(&name);
        assert_eq!(identifier, same_identifier);
    }

    #[test]
    fn a_checksum_in_hexadecimal_comes_back_as_the_text_it_came_from() {
        let text = "32FAAF58";

        let identifier = Identifier::try_from(text).unwrap();

        let identifier_text = identifier.to_string();
        assert_eq!(identifier_text, text);
    }

    #[test]
    fn a_checksum_that_leads_with_a_zero_keeps_its_width() {
        let text = "00F0A1B2";

        let identifier = Identifier::try_from(text).unwrap();

        let identifier_text = identifier.to_string();
        assert_eq!(identifier_text, text);
    }

    #[test]
    fn a_checksum_in_lower_case_is_no_identifier() {
        let text = "32faaf58";

        let identifier = Identifier::try_from(text);

        assert!(identifier.is_err());
    }

    #[test]
    fn a_checksum_of_another_width_is_no_identifier() {
        let text = "32FAAF5";

        let identifier = Identifier::try_from(text);

        assert!(identifier.is_err());
    }

    #[test]
    fn a_writing_that_is_no_hexadecimal_is_no_identifier() {
        let text = "32FAAF5Z";

        let identifier = Identifier::try_from(text);

        let error = Error::Identifier {
            text: String::from("32FAAF5Z"),
        };
        assert_eq!(identifier.unwrap_err(), error);
    }

    #[test]
    fn the_checksum_a_name_yields_is_the_checksum_that_writing_reads_back() {
        let name = Name::try_from("sceNpTrophyUnlockTrophy").unwrap();

        let identifier = Identifier::from(&name);

        let identifier_text = identifier.to_string();
        let same_identifier = Identifier::try_from(identifier_text.as_str()).unwrap();
        assert_eq!(same_identifier, identifier);
    }

    #[test]
    fn a_checksum_in_lower_case_is_no_identifier_on_the_way_back_either() {
        let deserializer = IntoDeserializer::<value::Error>::into_deserializer("32faaf58");

        let identifier = Identifier::deserialize(deserializer);

        assert!(identifier.is_err());
    }

    #[test]
    fn an_identifier_comes_back_from_its_hexadecimal_text() {
        let deserializer = IntoDeserializer::<value::Error>::into_deserializer("32FAAF58");

        let identifier = Identifier::deserialize(deserializer).unwrap();

        assert_eq!(identifier, Identifier::try_from("32FAAF58").unwrap());
    }
}
