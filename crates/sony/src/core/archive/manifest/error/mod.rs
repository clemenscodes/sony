use crate::core::archive::file_path;
use std::error;
use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Error {
    Manifest { count: usize },
    FilePath { file_path: file_path::Error },
}

impl fmt::Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Manifest { count } => {
                write!(
                    formatter,
                    "[SONY] the manifest of this archive names {count} files and the records state otherwise."
                )
            }
            Self::FilePath { file_path } => fmt::Display::fmt(file_path, formatter),
        }
    }
}

impl error::Error for Error {}

impl From<file_path::Error> for Error {
    fn from(file_path: file_path::Error) -> Self {
        Self::FilePath { file_path }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_line_of_a_manifest_that_is_no_path_is_shown_as_the_path_states_it() {
        let letters = String::from("/net.pak");
        let file_path = file_path::Error::FilePath { letters };

        let error = Error::FilePath { file_path };

        let error_text = error.to_string();
        assert_eq!(
            error_text,
            "[SONY] /net.pak is no path from the root of an archive to a file."
        );
    }

    #[test]
    fn an_error_of_a_path_reaches_the_manifest_it_was_read_for() {
        let letters = String::from("/net.pak");
        let file_path = file_path::Error::FilePath { letters };

        let error = Error::from(file_path.clone());

        assert_eq!(error, Error::FilePath { file_path });
    }
}
