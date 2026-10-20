use sony::dictionary;
use sony::identifier;
use std::error;
use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub(in crate::application) enum Error {
    Identifier { identifier: identifier::Error },
    Dictionary { dictionary: dictionary::Error },
}

impl fmt::Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Identifier { identifier } => fmt::Display::fmt(identifier, formatter),
            Self::Dictionary { dictionary } => fmt::Display::fmt(dictionary, formatter),
        }
    }
}

impl error::Error for Error {}

impl From<identifier::Error> for Error {
    fn from(identifier: identifier::Error) -> Self {
        Self::Identifier { identifier }
    }
}

impl From<dictionary::Error> for Error {
    fn from(dictionary: dictionary::Error) -> Self {
        Self::Dictionary { dictionary }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_writing_that_is_no_identifier_is_shown_as_the_identifier_states_it() {
        let text = String::from("32faaf58");
        let identifier = identifier::Error::Identifier { text };

        let error = Error::Identifier { identifier };

        let error_text = error.to_string();
        assert_eq!(error_text, "[SONY] 32faaf58 is no identifier.");
    }

    #[test]
    fn an_identifier_no_name_yields_is_shown_as_the_dictionary_states_it() {
        let identifier = String::from("DEADBEEF");
        let dictionary = dictionary::Error::Unknown { identifier };

        let error = Error::Dictionary { dictionary };

        let error_text = error.to_string();
        assert_eq!(
            error_text,
            "[SONY] no name of this dictionary yields DEADBEEF."
        );
    }
}
