use std::error;
use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Error {
    Identifier { text: String },
}

impl fmt::Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Identifier { text } => {
                write!(formatter, "[SONY] {text} is no identifier.")
            }
        }
    }
}

impl error::Error for Error {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_text_that_yields_no_identifier_is_shown_as_it_is() {
        let text = String::from("32faaf58");

        let error = Error::Identifier { text };

        let error_text = error.to_string();
        assert_eq!(error_text, "[SONY] 32faaf58 is no identifier.");
    }
}
