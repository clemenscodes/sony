use crate::core::package::metadata::entry;
use std::error;
use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Error {
    Length { bytes: usize },
    Metadata { offset: usize, ceiling: usize },
    Entry { entry: entry::Error },
}

impl fmt::Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Length { bytes } => {
                write!(
                    formatter,
                    "[SONY] this file is {bytes} bytes and its metadata begins behind that."
                )
            }
            Self::Metadata { offset, ceiling } => {
                write!(
                    formatter,
                    "[SONY] the entries of the metadata reach to {offset} and the metadata ends at {ceiling}."
                )
            }
            Self::Entry { entry } => fmt::Display::fmt(entry, formatter),
        }
    }
}

impl error::Error for Error {}

impl From<entry::Error> for Error {
    fn from(entry: entry::Error) -> Self {
        Self::Entry { entry }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_file_that_ends_before_its_metadata_begins_is_shown_as_the_bytes_it_has() {
        let bytes: usize = 12;

        let error = Error::Length { bytes };

        let error_text = error.to_string();
        assert_eq!(
            error_text,
            "[SONY] this file is 12 bytes and its metadata begins behind that."
        );
    }

    #[test]
    fn entries_that_reach_past_the_metadata_are_shown_with_the_offset_and_the_ceiling() {
        let offset: usize = 0x190;
        let ceiling: usize = 0x180;

        let error = Error::Metadata { offset, ceiling };

        let error_text = error.to_string();
        assert_eq!(
            error_text,
            "[SONY] the entries of the metadata reach to 400 and the metadata ends at 384."
        );
    }

    #[test]
    fn an_entry_that_did_not_read_is_shown_as_the_entry_states_it() {
        let bytes: usize = 4;
        let entry = entry::Error::Length { bytes };

        let error = Error::from(entry);

        let error_text = error.to_string();
        assert_eq!(
            error_text,
            "[SONY] an entry of the metadata needs more than the 4 bytes that follow it."
        );
    }
}
