use std::error;
use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Copy)]
pub enum Error {
    Length { bytes: usize },
    Metadata,
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
            Self::Metadata => {
                write!(
                    formatter,
                    "[SONY] the metadata did not decrypt, so the keyset is the wrong one."
                )
            }
        }
    }
}

impl error::Error for Error {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_file_that_ends_before_the_metadata_does_is_shown_with_how_many_bytes_it_has() {
        let bytes: usize = 64;

        let error = Error::Length { bytes };

        let error_text = error.to_string();
        assert_eq!(
            error_text,
            "[SONY] this file is 64 bytes and states it is longer."
        );
    }
}
