use bytes::byte::Byte;
use std::error;
use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Error {
    Letter { byte: Byte },
}

impl fmt::Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Letter { byte } => {
                write!(
                    formatter,
                    "[SONY] a content identifier is letters and zeroes, and the byte {byte} is neither."
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
    fn a_byte_that_is_no_letter_is_shown_as_the_byte_it_is() {
        let byte = Byte::from(0xFF);

        let error = Error::Letter { byte };

        let error_text = error.to_string();
        assert_eq!(
            error_text,
            "[SONY] a content identifier is letters and zeroes, and the byte FF is neither."
        );
    }
}
