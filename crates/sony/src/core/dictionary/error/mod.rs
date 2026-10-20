use crate::core::name;
use std::error;
use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Error {
    Shared {
        identifier: String,
        name: String,
        entry_name: String,
    },
    Unknown {
        identifier: String,
    },
    Nameless,
    Name {
        name: name::Error,
    },
}

impl fmt::Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Shared {
                identifier,
                name,
                entry_name,
            } => {
                write!(
                    formatter,
                    "[SONY] {name} and {entry_name} both yield {identifier}, so a identifier no longer names one function."
                )
            }
            Self::Unknown { identifier } => {
                write!(
                    formatter,
                    "[SONY] no name of this dictionary yields {identifier}."
                )
            }
            Self::Nameless => {
                write!(
                    formatter,
                    "[SONY] a dictionary is made of at least one name."
                )
            }
            Self::Name { name } => fmt::Display::fmt(name, formatter),
        }
    }
}

impl error::Error for Error {}

impl From<name::Error> for Error {
    fn from(name: name::Error) -> Self {
        Self::Name { name }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::name;

    #[test]
    fn two_names_that_yield_one_identifier_are_both_named() {
        let identifier = String::from("32FAAF58");
        let name = String::from("cellHttpUtilParseUri");
        let entry_name = String::from("somethingElse");

        let error = Error::Shared {
            identifier,
            name,
            entry_name,
        };

        let error_text = error.to_string();
        assert!(error_text.contains("cellHttpUtilParseUri"));
        assert!(error_text.contains("somethingElse"));
        assert!(error_text.contains("32FAAF58"));
    }

    #[test]
    fn an_identifier_no_name_yields_is_shown() {
        let identifier = String::from("DEADBEEF");

        let error = Error::Unknown { identifier };

        let error_text = error.to_string();
        assert_eq!(
            error_text,
            "[SONY] no name of this dictionary yields DEADBEEF."
        );
    }

    #[test]
    fn a_dictionary_of_no_names_states_what_a_dictionary_needs() {
        let error = Error::Nameless;

        let error_text = error.to_string();

        assert_eq!(
            error_text,
            "[SONY] a dictionary is made of at least one name."
        );
    }

    #[test]
    fn an_error_of_a_name_reaches_the_dictionary_it_was_read_for() {
        let text = String::from("cell Http");
        let name = name::Error::Name { text };

        let error = Error::from(name.clone());

        assert_eq!(error, Error::Name { name });
    }

    #[test]
    fn an_entry_that_is_no_name_is_shown_as_the_name_states_it() {
        let text = String::from("cell Http");
        let name = name::Error::Name { text };

        let error = Error::Name { name };

        let error_text = error.to_string();
        assert_eq!(error_text, "[SONY] cell Http is no name.");
    }
}
