use crate::core::package::file_path;
use crate::core::package::header;
use crate::core::package::metadata;
use crate::core::package::record;
use std::error;
use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Error {
    Length { bytes: usize },
    Name { letters: usize },
    Contents { offset: usize, length: usize },
    Offset { offset: usize },
    Header { header: header::Error },
    Metadata { metadata: metadata::Error },
    Record { record: record::Error },
    FilePath { file_path: file_path::Error },
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
            Self::Name { letters } => {
                write!(
                    formatter,
                    "[SONY] a name in a package is 256 letters or fewer and this one is {letters}."
                )
            }
            Self::Contents { offset, length } => {
                write!(
                    formatter,
                    "[SONY] a record names {length} bytes from {offset} and the contents end before that."
                )
            }
            Self::Offset { offset } => {
                write!(
                    formatter,
                    "[SONY] a record names its name and its file from {offset}, and those are never one offset."
                )
            }
            Self::Header { header } => fmt::Display::fmt(header, formatter),
            Self::Metadata { metadata } => fmt::Display::fmt(metadata, formatter),
            Self::Record { record } => fmt::Display::fmt(record, formatter),
            Self::FilePath { file_path } => fmt::Display::fmt(file_path, formatter),
        }
    }
}

impl error::Error for Error {}

impl From<file_path::Error> for Error {
    fn from(file_path: file_path::Error) -> Self {
        Self::FilePath { file_path }
    }
}

impl From<header::Error> for Error {
    fn from(header: header::Error) -> Self {
        Self::Header { header }
    }
}

impl From<metadata::Error> for Error {
    fn from(metadata: metadata::Error) -> Self {
        Self::Metadata { metadata }
    }
}

impl From<record::Error> for Error {
    fn from(record: record::Error) -> Self {
        Self::Record { record }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn entries_that_reach_past_the_metadata_are_shown_as_the_metadata_states_them() {
        let offset: usize = 200;
        let ceiling: usize = 128;
        let metadata = metadata::Error::Metadata { offset, ceiling };

        let error = Error::Metadata { metadata };

        let error_text = error.to_string();
        assert_eq!(
            error_text,
            "[SONY] the entries of the metadata reach to 200 and the metadata ends at 128."
        );
    }

    #[test]
    fn an_error_of_the_metadata_reaches_the_package_it_was_read_for() {
        let offset: usize = 200;
        let ceiling: usize = 128;
        let metadata = metadata::Error::Metadata { offset, ceiling };

        let error = Error::from(metadata.clone());

        assert_eq!(error, Error::Metadata { metadata });
    }

    #[test]
    fn a_file_that_ends_before_its_contents_do_is_shown_as_the_bytes_it_has() {
        let bytes: usize = 4;

        let error = Error::Length { bytes };

        let error_text = error.to_string();
        assert_eq!(
            error_text,
            "[SONY] this file is 4 bytes and states it is longer."
        );
    }

    #[test]
    fn a_name_longer_than_a_package_allows_is_shown_as_the_letters_it_counts() {
        let letters: usize = 300;

        let error = Error::Name { letters };

        let error_text = error.to_string();
        assert_eq!(
            error_text,
            "[SONY] a name in a package is 256 letters or fewer and this one is 300."
        );
    }

    #[test]
    fn a_record_that_reaches_past_the_contents_is_shown_with_both_numbers() {
        let offset: usize = 1000;
        let length: usize = 40;

        let error = Error::Contents { offset, length };

        let error_text = error.to_string();
        assert_eq!(
            error_text,
            "[SONY] a record names 40 bytes from 1000 and the contents end before that."
        );
    }

    #[test]
    fn a_record_whose_name_and_file_begin_alike_is_shown_with_that_one_offset() {
        let offset: usize = 64;

        let error = Error::Offset { offset };

        let error_text = error.to_string();
        assert_eq!(
            error_text,
            "[SONY] a record names its name and its file from 64, and those are never one offset."
        );
    }

    #[test]
    fn a_file_that_ends_before_a_header_is_shown_as_the_header_states_it() {
        let bytes: usize = 4;
        let header = header::Error::Length { bytes };

        let error = Error::from(header);

        let error_text = error.to_string();
        assert_eq!(
            error_text,
            "[SONY] this file is 4 bytes and states it is longer."
        );
    }

    #[test]
    fn bytes_that_are_no_record_are_shown_as_the_record_states_them() {
        let bytes: usize = 12;
        let record = record::Error::Length { bytes };

        let error = Error::from(record);

        let error_text = error.to_string();
        assert_eq!(
            error_text,
            "[SONY] a record of a package is 32 bytes and this one is 12."
        );
    }

    #[test]
    fn a_name_that_is_no_file_path_is_shown_as_the_file_path_states_it() {
        let file_path = file_path::Error::Letters;

        let error = Error::from(file_path);

        let error_text = error.to_string();
        let file_path = file_path::Error::Letters;
        assert_eq!(error_text, file_path.to_string());
    }
}
