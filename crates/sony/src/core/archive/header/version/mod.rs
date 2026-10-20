pub(in crate::core::archive) mod error;

use bytes::Integer;
use ddd::{DomainLayer, Layered, ValueObject};
use serde::{Deserialize, Serialize};
use std::fmt;

pub(in crate::core::archive) use error::Error;

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Copy, Serialize, Deserialize)]
#[serde(
    into = "(Integer<{ Version::FIELD_LENGTH }>, Integer<{ Version::FIELD_LENGTH }>)",
    try_from = "(Integer<{ Version::FIELD_LENGTH }>, Integer<{ Version::FIELD_LENGTH }>)"
)]
pub(in crate::core::archive) struct Version {
    major: Integer<{ Version::FIELD_LENGTH }>,
    minor: Integer<{ Version::FIELD_LENGTH }>,
}

impl Layered for Version {
    type Layer = DomainLayer;
}

impl ValueObject for Version {}

impl Version {
    pub(in crate::core::archive) const FIELD_LENGTH: usize = 2;

    const MAJOR_VERSION: [u8; Version::FIELD_LENGTH] = [0, 1];
    const DISC: [u8; Version::FIELD_LENGTH] = [0, 3];
    const STORE: [u8; Version::FIELD_LENGTH] = [0, 4];

    pub(in crate::core::archive) fn major(&self) -> Integer<{ Self::FIELD_LENGTH }> {
        self.major
    }

    pub(in crate::core::archive) fn minor(&self) -> Integer<{ Self::FIELD_LENGTH }> {
        self.minor
    }
}

impl
    TryFrom<(
        Integer<{ Version::FIELD_LENGTH }>,
        Integer<{ Version::FIELD_LENGTH }>,
    )> for Version
{
    type Error = Error;

    fn try_from(
        (major, minor): (
            Integer<{ Version::FIELD_LENGTH }>,
            Integer<{ Version::FIELD_LENGTH }>,
        ),
    ) -> Result<Self, Self::Error> {
        let disc = Integer::from(Self::DISC);
        let store = Integer::from(Self::STORE);
        let minor_fit = minor == disc || minor == store;
        let fit = major == Integer::from(Self::MAJOR_VERSION) && minor_fit;
        if !fit {
            return Err(Error::ArchiveVersion { major, minor });
        }
        Ok(Self { major, minor })
    }
}

impl From<Version>
    for (
        Integer<{ Version::FIELD_LENGTH }>,
        Integer<{ Version::FIELD_LENGTH }>,
    )
{
    fn from(version: Version) -> Self {
        (version.major, version.minor)
    }
}

impl Default for Version {
    fn default() -> Self {
        let major = Integer::from(Self::MAJOR_VERSION);
        let minor = Integer::from(Self::STORE);
        Self { major, minor }
    }
}

impl fmt::Display for Version {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let major = u16::from(self.major);
        let minor = u16::from(self.minor);
        write!(formatter, "{major}.{minor}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::de::IntoDeserializer;
    use serde::de::value;

    #[test]
    fn the_archives_of_the_disc_are_of_the_third_version() {
        let version = Version::try_from((Integer::from([0, 1]), Integer::from([0, 3]))).unwrap();

        let version_text = version.to_string();

        assert_eq!(version_text, "1.3");
    }

    #[test]
    fn the_archive_the_store_serves_is_of_the_fourth_version() {
        let version = Version::try_from((Integer::from([0, 1]), Integer::from([0, 4]))).unwrap();

        assert_eq!(version.major(), Integer::from([0, 1]));
        assert_eq!(version.minor(), Integer::from([0, 4]));
    }

    #[test]
    fn a_version_this_repository_has_never_read_is_an_error() {
        let version = Version::try_from((Integer::from([0, 1]), Integer::from([0, 2])));

        let error = Error::ArchiveVersion {
            major: Integer::from([0, 1]),
            minor: Integer::from([0, 2]),
        };
        assert_eq!(version.unwrap_err(), error);
    }

    #[test]
    fn what_this_crate_writes_is_the_version_the_store_serves() {
        let version = Version::default();

        let version_text = version.to_string();

        assert_eq!(version_text, "1.4");
    }

    #[test]
    fn another_major_version_is_an_error_as_well() {
        let version = Version::try_from((Integer::from([0, 2]), Integer::from([0, 3])));

        let error = Error::ArchiveVersion {
            major: Integer::from([0, 2]),
            minor: Integer::from([0, 3]),
        };
        assert_eq!(version.unwrap_err(), error);
    }

    #[test]
    fn a_version_comes_back_from_its_two_integers() {
        let integers: Vec<&str> = vec!["0001", "0004"];
        let deserializer = IntoDeserializer::<value::Error>::into_deserializer(integers);

        let version = Version::deserialize(deserializer).unwrap();

        assert_eq!(version, Version::default());
    }

    #[test]
    fn a_version_this_repository_has_never_read_is_an_error_on_the_way_back_too() {
        let integers: Vec<&str> = vec!["0001", "0002"];
        let deserializer = IntoDeserializer::<value::Error>::into_deserializer(integers);

        let version = Version::deserialize(deserializer);

        assert!(version.is_err());
    }
}
