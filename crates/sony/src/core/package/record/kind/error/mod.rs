use bytes::Integer;
use std::error;
use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Error {
    Kind { kind: Integer<4> },
}

impl fmt::Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Kind { kind } => {
                let kind = u32::from(*kind);
                write!(
                    formatter,
                    "[SONY] this repository knows the kinds 1, 2, 3, 4 and 9 of a record and this one states {kind}."
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
    fn a_kind_this_repository_has_never_read_is_shown_as_the_value_it_states() {
        let kind = Integer::from([0x00, 0x00, 0x00, 0x07]);

        let error = Error::Kind { kind };

        let error_text = error.to_string();
        assert_eq!(
            error_text,
            "[SONY] this repository knows the kinds 1, 2, 3, 4 and 9 of a record and this one states 7."
        );
    }
}
