use crate::core::content_identifier;
use crate::core::package::header::platform;
use crate::core::package::header::release;
use bytes::Bytes;
use bytes::byte::Byte;
use std::error;
use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Error {
    Magic { magic: Bytes<4> },
    Length { bytes: usize },
    Letter { byte: Byte },
    Release { release: release::Error },
    Platform { platform: platform::Error },
    ContentIdentifier { content: content_identifier::Error },
}

impl fmt::Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Magic { magic } => {
                write!(
                    formatter,
                    "[SONY] a package begins with 7F504B47 and this one begins with {magic}."
                )
            }
            Self::Length { bytes } => {
                write!(
                    formatter,
                    "[SONY] this file is {bytes} bytes and states it is longer."
                )
            }
            Self::Letter { byte } => {
                write!(
                    formatter,
                    "[SONY] the byte {byte} is no letter, a letter is one byte of ASCII."
                )
            }
            Self::Release { release } => fmt::Display::fmt(release, formatter),
            Self::Platform { platform } => fmt::Display::fmt(platform, formatter),
            Self::ContentIdentifier { content } => fmt::Display::fmt(content, formatter),
        }
    }
}

impl error::Error for Error {}

impl From<release::Error> for Error {
    fn from(release: release::Error) -> Self {
        Self::Release { release }
    }
}

impl From<platform::Error> for Error {
    fn from(platform: platform::Error) -> Self {
        Self::Platform { platform }
    }
}

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
    use bytes::Integer;

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
            "[SONY] a package begins with 7F504B47 and this one begins with 50534152."
        );
    }

    #[test]
    fn a_file_that_ends_before_the_header_is_shown_as_the_bytes_it_has() {
        let bytes: usize = 12;

        let error = Error::Length { bytes };

        let error_text = error.to_string();
        assert_eq!(
            error_text,
            "[SONY] this file is 12 bytes and states it is longer."
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
    fn bytes_that_were_not_read_are_a_file_that_ends_too_early() {
        let error = bytes::blob::Error::Length {
            reality: 3,
            expectation: 4,
        };

        let error = Error::from(error);

        let bytes: usize = 3;
        assert_eq!(error, Error::Length { bytes });
    }

    #[test]
    fn a_release_the_store_never_served_is_shown_as_the_release_states_it() {
        let release = Integer::from([0x00, 0x00]);
        let release = release::Error::Release { release };

        let error = Error::from(release);

        let error_text = error.to_string();
        assert_eq!(
            error_text,
            "[SONY] every package this repository reads came from the store and this one states 0."
        );
    }

    #[test]
    fn a_platform_this_repository_has_no_package_of_is_shown_as_the_platform_states_it() {
        let platform = Integer::from([0x00, 0x02]);
        let platform = platform::Error::Platform { platform };

        let error = Error::from(platform);

        let error_text = error.to_string();
        assert_eq!(
            error_text,
            "[SONY] this repository reads the packages of one console and this one states 2."
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
}
