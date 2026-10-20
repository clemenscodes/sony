use crate::core::package::record::kind;
use bytes::byte::Byte;
use std::error;
use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Error {
    Length { bytes: usize },
    Letter { byte: Byte },
    Kind { kind: kind::Error },
}

impl fmt::Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Length { bytes } => {
                write!(
                    formatter,
                    "[SONY] a record of a package is 32 bytes and this one is {bytes}."
                )
            }
            Self::Letter { byte } => {
                write!(
                    formatter,
                    "[SONY] the byte {byte} is no letter, a letter is one byte of ASCII."
                )
            }
            Self::Kind { kind } => fmt::Display::fmt(kind, formatter),
        }
    }
}

impl error::Error for Error {}

impl From<kind::Error> for Error {
    fn from(kind: kind::Error) -> Self {
        Self::Kind { kind }
    }
}

impl From<bytes::blob::Error> for Error {
    fn from(error: bytes::blob::Error) -> Self {
        match error {
            bytes::blob::Error::Length { reality, .. } => Self::Length { bytes: reality },
            bytes::blob::Error::Letter { byte } => Self::Letter { byte },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_byte_that_is_no_letter_reaches_the_record_it_was_read_for() {
        let byte = Byte::from(0xFF);
        let blob_error = bytes::blob::Error::Letter { byte };

        let error = Error::from(blob_error);

        assert_eq!(error, Error::Letter { byte });
    }

    #[test]
    fn a_record_that_ends_too_early_is_shown_as_the_bytes_it_has() {
        let bytes: usize = 12;

        let error = Error::Length { bytes };

        let error_text = error.to_string();
        assert_eq!(
            error_text,
            "[SONY] a record of a package is 32 bytes and this one is 12."
        );
    }

    #[test]
    fn a_byte_of_a_record_that_is_no_letter_is_shown_as_the_byte_it_is() {
        let byte = Byte::from(0xFF);

        let error = Error::Letter { byte };

        let error_text = error.to_string();
        assert_eq!(
            error_text,
            "[SONY] the byte FF is no letter, a letter is one byte of ASCII."
        );
    }

    #[test]
    fn a_kind_this_repository_has_never_read_is_shown_as_the_kind_states_it() {
        let kind = bytes::Integer::from([0x00, 0x00, 0x00, 0x07]);
        let kind = kind::Error::Kind { kind };

        let error = Error::from(kind);

        let error_text = error.to_string();
        assert_eq!(
            error_text,
            "[SONY] this repository knows the kinds 1, 2, 3, 4 and 9 of a record and this one states 7."
        );
    }

    #[test]
    fn bytes_that_were_not_read_are_a_record_that_ends_too_early() {
        let error = bytes::blob::Error::Length {
            reality: 3,
            expectation: 4,
        };

        let error = Error::from(error);

        let bytes: usize = 3;
        assert_eq!(error, Error::Length { bytes });
    }
}
