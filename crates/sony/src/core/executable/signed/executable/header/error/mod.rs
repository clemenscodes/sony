use std::error;
use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Error {
    SignedExecutable { text: String },
    Length { bytes: usize },
}

impl fmt::Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SignedExecutable { text } => {
                write!(
                    formatter,
                    "[SONY] a signed executable begins with SCE and this one begins with {text}."
                )
            }
            Self::Length { bytes } => {
                write!(
                    formatter,
                    "[SONY] this file is {bytes} bytes and states it is longer."
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
    fn a_file_that_ends_before_the_header_does_is_shown_with_how_many_bytes_it_has() {
        let bytes: usize = 16;

        let error = Error::Length { bytes };

        let error_text = error.to_string();
        assert_eq!(
            error_text,
            "[SONY] this file is 16 bytes and states it is longer."
        );
    }
}
