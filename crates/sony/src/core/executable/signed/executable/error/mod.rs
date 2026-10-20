use crate::core::executable::signed::executable::header;
use crate::core::executable::signed::executable::license;
use crate::core::executable::signed::executable::metadata;
use crate::core::executable::signed::executable::vault;
use std::error;
use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Error {
    Length { bytes: usize },
    Unpack { bytes: usize },
    Header { header: header::Error },
    License { license: license::Error },
    Vault { vault: vault::Error },
    Metadata { metadata: metadata::Error },
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
            Self::Unpack { bytes } => {
                write!(formatter, "[SONY] {bytes} packed bytes did not unpack.")
            }
            Self::Header { header } => fmt::Display::fmt(header, formatter),
            Self::License { license } => fmt::Display::fmt(license, formatter),
            Self::Vault { vault } => fmt::Display::fmt(vault, formatter),
            Self::Metadata { metadata } => fmt::Display::fmt(metadata, formatter),
        }
    }
}

impl error::Error for Error {}

impl From<header::Error> for Error {
    fn from(header: header::Error) -> Self {
        Self::Header { header }
    }
}

impl From<license::Error> for Error {
    fn from(license: license::Error) -> Self {
        Self::License { license }
    }
}

impl From<vault::Error> for Error {
    fn from(vault: vault::Error) -> Self {
        Self::Vault { vault }
    }
}

impl From<metadata::Error> for Error {
    fn from(metadata: metadata::Error) -> Self {
        Self::Metadata { metadata }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::Integer;

    #[test]
    fn a_file_that_is_no_signed_executable_is_shown_as_the_header_states_it() {
        let text = String::from("ELF\0");
        let header = header::Error::SignedExecutable { text };

        let error = Error::Header { header };

        let error_text = error.to_string();
        assert_eq!(
            error_text,
            "[SONY] a signed executable begins with SCE and this one begins with ELF\0."
        );
    }

    #[test]
    fn a_license_this_repository_has_no_key_for_is_shown_as_the_license_states_it() {
        let license = Integer::from([0, 0, 0, 2]);
        let license = license::Error::License { license };

        let error = Error::License { license };

        let error_text = error.to_string();
        assert_eq!(
            error_text,
            "[SONY] a license of 2 needs a key this repository does not have."
        );
    }

    #[test]
    fn a_revision_the_vault_has_no_keyset_for_is_shown_as_the_vault_states_it() {
        let revision = Integer::from([0, 7]);
        let program = Integer::from([0, 0, 0, 8]);
        let vault = vault::Error::Keyset { revision, program };

        let error = Error::Vault { vault };

        let error_text = error.to_string();
        assert_eq!(
            error_text,
            "[SONY] no keyset here fits revision 7 of program type 8."
        );
    }

    #[test]
    fn metadata_that_did_not_decrypt_is_shown_as_the_metadata_states_it() {
        let metadata = metadata::Error::Metadata;

        let error = Error::Metadata { metadata };

        let error_text = error.to_string();
        assert_eq!(
            error_text,
            "[SONY] the metadata did not decrypt, so the keyset is the wrong one."
        );
    }

    #[test]
    fn a_file_that_ends_before_it_states_it_does_is_shown_with_how_many_bytes_it_has() {
        let bytes: usize = 128;

        let error = Error::Length { bytes };

        let error_text = error.to_string();
        assert_eq!(
            error_text,
            "[SONY] this file is 128 bytes and states it is longer."
        );
    }

    #[test]
    fn packed_bytes_that_did_not_unpack_are_shown_with_how_many_there_were() {
        let bytes: usize = 9;

        let error = Error::Unpack { bytes };

        let error_text = error.to_string();
        assert_eq!(error_text, "[SONY] 9 packed bytes did not unpack.");
    }

    #[test]
    fn an_error_of_a_license_reaches_the_executable_it_was_read_for() {
        let license = Integer::from([0, 0, 0, 2]);
        let license = license::Error::License { license };

        let error = Error::from(license);

        assert_eq!(error, Error::License { license });
    }

    #[test]
    fn an_error_of_the_vault_reaches_the_executable_it_was_read_for() {
        let revision = Integer::from([0, 7]);
        let program = Integer::from([0, 0, 0, 8]);
        let vault = vault::Error::Keyset { revision, program };

        let error = Error::from(vault);

        assert_eq!(error, Error::Vault { vault });
    }

    #[test]
    fn an_error_of_the_metadata_reaches_the_executable_it_was_read_for() {
        let metadata = metadata::Error::Metadata;

        let error = Error::from(metadata);

        assert_eq!(error, Error::Metadata { metadata });
    }
}
