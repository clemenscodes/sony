use std::error;
use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Error {
    Packer { text: String },
}

impl fmt::Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Packer { text } => {
                write!(
                    formatter,
                    "[SONY] the archives this crate reads are packed with zlib and this one states {text}."
                )
            }
        }
    }
}

impl error::Error for Error {}
