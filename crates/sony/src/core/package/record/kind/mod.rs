pub mod error;

pub(super) use error::Error;

use bytes::Bytes;
use bytes::Integer;
use ddd::{DomainLayer, Layered, ValueObject};

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Copy)]
pub(super) struct Kind {
    bytes: Bytes<4>,
}

impl Layered for Kind {
    type Layer = DomainLayer;
}

impl ValueObject for Kind {}

impl Kind {
    const DIRECTORY: u8 = 4;
    const FILE: u8 = 3;
    const FILES: [u8; 4] = [1, 2, Kind::FILE, 9];

    pub(super) fn bytes(&self) -> Bytes<4> {
        self.bytes
    }

    pub(super) fn is_directory(&self) -> bool {
        Self::letter(self) == Self::DIRECTORY
    }

    fn letter(kind: &Self) -> u8 {
        let bytes = <[u8; 4]>::from(kind.bytes);
        bytes[3]
    }
}

impl Default for Kind {
    fn default() -> Self {
        let bytes = Bytes::from([0, 0, 0, Self::FILE]);
        Self { bytes }
    }
}

impl TryFrom<Bytes<4>> for Kind {
    type Error = Error;

    fn try_from(bytes: Bytes<4>) -> Result<Self, Self::Error> {
        let kind = Self { bytes };
        let letter = Self::letter(&kind);
        let file = Self::FILES.contains(&letter);
        let directory = letter == Self::DIRECTORY;
        if !file && !directory {
            let kind = Integer::from(bytes);
            return Err(Error::Kind { kind });
        }
        Ok(kind)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_kind_of_a_file_a_package_brought_is_no_directory() {
        let bytes = Bytes::from([0x00, 0x00, 0x00, 0x03]);

        let kind = Kind::try_from(bytes).unwrap();

        assert!(!kind.is_directory());
    }

    #[test]
    fn the_kind_a_package_writes_for_a_directory_is_one() {
        let bytes = Bytes::from([0x00, 0x00, 0x00, 0x04]);

        let kind = Kind::try_from(bytes).unwrap();

        assert!(kind.is_directory());
    }

    #[test]
    fn a_kind_keeps_the_flags_the_package_wrote_beside_it() {
        let bytes = Bytes::from([0x80, 0x00, 0x00, 0x03]);

        let kind = Kind::try_from(bytes).unwrap();

        assert_eq!(kind.bytes(), bytes);
    }

    #[test]
    fn a_kind_this_repository_has_never_read_is_an_error() {
        let bytes = Bytes::from([0x00, 0x00, 0x00, 0x07]);

        let response = Kind::try_from(bytes);

        let kind = Integer::from([0x00, 0x00, 0x00, 0x07]);
        assert_eq!(response.unwrap_err(), Error::Kind { kind });
    }
}
