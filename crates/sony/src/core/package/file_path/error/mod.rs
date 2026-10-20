use std::error;
use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Error {
    FilePath { letters: String },
    Letters,
}

impl fmt::Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FilePath { letters } => {
                write!(
                    formatter,
                    "[SONY] a file path of a package is a way under its installation directory and this one is {letters}."
                )
            }
            Self::Letters => {
                write!(
                    formatter,
                    "[SONY] a name in a package is letters and these bytes are none."
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
    fn a_path_that_is_no_way_under_the_installation_directory_is_shown_as_it_reads() {
        let letters = String::from("/USRDIR/one.bin");

        let error = Error::FilePath { letters };

        let error_text = error.to_string();
        assert_eq!(
            error_text,
            "[SONY] a file path of a package is a way under its installation directory and this one is /USRDIR/one.bin."
        );
    }

    #[test]
    fn bytes_that_are_no_letters_are_shown_as_no_name() {
        let error = Error::Letters;

        let error_text = error.to_string();

        assert_eq!(
            error_text,
            "[SONY] a name in a package is letters and these bytes are none."
        );
    }
}
