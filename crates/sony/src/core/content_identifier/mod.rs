pub mod error;

use bytes::byte::Byte;
use ddd::{DomainLayer, Layered, ValueObject};
use std::fmt;

pub use error::Error;

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Copy)]
pub struct ContentIdentifier {
    bytes: [u8; ContentIdentifier::LENGTH],
}

impl Layered for ContentIdentifier {
    type Layer = DomainLayer;
}

impl ValueObject for ContentIdentifier {}

impl ContentIdentifier {
    pub const LENGTH: usize = 48;

    const FIRST_LETTER: u8 = 0x20;
    const LAST_LETTER: u8 = 0x7E;

    pub fn bytes(&self) -> &[u8; Self::LENGTH] {
        &self.bytes
    }
}

impl Default for ContentIdentifier {
    fn default() -> Self {
        let bytes = [0; Self::LENGTH];
        Self { bytes }
    }
}

impl TryFrom<[u8; ContentIdentifier::LENGTH]> for ContentIdentifier {
    type Error = Error;

    fn try_from(bytes: [u8; ContentIdentifier::LENGTH]) -> Result<Self, Self::Error> {
        for byte in bytes {
            let zero = byte == 0;
            let letters = Self::FIRST_LETTER..=Self::LAST_LETTER;
            let letter = letters.contains(&byte);
            if !zero && !letter {
                let byte = Byte::from(byte);
                return Err(Error::Letter { byte });
            }
        }
        Ok(Self { bytes })
    }
}

impl fmt::Display for ContentIdentifier {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let index = self.bytes.iter().position(|byte| *byte == 0);
        let index = index.unwrap_or(Self::LENGTH);
        let letters = self.bytes.get(..index).unwrap_or_default();
        let letters = String::from_utf8_lossy(letters);
        fmt::Display::fmt(&letters, formatter)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_content_identifier_of_the_golden_weapons_pack_is_shown_as_its_letters() {
        let mut bytes = [0; ContentIdentifier::LENGTH];
        let letters = b"EP0000-XBRK00000_00-XBREAKTESTLABEL0";
        bytes[..letters.len()].copy_from_slice(letters);

        let content_identifier = ContentIdentifier::try_from(bytes).unwrap();

        let text = content_identifier.to_string();
        assert_eq!(text, "EP0000-XBRK00000_00-XBREAKTESTLABEL0");
    }

    #[test]
    fn a_content_identifier_of_nothing_but_zeroes_is_shown_as_no_letter_at_all() {
        let bytes = [0; ContentIdentifier::LENGTH];

        let content_identifier = ContentIdentifier::try_from(bytes).unwrap();

        let text = content_identifier.to_string();
        assert_eq!(text, "");
    }

    #[test]
    fn a_content_identifier_with_a_byte_that_is_no_letter_is_an_error() {
        let mut bytes = [0; ContentIdentifier::LENGTH];
        bytes[0] = 0xFF;

        let content_identifier = ContentIdentifier::try_from(bytes);

        let byte = Byte::from(0xFF);
        assert_eq!(content_identifier.unwrap_err(), Error::Letter { byte });
    }

    #[test]
    fn a_content_identifier_keeps_every_byte_it_was_read_from() {
        let mut bytes = [0; ContentIdentifier::LENGTH];
        bytes[0] = b'E';

        let content_identifier = ContentIdentifier::try_from(bytes).unwrap();

        assert_eq!(content_identifier.bytes(), &bytes);
    }
}
