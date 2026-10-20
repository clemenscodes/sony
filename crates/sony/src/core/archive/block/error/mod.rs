use std::error;
use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Copy)]
pub enum Error {
    Unpack { bytes: usize },
}

impl fmt::Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unpack { bytes } => {
                write!(formatter, "[SONY] {bytes} packed bytes did not unpack.")
            }
        }
    }
}

impl error::Error for Error {}
