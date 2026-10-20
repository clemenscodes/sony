use crate::core::archive::header::packer;
use crate::core::archive::header::version;
use bytes::byte::Byte;
use std::error;
use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Error {
    Archive { text: String },
    Length { bytes: usize },
    Letter { byte: Byte },
    Packer { packer: packer::Error },
    Version { version: version::Error },
}

impl fmt::Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Archive { text } => {
                write!(
                    formatter,
                    "[SONY] an archive begins with PSAR and this one begins with {text}."
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
            Self::Packer { packer } => fmt::Display::fmt(packer, formatter),
            Self::Version { version } => fmt::Display::fmt(version, formatter),
        }
    }
}

impl error::Error for Error {}

impl From<packer::Error> for Error {
    fn from(packer: packer::Error) -> Self {
        Self::Packer { packer }
    }
}

impl From<version::Error> for Error {
    fn from(version: version::Error) -> Self {
        Self::Version { version }
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
    fn a_packer_this_crate_cannot_unpack_is_shown_as_the_packer_states_it() {
        let text = String::from("lzma");
        let packer = packer::Error::Packer { text };

        let error = Error::Packer { packer };

        let error_text = error.to_string();
        assert_eq!(
            error_text,
            "[SONY] the archives this crate reads are packed with zlib and this one states lzma."
        );
    }

    #[test]
    fn a_version_this_repository_has_never_read_is_shown_as_the_version_states_it() {
        let major = Integer::from([0, 1]);
        let minor = Integer::from([0, 5]);
        let version = version::Error::ArchiveVersion { major, minor };

        let error = Error::Version { version };

        let error_text = error.to_string();
        assert_eq!(
            error_text,
            "[SONY] this repository has read the archive versions 1.3 and 1.4 and this one states 1.5."
        );
    }

    #[test]
    fn a_file_that_ends_before_the_header_does_is_shown_with_how_many_bytes_it_has() {
        let bytes: usize = 12;

        let error = Error::Length { bytes };

        let error_text = error.to_string();
        assert_eq!(
            error_text,
            "[SONY] this file is 12 bytes and states it is longer."
        );
    }

    #[test]
    fn a_byte_that_is_no_letter_reaches_the_header_it_was_read_for() {
        let byte = Byte::from(0xFF);
        let blob_error = bytes::blob::Error::Letter { byte };

        let error = Error::from(blob_error);

        assert_eq!(error, Error::Letter { byte });
    }

    #[test]
    fn an_error_of_a_packer_reaches_the_header_it_was_read_for() {
        let text = String::from("lzma");
        let packer = packer::Error::Packer { text };

        let error = Error::from(packer.clone());

        assert_eq!(error, Error::Packer { packer });
    }

    #[test]
    fn an_error_of_a_version_reaches_the_header_it_was_read_for() {
        let major = Integer::from([0, 1]);
        let minor = Integer::from([0, 5]);
        let version = version::Error::ArchiveVersion { major, minor };

        let error = Error::from(version);

        assert_eq!(error, Error::Version { version });
    }
}
