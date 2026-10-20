pub(super) mod error;

use bytes::Integer;
use ddd::{DomainLayer, Layered, ValueObject};
use std::fmt;

pub(super) use error::Error;

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Copy)]
pub(super) struct Release {
    bytes: [u8; Release::LENGTH],
}

impl Layered for Release {
    type Layer = DomainLayer;
}

impl ValueObject for Release {}

impl Release {
    pub(super) const LENGTH: usize = 2;

    const STORE: [u8; Release::LENGTH] = [0x80, 0x00];

    pub(super) fn bytes(&self) -> &[u8; Self::LENGTH] {
        &self.bytes
    }
}

impl TryFrom<[u8; Release::LENGTH]> for Release {
    type Error = Error;

    fn try_from(bytes: [u8; Release::LENGTH]) -> Result<Self, Self::Error> {
        if bytes != Self::STORE {
            let release = Integer::from(bytes);
            return Err(Error::Release { release });
        }
        Ok(Self { bytes })
    }
}

impl Default for Release {
    fn default() -> Self {
        let bytes = Self::STORE;
        Self { bytes }
    }
}

impl fmt::Display for Release {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let release = Integer::from(self.bytes);
        fmt::Display::fmt(&release, formatter)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_release_every_package_of_this_repository_states_is_the_one_the_store_serves() {
        let bytes = [0x80, 0x00];

        let release = Release::try_from(bytes);

        assert_eq!(release.unwrap(), Release::default());
    }

    #[test]
    fn a_package_that_never_left_the_studio_is_an_error() {
        let bytes = [0x00, 0x00];

        let response = Release::try_from(bytes);

        let release = Integer::from([0x00, 0x00]);
        assert_eq!(response.unwrap_err(), Error::Release { release });
    }

    #[test]
    fn a_release_is_shown_as_the_value_its_two_bytes_are() {
        let release = Release::default();

        let letters = release.to_string();

        assert_eq!(letters, "0x8000");
    }

    #[test]
    fn a_release_keeps_the_two_bytes_it_was_read_from() {
        let release = Release::default();

        let bytes = release.bytes();

        assert_eq!(bytes, &[0x80, 0x00]);
    }
}
