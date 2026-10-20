pub mod error;

use ddd::{DomainLayer, Layered, ValueObject};
use serde::de;
use serde::{Deserialize, Deserializer, Serialize};
use std::fmt;

pub use error::Error;

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize)]
#[serde(transparent)]
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
        if path.is_empty() || backslash || disc_root {
            return Err(Error::FilePath { letters });
        }
        Ok(Self { letters })
    }
}

impl fmt::Display for FilePath {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.letters, formatter)
    }
}

impl<'letters> Deserialize<'letters> for FilePath {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'letters>,
    {
        let letters = String::deserialize(deserializer)?;
        Self::try_from(letters.as_str()).map_err(de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::de::IntoDeserializer;
    use serde::de::value;

    #[test]
    fn a_file_path_is_the_way_from_the_root_of_the_archive_to_the_file() {
        let letters = "build/main/pak22/net.pak";

        let path = FilePath::try_from(letters).unwrap();

        assert_eq!(path.letters(), letters);
    }

    #[test]
    fn a_file_at_the_root_has_a_path_of_one_step() {
        let letters = "net-patch9.bin";

        let path = FilePath::try_from(letters).unwrap();

        assert_eq!(path.letters(), letters);
    }

    #[test]
    fn a_path_of_no_letters_at_all_is_an_error() {
        let letters = String::new();

        let path = FilePath::try_from(letters.as_str());

        assert_eq!(path, Err(Error::FilePath { letters }));
    }

    #[test]
    fn a_path_separated_the_way_windows_separates_is_an_error() {
        let letters = String::from("build\\main\\net.pak");

        let path = FilePath::try_from(letters.as_str());

        assert_eq!(path, Err(Error::FilePath { letters }));
    }

    #[test]
    fn a_path_that_begins_at_the_root_of_a_disc_and_not_of_the_archive_is_an_error() {
        let letters = String::from("/build/main/net.pak");

        let path = FilePath::try_from(letters.as_str());

        assert_eq!(path, Err(Error::FilePath { letters }));
    }

    #[test]
    fn two_paths_of_the_same_letters_are_the_same_path() {
        let file_path = FilePath::try_from("net.pak").unwrap();

        let same_file_path = FilePath::try_from("net.pak").unwrap();

        assert_eq!(file_path, same_file_path);
    }

    #[test]
    fn a_path_comes_back_from_its_letters() {
        let deserializer =
            IntoDeserializer::<value::Error>::into_deserializer("build/main/pak22/net.pak");

        let path = FilePath::deserialize(deserializer).unwrap();

        assert_eq!(
            path,
            FilePath::try_from("build/main/pak22/net.pak").unwrap()
        );
    }

    #[test]
    fn a_number_is_no_path_on_the_way_back_either() {
        let tokens = [serde_test::Token::U32(7)];

        serde_test::assert_de_tokens_error::<FilePath>(
            &tokens,
            "invalid type: integer `7`, expected a string",
        );
    }

    #[test]
    fn no_letters_at_all_are_no_path_on_the_way_back_either() {
        let deserializer = IntoDeserializer::<value::Error>::into_deserializer("");

        let path = FilePath::deserialize(deserializer);

        assert!(path.is_err());
    }
}
