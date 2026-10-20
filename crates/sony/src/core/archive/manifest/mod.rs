pub(super) mod error;

use crate::core::archive::file_path::FilePath;
use bytes::blob::Blob;
use ddd::{DomainLayer, Layered, ValueObject};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::slice::Iter;

pub(super) use error::Error;

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Default, Serialize, Deserialize)]
#[serde(transparent)]
pub(super) struct Manifest {
    paths: Vec<FilePath>,
}

impl Layered for Manifest {
    type Layer = DomainLayer;
}

impl ValueObject for Manifest {}

impl Manifest {
    pub(super) fn path(&self, index: usize) -> Option<&FilePath> {
        let record = index.checked_sub(1)?;
        self.paths.get(record)
    }

    pub(super) fn count(&self) -> usize {
        self.paths.len()
    }

    pub(super) fn bytes(&self) -> Blob {
        let mut lines = self.to_string();
        lines.pop();
        Blob::from(lines.into_bytes())
    }
}

impl<'manifest> IntoIterator for &'manifest Manifest {
    type Item = &'manifest FilePath;
    type IntoIter = Iter<'manifest, FilePath>;

    fn into_iter(self) -> Self::IntoIter {
        self.paths.iter()
    }
}

impl TryFrom<&Blob> for Manifest {
    type Error = Error;

    fn try_from(blob: &Blob) -> Result<Self, Self::Error> {
        let text = String::from_utf8(Vec::from(blob));
        let Ok(text) = text else {
            let count = blob.length();
            return Err(Error::Manifest { count });
        };
        let mut paths = Vec::new();
        for line in text.lines() {
            if line.is_empty() {
                continue;
            }
            let path = FilePath::try_from(line)?;
            paths.push(path);
        }
        Ok(Self { paths })
    }
}

impl From<Vec<FilePath>> for Manifest {
    fn from(paths: Vec<FilePath>) -> Self {
        Self { paths }
    }
}

impl fmt::Display for Manifest {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut text = String::new();
        for path in &self.paths {
            let line = format!("{path}\n");
            text.push_str(&line);
        }
        fmt::Display::fmt(&text, formatter)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::de::IntoDeserializer;
    use serde::de::value;

    #[test]
    fn a_manifest_hands_out_its_paths_in_the_order_they_are_in_it() {
        let blob = Blob::from(b"one.pak\ntwo.pak".as_slice());
        let manifest = Manifest::try_from(&blob).unwrap();

        let paths: Vec<&FilePath> = (&manifest).into_iter().collect();

        assert_eq!(paths.len(), 2);
        assert_eq!(paths[0].letters(), "one.pak");
        assert_eq!(paths[1].letters(), "two.pak");
    }

    #[test]
    fn a_blank_line_of_a_manifest_names_no_file() {
        let blob = Blob::from(b"one.pak\n\ntwo.pak\n".as_slice());

        let manifest = Manifest::try_from(&blob).unwrap();

        assert_eq!(manifest.count(), 2);
    }

    #[test]
    fn a_line_of_a_manifest_that_is_no_path_is_an_error() {
        let blob = Blob::from(b"/one.pak".as_slice());

        let manifest = Manifest::try_from(&blob);

        let letters = String::from("/one.pak");
        let file_path = crate::core::archive::file_path::Error::FilePath { letters };
        assert_eq!(manifest.unwrap_err(), Error::from(file_path));
    }

    #[test]
    fn the_manifest_of_the_store_names_the_one_file_behind_it() {
        let blob = Blob::from(*b"net-patch9.bin");

        let manifest = Manifest::try_from(&blob).unwrap();

        assert_eq!(manifest.count(), 1);
        assert_eq!(manifest.path(1).unwrap().letters(), "net-patch9.bin");
    }

    #[test]
    fn the_first_line_names_the_record_behind_the_manifest_and_not_the_manifest() {
        let blob = Blob::from(*b"one.pak\ntwo.pak");
        let manifest = Manifest::try_from(&blob).unwrap();

        let first_record = manifest.path(0);

        assert_eq!(first_record, None);
        assert_eq!(manifest.path(1).unwrap().letters(), "one.pak");
        assert_eq!(manifest.path(2).unwrap().letters(), "two.pak");
    }

    #[test]
    fn a_manifest_that_ends_a_line_the_way_a_typewriter_does_is_read_too() {
        let blob = Blob::from(*b"one.pak\r\ntwo.pak");

        let manifest = Manifest::try_from(&blob).unwrap();

        assert_eq!(manifest.count(), 2);
        assert_eq!(manifest.path(2).unwrap().letters(), "two.pak");
    }

    #[test]
    fn a_manifest_ends_without_a_line_of_its_own() {
        let path = FilePath::try_from("one.pak").unwrap();
        let next_path = FilePath::try_from("two.pak").unwrap();
        let manifest = Manifest::from(vec![path, next_path]);

        let bytes = manifest.bytes();

        assert_eq!(bytes, Blob::from(b"one.pak\ntwo.pak".as_slice()));
    }

    #[test]
    fn the_one_the_store_serves_is_fourteen_bytes_for_a_path_of_fourteen_letters() {
        let path = FilePath::try_from("net-patch9.bin").unwrap();
        let manifest = Manifest::from(vec![path]);

        let bytes = manifest.bytes();

        assert_eq!(bytes.length(), 14);
    }

    #[test]
    fn a_manifest_that_is_no_text_is_an_error() {
        let blob = Blob::from([0xFF, 0xFE, 0x00]);

        let manifest = Manifest::try_from(&blob);

        let count: usize = 3;
        assert_eq!(manifest.unwrap_err(), Error::Manifest { count });
    }

    #[test]
    fn an_archive_of_no_files_has_a_manifest_of_no_paths() {
        let blob = Blob::default();

        let manifest = Manifest::try_from(&blob).unwrap();

        assert_eq!(manifest.count(), 0);
    }

    #[test]
    fn a_manifest_comes_back_from_the_paths_it_names() {
        let paths: Vec<&str> = vec!["one.pak", "two.pak"];
        let deserializer = IntoDeserializer::<value::Error>::into_deserializer(paths);

        let manifest = Manifest::deserialize(deserializer).unwrap();

        assert_eq!(manifest.bytes(), Blob::from(b"one.pak\ntwo.pak".as_slice()));
    }

    #[test]
    fn a_manifest_that_names_a_path_of_a_disc_is_an_error_on_the_way_back_too() {
        let paths: Vec<&str> = vec!["one.pak", "/two.pak"];
        let deserializer = IntoDeserializer::<value::Error>::into_deserializer(paths);

        let manifest = Manifest::deserialize(deserializer);

        assert!(manifest.is_err());
    }
}
