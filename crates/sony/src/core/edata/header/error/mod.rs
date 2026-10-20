use crate::core::content_identifier;
use bytes::Bytes;
use bytes::Integer;
use bytes::byte::Byte;
use std::error;
use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Error {
    Magic { magic: Bytes<4> },
    Length { bytes: usize },
    Letter { byte: Byte },
    Version { version: Integer<4> },
    License { license: Integer<4> },
    Flags { flags: Integer<4> },
    ContentIdentifier { content: content_identifier::Error },
}

impl fmt::Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Magic { magic } => {
                write!(
                    formatter,
                    "[SONY] a protected file begins with 4E504400 and this one begins with {magic}."
                )
            }
            Self::Length { bytes } => {
                write!(
                    formatter,
                    "[SONY] this file is {bytes} bytes and a protected file is 256 or more."
                )
            }
            Self::Letter { byte } => {
                write!(
                    formatter,
                    "[SONY] the byte {byte} is no letter, a letter is one byte of ASCII."
                )
            }
            Self::Version { version } => {
                let version = u32::from(*version);
                write!(
                    formatter,
                    "[SONY] this repository has read the version 2 of a protected file and this one states {version}."
                )
            }
            Self::License { license } => {
                let license = u32::from(*license);
                write!(
                    formatter,
                    "[SONY] this repository opens a protected file with a license file, which is the kind 2, and this one states {license}."
                )
            }
            Self::Flags { flags } => {
                let flags = u32::from(*flags);
                write!(
                    formatter,
                    "[SONY] this repository has read the flags 12 of a protected file and this one states {flags}."
                )
            }
            Self::ContentIdentifier { content } => fmt::Display::fmt(content, formatter),
        }
    }
}

impl error::Error for Error {}

impl From<content_identifier::Error> for Error {
    fn from(content: content_identifier::Error) -> Self {
        Self::ContentIdentifier { content }
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
    fn a_byte_that_is_no_letter_reaches_the_header_it_was_read_for() {
        let byte = Byte::from(0xFF);
        let blob_error = bytes::blob::Error::Letter { byte };

        let error = Error::from(blob_error);

        assert_eq!(error, Error::Letter { byte });
    }

    #[test]
    fn a_file_that_begins_with_other_bytes_is_shown_as_the_four_it_begins_with() {
        let magic = Bytes::from([0x50, 0x53, 0x41, 0x52]);

        let error = Error::Magic { magic };

        let error_text = error.to_string();
        assert_eq!(
            error_text,
            "[SONY] a protected file begins with 4E504400 and this one begins with 50534152."
        );
    }

    #[test]
    fn a_file_that_ends_before_a_header_is_shown_as_the_bytes_it_has() {
        let bytes: usize = 12;

        let error = Error::Length { bytes };

        let error_text = error.to_string();
        assert_eq!(
            error_text,
            "[SONY] this file is 12 bytes and a protected file is 256 or more."
        );
    }

    #[test]
    fn a_byte_of_a_header_that_is_no_letter_is_shown_as_the_byte_it_is() {
        let byte = Byte::from(0xFF);

        let error = Error::Letter { byte };

        let error_text = error.to_string();
        assert_eq!(
            error_text,
            "[SONY] the byte FF is no letter, a letter is one byte of ASCII."
        );
    }

    #[test]
    fn a_version_this_repository_has_never_read_is_shown_as_the_value_it_states() {
        let version = Integer::from([0x00, 0x00, 0x00, 0x04]);

        let error = Error::Version { version };

        let error_text = error.to_string();
        assert_eq!(
            error_text,
            "[SONY] this repository has read the version 2 of a protected file and this one states 4."
        );
    }

    #[test]
    fn a_license_kind_this_repository_cannot_open_is_shown_as_the_value_it_states() {
        let license = Integer::from([0x00, 0x00, 0x00, 0x03]);

        let error = Error::License { license };

        let error_text = error.to_string();
        assert_eq!(
            error_text,
            "[SONY] this repository opens a protected file with a license file, which is the kind 2, and this one states 3."
        );
    }

    #[test]
    fn flags_this_repository_has_never_read_are_shown_as_the_value_they_state() {
        let flags = Integer::from([0x00, 0x00, 0x00, 0x0D]);

        let error = Error::Flags { flags };

        let error_text = error.to_string();
        assert_eq!(
            error_text,
            "[SONY] this repository has read the flags 12 of a protected file and this one states 13."
        );
    }

    #[test]
    fn a_content_identifier_that_is_no_letters_is_shown_as_the_content_identifier_states_it() {
        let byte = Byte::from(0xFF);
        let content = content_identifier::Error::Letter { byte };

        let error = Error::from(content);

        let error_text = error.to_string();
        assert_eq!(
            error_text,
            "[SONY] a content identifier is letters and zeroes, and the byte FF is neither."
        );
    }

    #[test]
    fn bytes_that_were_not_read_are_a_file_that_ends_too_early() {
        let error = bytes::blob::Error::Length {
            reality: 3,
            expectation: 4,
        };

        let error = Error::from(error);

        let bytes: usize = 3;
        assert_eq!(error, Error::Length { bytes });
    }
}
