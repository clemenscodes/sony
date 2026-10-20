use std::error;
use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Error {
    Name { text: String },
}

impl fmt::Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Name { text } => {
                write!(formatter, "[SONY] {text} is no name.")
            }
        }
    }
}

impl error::Error for Error {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_text_that_yields_no_name_is_shown_as_it_is() {
        let text = String::from("cell Http");

        let error = Error::Name { text };

        let error_text = error.to_string();
        assert_eq!(error_text, "[SONY] cell Http is no name.");
    }
}
