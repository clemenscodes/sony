use bytes::Integer;
use std::error;
use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Copy)]
pub enum Error {
    Length { bytes: usize },
    License { license: Integer<4> },
}

impl fmt::Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Length { bytes } => {
                write!(
                    formatter,
                    "[SONY] this file is {bytes} bytes and states it is longer."
                )
            }
            Self::License { license } => {
                let license = u32::from(*license);
                write!(
                    formatter,
                    "[SONY] a license of {license} needs a key this repository does not have."
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
    fn a_license_is_shown_as_the_value_it_states() {
        let license = Integer::from([0, 0, 0, 2]);

        let error = Error::License { license };

        let error_text = error.to_string();
        assert_eq!(
            error_text,
            "[SONY] a license of 2 needs a key this repository does not have."
        );
    }

    #[test]
    fn a_file_that_ends_before_the_control_information_does_is_shown_with_its_bytes() {
        let bytes: usize = 48;

        let error = Error::Length { bytes };

        let error_text = error.to_string();
        assert_eq!(
            error_text,
            "[SONY] this file is 48 bytes and states it is longer."
        );
    }
}
