use bytes::Integer;
use std::error;
use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Error {
    Platform { platform: Integer<2> },
}

impl fmt::Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Platform { platform } => {
                let platform = u16::from(*platform);
                write!(
                    formatter,
                    "[SONY] this repository reads the packages of one console and this one states {platform}."
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
    fn a_platform_this_repository_has_no_package_of_is_shown_as_the_value_it_states() {
        let platform = Integer::from([0x00, 0x02]);

        let error = Error::Platform { platform };

        let error_text = error.to_string();
        assert_eq!(
            error_text,
            "[SONY] this repository reads the packages of one console and this one states 2."
        );
    }
}
