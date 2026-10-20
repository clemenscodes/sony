use std::error;
use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Error {
    FilePath { letters: String },
}

impl fmt::Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FilePath { letters } => {
                write!(
                    formatter,
                    "[SONY] {letters} is no path from the root of an archive to a file."
                )
            }
        }
    }
}

impl error::Error for Error {}
