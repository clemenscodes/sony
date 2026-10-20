use crate::core::edata::header;
use std::error;
use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Error {
    Length { bytes: usize },
    Klicensee,
    Header { header: header::Error },
}

impl fmt::Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Length { bytes } => {
                write!(
                    formatter,
                    "[SONY] this file is {bytes} bytes and its hash begins behind that."
                )
            }
            Self::Klicensee => {
                write!(
                    formatter,
                    "[SONY] the hash of this protected file is not the one this klicensee yields, so this klicensee does not open it."
                )
            }
            Self::Header { header } => fmt::Display::fmt(header, formatter),
        }
    }
}

impl error::Error for Error {}

impl From<header::Error> for Error {
    fn from(header: header::Error) -> Self {
        Self::Header { header }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_file_that_ends_before_its_hash_is_shown_as_the_bytes_it_has() {
        let bytes: usize = 100;

        let error = Error::Length { bytes };

        let error_text = error.to_string();
        assert_eq!(
            error_text,
            "[SONY] this file is 100 bytes and its hash begins behind that."
        );
    }

    #[test]
    fn a_klicensee_that_does_not_open_a_protected_file_is_shown_as_that() {
        let error = Error::Klicensee;

        let error_text = error.to_string();

        assert_eq!(
            error_text,
            "[SONY] the hash of this protected file is not the one this klicensee yields, so this klicensee does not open it."
        );
    }

    #[test]
    fn a_file_that_is_no_protected_file_is_shown_as_the_header_states_it() {
        let bytes: usize = 4;
        let header = header::Error::Length { bytes };

        let error = Error::from(header);

        let error_text = error.to_string();
        assert_eq!(
            error_text,
            "[SONY] this file is 4 bytes and a protected file is 256 or more."
        );
    }
}
