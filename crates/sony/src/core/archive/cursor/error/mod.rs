use crate::core::archive;
use crate::core::archive::manifest;
use std::error;
use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Error {
    Digest { letters: String },
    FirstBlock { letters: String },
    Archive { archive: archive::Error },
    Manifest { manifest: manifest::Error },
}

impl fmt::Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Digest { letters } => {
                write!(
                    formatter,
                    "[SONY] the record of {letters} gives the digest of another name."
                )
            }
            Self::FirstBlock { letters } => {
                write!(
                    formatter,
                    "[SONY] the record of {letters} points at an offset where its first block does not begin."
                )
            }
            Self::Archive { archive } => fmt::Display::fmt(archive, formatter),
            Self::Manifest { manifest } => fmt::Display::fmt(manifest, formatter),
        }
    }
}

impl error::Error for Error {}

impl From<archive::Error> for Error {
    fn from(archive: archive::Error) -> Self {
        Self::Archive { archive }
    }
}

impl From<manifest::Error> for Error {
    fn from(manifest: manifest::Error) -> Self {
        Self::Manifest { manifest }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_record_whose_digest_names_another_path_is_shown_with_the_path() {
        let letters = String::from("one.pak");

        let error = Error::Digest { letters };

        let error_text = error.to_string();
        assert_eq!(
            error_text,
            "[SONY] the record of one.pak gives the digest of another name."
        );
    }

    #[test]
    fn a_record_that_points_where_its_first_block_is_not_is_shown_with_the_path() {
        let letters = String::from("one.pak");

        let error = Error::FirstBlock { letters };

        let error_text = error.to_string();
        assert_eq!(
            error_text,
            "[SONY] the record of one.pak points at an offset where its first block does not begin."
        );
    }

    #[test]
    fn an_error_of_an_archive_reaches_the_cursor_that_reads_it() {
        let bytes: usize = 40;
        let archive = archive::Error::Length { bytes };

        let error = Error::from(archive.clone());

        assert_eq!(error, Error::Archive { archive });
    }

    #[test]
    fn an_error_of_a_manifest_reaches_the_cursor_that_reads_it() {
        let count: usize = 3;
        let manifest = manifest::Error::Manifest { count };

        let error = Error::from(manifest.clone());

        assert_eq!(error, Error::Manifest { manifest });
    }

    #[test]
    fn an_archive_a_cursor_cannot_read_is_shown_as_the_archive_states_it() {
        let bytes: usize = 40;
        let archive = archive::Error::Length { bytes };

        let error = Error::Archive { archive };

        let error_text = error.to_string();
        assert_eq!(
            error_text,
            "[SONY] this file is 40 bytes and states it is longer."
        );
    }

    #[test]
    fn a_manifest_a_cursor_cannot_read_is_shown_as_the_manifest_states_it() {
        let count: usize = 3;
        let manifest = manifest::Error::Manifest { count };

        let error = Error::Manifest { manifest };

        let error_text = error.to_string();
        assert_eq!(
            error_text,
            "[SONY] the manifest of this archive names 3 files and the records state otherwise."
        );
    }
}
