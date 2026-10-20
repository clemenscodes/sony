use crate::core::archive::block;
use crate::core::archive::header;
use crate::core::archive::manifest;
use crate::core::archive::record;
use std::error;
use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Error {
    Length { bytes: usize },
    Unpack { bytes: usize },
    Alignment { offset: usize, alignment: usize },
    Header { header: header::Error },
    Record { record: record::Error },
    Block { block: block::Error },
    Manifest { manifest: manifest::Error },
}

impl fmt::Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Length { bytes } => {
                write!(
                    formatter,
                    "[SONY] this file is {bytes} bytes and states it is longer."
                )
            }
            Self::Unpack { bytes } => {
                write!(formatter, "[SONY] {bytes} packed bytes did not unpack.")
            }
            Self::Alignment { offset, alignment } => {
                write!(
                    formatter,
                    "[SONY] byte {offset} is no multiple of {alignment}."
                )
            }
            Self::Header { header } => fmt::Display::fmt(header, formatter),
            Self::Record { record } => fmt::Display::fmt(record, formatter),
            Self::Block { block } => fmt::Display::fmt(block, formatter),
            Self::Manifest { manifest } => fmt::Display::fmt(manifest, formatter),
        }
    }
}

impl error::Error for Error {}

impl From<header::Error> for Error {
    fn from(header: header::Error) -> Self {
        Self::Header { header }
    }
}

impl From<record::Error> for Error {
    fn from(record: record::Error) -> Self {
        Self::Record { record }
    }
}

impl From<block::Error> for Error {
    fn from(block: block::Error) -> Self {
        Self::Block { block }
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
    fn an_offset_that_is_no_multiple_of_the_alignment_is_shown_with_both() {
        let offset: usize = 30;
        let alignment: usize = 16;

        let error = Error::Alignment { offset, alignment };

        let error_text = error.to_string();
        assert_eq!(error_text, "[SONY] byte 30 is no multiple of 16.");
    }

    #[test]
    fn a_file_that_is_no_archive_is_shown_as_the_header_states_it() {
        let text = String::from("PSAF");
        let header = header::Error::Archive { text };

        let error = Error::Header { header };

        let error_text = error.to_string();
        assert_eq!(
            error_text,
            "[SONY] an archive begins with PSAR and this one begins with PSAF."
        );
    }

    #[test]
    fn a_record_that_ends_too_early_is_shown_as_the_record_states_it() {
        let bytes: usize = 29;
        let record = record::Error::Length { bytes };

        let error = Error::Record { record };

        let error_text = error.to_string();
        assert_eq!(
            error_text,
            "[SONY] this file is 29 bytes and states it is longer."
        );
    }

    #[test]
    fn a_block_that_did_not_unpack_is_shown_as_the_block_states_it() {
        let bytes: usize = 5;
        let block = block::Error::Unpack { bytes };

        let error = Error::Block { block };

        let error_text = error.to_string();
        assert_eq!(error_text, "[SONY] 5 packed bytes did not unpack.");
    }

    #[test]
    fn a_manifest_that_disagrees_with_the_records_is_shown_as_the_manifest_states_it() {
        let count: usize = 3;
        let manifest = manifest::Error::Manifest { count };

        let error = Error::Manifest { manifest };

        let error_text = error.to_string();
        assert_eq!(
            error_text,
            "[SONY] the manifest of this archive names 3 files and the records state otherwise."
        );
    }

    #[test]
    fn packed_bytes_that_did_not_unpack_are_shown_with_how_many_there_were() {
        let bytes: usize = 5;

        let error = Error::Unpack { bytes };

        let error_text = error.to_string();
        assert_eq!(error_text, "[SONY] 5 packed bytes did not unpack.");
    }

    #[test]
    fn an_error_of_a_record_reaches_the_archive_it_was_read_for() {
        let bytes: usize = 29;
        let record = record::Error::Length { bytes };

        let error = Error::from(record);

        assert_eq!(error, Error::Record { record });
    }

    #[test]
    fn an_error_of_a_block_reaches_the_archive_it_was_read_for() {
        let bytes: usize = 5;
        let block = block::Error::Unpack { bytes };

        let error = Error::from(block);

        assert_eq!(error, Error::Block { block });
    }

    #[test]
    fn an_error_of_a_manifest_reaches_the_archive_it_was_read_for() {
        let count: usize = 3;
        let manifest = manifest::Error::Manifest { count };

        let error = Error::from(manifest.clone());

        assert_eq!(error, Error::Manifest { manifest });
    }
}
