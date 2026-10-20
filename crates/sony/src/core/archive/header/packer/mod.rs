pub(super) mod error;

use ddd::{DomainLayer, Layered, ValueObject};
use serde::de;
use serde::{Deserialize, Deserializer, Serialize};
use std::fmt;

pub(super) use error::Error;

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Copy, Serialize)]
#[serde(transparent)]
pub(super) struct Packer {
    bytes: [u8; Packer::LENGTH],
}

impl Layered for Packer {
    type Layer = DomainLayer;
}

impl ValueObject for Packer {}

impl Packer {
    pub(super) const LENGTH: usize = 4;

    const ZLIB: [u8; Packer::LENGTH] = *b"zlib";

    pub(super) fn bytes(&self) -> &[u8; Self::LENGTH] {
        &self.bytes
    }
}

impl TryFrom<[u8; Packer::LENGTH]> for Packer {
    type Error = Error;

    fn try_from(bytes: [u8; Packer::LENGTH]) -> Result<Self, Self::Error> {
        if bytes != Self::ZLIB {
            let text = String::from_utf8_lossy(&bytes).to_string();
            return Err(Error::Packer { text });
        }
        Ok(Self { bytes })
    }
}

impl Default for Packer {
    fn default() -> Self {
        let bytes = Self::ZLIB;
        Self { bytes }
    }
}

impl fmt::Display for Packer {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let letters = String::from_utf8_lossy(&self.bytes);
        fmt::Display::fmt(&letters, formatter)
    }
}

impl<'bytes> Deserialize<'bytes> for Packer {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'bytes>,
    {
        let bytes = <[u8; Self::LENGTH]>::deserialize(deserializer)?;
        Self::try_from(bytes).map_err(de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::de::IntoDeserializer;
    use serde::de::value;

    #[test]
    fn every_archive_of_this_game_is_packed_with_zlib() {
        let bytes = *b"zlib";

        let packer = Packer::try_from(bytes).unwrap();

        let letters = packer.to_string();
        assert_eq!(letters, "zlib");
    }

    #[test]
    fn an_archive_packed_with_lzma_is_an_error_by_name() {
        let bytes = *b"lzma";

        let packer = Packer::try_from(bytes);

        let text = String::from("lzma");
        assert_eq!(packer.unwrap_err(), Error::Packer { text });
    }

    #[test]
    fn a_packer_of_no_bytes_at_all_is_an_error() {
        let bytes = [0; Packer::LENGTH];

        let packer = Packer::try_from(bytes);

        assert!(packer.is_err());
    }

    #[test]
    fn what_this_crate_packs_with_is_zlib() {
        let packer = Packer::default();

        let bytes = packer.bytes();

        assert_eq!(bytes, b"zlib");
    }

    #[test]
    fn a_packer_comes_back_from_its_four_bytes() {
        let bytes: Vec<u8> = b"zlib".to_vec();
        let deserializer = IntoDeserializer::<value::Error>::into_deserializer(bytes);

        let packer = Packer::deserialize(deserializer).unwrap();

        assert_eq!(packer, Packer::default());
    }

    #[test]
    fn fewer_bytes_than_a_packer_needs_are_no_packer_on_the_way_back() {
        let bytes: Vec<u8> = b"zli".to_vec();
        let deserializer = IntoDeserializer::<value::Error>::into_deserializer(bytes);

        let packer = Packer::deserialize(deserializer);

        assert!(packer.is_err());
    }

    #[test]
    fn a_packer_this_crate_cannot_unpack_is_an_error_on_the_way_back_too() {
        let bytes: Vec<u8> = b"lzma".to_vec();
        let deserializer = IntoDeserializer::<value::Error>::into_deserializer(bytes);

        let packer = Packer::deserialize(deserializer);

        assert!(packer.is_err());
    }
}
