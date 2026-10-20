use crate::core::executable::signed::executable::vault::Keyset;
use bytes::Integer;
use std::error;
use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Copy)]
pub enum Error {
    Keyset {
        revision: Integer<{ Keyset::REVISION_LENGTH }>,
        program: Integer<4>,
    },
}

impl fmt::Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Keyset { revision, program } => {
                let revision = u16::from(*revision);
                let program = u32::from(*program);
                write!(
                    formatter,
                    "[SONY] no keyset here fits revision {revision} of program type {program}."
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
    fn a_revision_and_a_program_type_are_shown_as_the_values_a_reader_compares() {
        let revision = Integer::from([0, 1]);
        let program = Integer::from([0, 0, 0, 8]);

        let error = Error::Keyset { revision, program };

        let error_text = error.to_string();
        assert_eq!(
            error_text,
            "[SONY] no keyset here fits revision 1 of program type 8."
        );
    }
}
