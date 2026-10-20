use bytes::Integer;
use std::error;
use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Error {
    Release { release: Integer<2> },
}

impl fmt::Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Release { release } => {
                let release = u16::from(*release);
                write!(
                    formatter,
                    "[SONY] every package this repository reads came from the store and this one states {release}."
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
    fn a_release_the_store_never_served_is_shown_as_the_value_it_states() {
        let release = Integer::from([0x00, 0x00]);

        let error = Error::Release { release };

        let error_text = error.to_string();
        assert_eq!(
            error_text,
            "[SONY] every package this repository reads came from the store and this one states 0."
        );
    }
}
