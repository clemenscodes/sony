pub mod error;

pub use error::Error;

use bytes::blob::Blob;
use ddd::{DomainLayer, Layered, ValueObject};
use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct FilePath {
    letters: String,
}

impl Layered for FilePath {
    type Layer = DomainLayer;
}

impl ValueObject for FilePath {}

impl FilePath {
    const SEPARATOR: char = '/';
    const BACKSLASH: char = '\\';
    const PARENT: &'static str = "..";

    pub fn letters(&self) -> &str {
        &self.letters
    }
}

impl TryFrom<&str> for FilePath {
    type Error = Error;

    fn try_from(path: &str) -> Result<Self, Self::Error> {
        let letters = path.to_string();
        let backslash = path.chars().any(|letter| letter == Self::BACKSLASH);
        let disc_root = path.starts_with(Self::SEPARATOR);
        let parent = path
            .split(Self::SEPARATOR)
            .any(|letters| letters == Self::PARENT);
        if path.is_empty() || backslash || disc_root || parent {
            return Err(Error::FilePath { letters });
        }
        Ok(Self { letters })
    }
}

impl TryFrom<&Blob> for FilePath {
    type Error = Error;

    fn try_from(name: &Blob) -> Result<Self, Self::Error> {
        let bytes = Vec::<u8>::from(name);
        let letters = String::from_utf8(bytes);
        let Ok(letters) = letters else {
            return Err(Error::Letters);
        };
        Self::try_from(letters.as_str())
    }
}

impl fmt::Display for FilePath {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.letters, formatter)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_file_path_of_a_package_is_the_way_from_its_installation_directory_to_one_file() {
        let path = "USRDIR/one.bin";

        let file_path = FilePath::try_from(path).unwrap();

        assert_eq!(file_path.letters(), "USRDIR/one.bin");
    }

    #[test]
    fn a_file_path_is_read_out_of_the_bytes_a_package_names_it_with() {
        let name = Blob::from(*b"USRDIR/one.bin");

        let file_path = FilePath::try_from(&name).unwrap();

        assert_eq!(file_path.to_string(), "USRDIR/one.bin");
    }

    #[test]
    fn bytes_that_are_no_letters_are_no_file_path() {
        let name = Blob::from([0xFF, 0xFE]);

        let file_path = FilePath::try_from(&name);

        assert_eq!(file_path.unwrap_err(), Error::Letters);
    }

    #[test]
    fn a_path_from_the_root_of_a_disc_is_no_file_path_of_a_package() {
        let path = "/USRDIR/one.bin";

        let file_path = FilePath::try_from(path);

        let letters = String::from("/USRDIR/one.bin");
        assert_eq!(file_path.unwrap_err(), Error::FilePath { letters });
    }

    #[test]
    fn a_path_that_steps_out_of_the_installation_directory_is_an_error() {
        let path = "USRDIR/../../one.bin";

        let file_path = FilePath::try_from(path);

        let letters = String::from("USRDIR/../../one.bin");
        assert_eq!(file_path.unwrap_err(), Error::FilePath { letters });
    }

    #[test]
    fn a_path_of_the_other_platform_is_no_file_path_of_a_package() {
        let path = "USRDIR\\one.bin";

        let file_path = FilePath::try_from(path);

        let letters = String::from("USRDIR\\one.bin");
        assert_eq!(file_path.unwrap_err(), Error::FilePath { letters });
    }

    #[test]
    fn a_path_of_no_letters_is_no_file_path() {
        let path = "";

        let file_path = FilePath::try_from(path);

        let letters = String::new();
        assert_eq!(file_path.unwrap_err(), Error::FilePath { letters });
    }
}
