use crate::core::archive::file_path::FilePath;
use bytes::Bytes;
use ddd::{DomainLayer, Layered, ValueObject};
use md5::{Digest as _, Md5};
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(
    Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Copy, Default, Serialize, Deserialize,
)]
#[serde(transparent)]
pub(in crate::core::archive) struct Digest {
    bytes: Bytes<{ Digest::LENGTH }>,
}

impl Layered for Digest {
    type Layer = DomainLayer;
}

impl ValueObject for Digest {}

impl Digest {
    pub(in crate::core::archive) const LENGTH: usize = 16;

    pub(in crate::core::archive) fn bytes(&self) -> Bytes<{ Self::LENGTH }> {
        self.bytes
    }

    pub(in crate::core::archive) fn is_nameless(&self) -> bool {
        self.bytes.is_zero()
    }
}

impl From<&FilePath> for Digest {
    fn from(path: &FilePath) -> Self {
        let mut md5 = Md5::new();
        md5.update(path.letters().as_bytes());
        let bytes: [u8; Self::LENGTH] = md5.finalize().into();
        Self::from(bytes)
    }
}

impl From<[u8; Digest::LENGTH]> for Digest {
    fn from(bytes: [u8; Digest::LENGTH]) -> Self {
        let bytes = Bytes::from(bytes);
        Self { bytes }
    }
}

impl fmt::Display for Digest {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.bytes, formatter)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::de::IntoDeserializer;
    use serde::de::value;

    #[test]
    fn the_digest_of_a_file_path_is_the_one_an_archive_states() {
        let path = FilePath::try_from("net-patch9.bin").unwrap();

        let digest = Digest::from(&path);

        let digest_text = digest.to_string();
        assert_eq!(digest_text, "1AA7B23FF54BF508C1D86EAA35340A14");
    }

    #[test]
    fn the_digest_is_taken_over_all_of_the_path_and_not_over_its_last_step() {
        let path = FilePath::try_from("build/main/net-patch9.bin").unwrap();
        let last_path = FilePath::try_from("net-patch9.bin").unwrap();

        let digest = Digest::from(&path);

        assert_ne!(digest, Digest::from(&last_path));
    }

    #[test]
    fn a_digest_of_sixteen_zero_bytes_is_the_nameless_one() {
        let bytes = [0; Digest::LENGTH];

        let digest = Digest::from(bytes);

        assert!(digest.is_nameless());
    }

    #[test]
    fn the_digest_of_a_file_path_is_no_nameless_one() {
        let path = FilePath::try_from("net-patch9.bin").unwrap();

        let digest = Digest::from(&path);

        assert!(!digest.is_nameless());
    }

    #[test]
    fn a_digest_comes_back_from_its_hexadecimal_text() {
        let deserializer =
            IntoDeserializer::<value::Error>::into_deserializer("1AA7B23FF54BF508C1D86EAA35340A14");

        let digest = Digest::deserialize(deserializer).unwrap();

        let path = FilePath::try_from("net-patch9.bin").unwrap();
        assert_eq!(digest, Digest::from(&path));
    }

    #[test]
    fn a_digest_shows_the_bytes_beneath_it_and_never_builds_that_text_itself() {
        let path = FilePath::try_from("net-patch9.bin").unwrap();

        let digest = Digest::from(&path);

        assert_eq!(digest.to_string(), digest.bytes().to_string());
    }
}
