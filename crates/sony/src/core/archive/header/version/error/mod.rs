use crate::core::archive::header::version::Version;
use bytes::Integer;
use std::error;
use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Copy)]
pub enum Error {
    ArchiveVersion {
        major: Integer<{ Version::FIELD_LENGTH }>,
        minor: Integer<{ Version::FIELD_LENGTH }>,
    },
}

impl fmt::Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ArchiveVersion { major, minor } => {
                let major = u16::from(*major);
                let minor = u16::from(*minor);
                write!(
                    formatter,
                    "[SONY] this repository has read the archive versions 1.3 and 1.4 and this one states {major}.{minor}."
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
    fn an_archive_version_is_shown_the_way_this_repository_writes_the_ones_it_reads() {
        let major = Integer::from([0, 1]);
        let minor = Integer::from([0, 5]);

        let error = Error::ArchiveVersion { major, minor };

        let error_text = error.to_string();
        assert_eq!(
            error_text,
            "[SONY] this repository has read the archive versions 1.3 and 1.4 and this one states 1.5."
        );
    }
}
