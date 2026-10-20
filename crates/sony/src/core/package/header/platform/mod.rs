pub(super) mod error;

use bytes::Integer;
use ddd::{DomainLayer, Layered, ValueObject};
use std::fmt;

pub(super) use error::Error;

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Copy)]
pub(super) struct Platform {
    bytes: [u8; Platform::LENGTH],
}

impl Layered for Platform {
    type Layer = DomainLayer;
}

impl ValueObject for Platform {}

impl Platform {
    pub(super) const LENGTH: usize = 2;

    const CONSOLE: [u8; Platform::LENGTH] = [0x00, 0x01];

    pub(super) fn bytes(&self) -> &[u8; Self::LENGTH] {
        &self.bytes
    }
}

impl TryFrom<[u8; Platform::LENGTH]> for Platform {
    type Error = Error;

    fn try_from(bytes: [u8; Platform::LENGTH]) -> Result<Self, Self::Error> {
        if bytes != Self::CONSOLE {
            let platform = Integer::from(bytes);
            return Err(Error::Platform { platform });
        }
        Ok(Self { bytes })
    }
}

impl Default for Platform {
    fn default() -> Self {
        let bytes = Self::CONSOLE;
        Self { bytes }
    }
}

impl fmt::Display for Platform {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let platform = Integer::from(self.bytes);
        fmt::Display::fmt(&platform, formatter)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_platform_every_package_of_this_repository_states_is_the_console_the_game_runs_on() {
        let bytes = [0x00, 0x01];

        let platform = Platform::try_from(bytes);

        assert_eq!(platform.unwrap(), Platform::default());
    }

    #[test]
    fn a_package_of_a_handheld_is_an_error() {
        let bytes = [0x00, 0x02];

        let response = Platform::try_from(bytes);

        let platform = Integer::from([0x00, 0x02]);
        assert_eq!(response.unwrap_err(), Error::Platform { platform });
    }

    #[test]
    fn a_platform_is_shown_as_the_value_its_two_bytes_are() {
        let platform = Platform::default();

        let letters = platform.to_string();

        assert_eq!(letters, "0x0001");
    }

    #[test]
    fn a_platform_keeps_the_two_bytes_it_was_read_from() {
        let platform = Platform::default();

        let bytes = platform.bytes();

        assert_eq!(bytes, &[0x00, 0x01]);
    }
}
