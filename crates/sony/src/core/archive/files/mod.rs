mod file;

pub use file::File;

use crate::core::archive::Archive;
use crate::core::archive::Cursor;
use crate::core::archive::FilePath;
use crate::core::archive::cursor;
use crate::core::archive::manifest::Manifest;
use ddd::{DomainLayer, Layered, ValueObject};
use serde::{Deserialize, Serialize};
use std::slice::Iter;

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Files {
    files: Vec<File>,
}

impl Layered for Files {
    type Layer = DomainLayer;
}

impl ValueObject for Files {}

impl Files {
    pub fn count(&self) -> usize {
        self.files.len()
    }

    pub fn file(&self, path: &FilePath) -> Option<&File> {
        self.files.iter().find(|file| file.path() == path)
    }

    pub(super) fn manifest(&self) -> Manifest {
        let paths: Vec<FilePath> = self.files.iter().map(File::path).cloned().collect();
        Manifest::from(paths)
    }
}

impl<'files> IntoIterator for &'files Files {
    type Item = &'files File;
    type IntoIter = Iter<'files, File>;

    fn into_iter(self) -> Self::IntoIter {
        self.files.iter()
    }
}

impl From<Vec<File>> for Files {
    fn from(files: Vec<File>) -> Self {
        Self { files }
    }
}

impl TryFrom<&Archive> for Files {
    type Error = cursor::Error;

    fn try_from(archive: &Archive) -> Result<Self, Self::Error> {
        let cursor = Cursor::try_from(archive)?;
        let count = cursor.file_count();
        let mut files = Vec::with_capacity(count);
        for file in cursor {
            let file = file?;
            files.push(file);
        }
        Ok(Self { files })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::blob::Blob;
    use serde_test::{Token, assert_tokens};

    #[test]
    fn an_archive_without_records_yields_no_files() {
        let path = crate::core::archive::FilePath::try_from("one.pak").unwrap();
        let files = Files::from(vec![File::from((path, Blob::from(vec![1; 100])))]);
        let mut archive = Archive::from(&files);
        archive.records.clear();

        let files = Files::try_from(&archive);

        assert!(files.is_err());
    }

    #[test]
    fn the_files_of_an_archive_are_found_by_their_path() {
        let path = FilePath::try_from("build/main/net.pak").unwrap();
        let file = File::from((path.clone(), Blob::from(vec![1, 2, 3])));
        let files = Files::from(vec![file]);

        let found_file = files.file(&path).unwrap();

        assert_eq!(found_file.bytes(), &Blob::from(vec![1, 2, 3]));
    }

    #[test]
    fn a_path_with_no_file_under_it_is_found_by_nobody() {
        let path = FilePath::try_from("net.pak").unwrap();
        let files = Files::from(vec![File::from((path, Blob::from(vec![1])))]);

        let absent_path = FilePath::try_from("other.pak").unwrap();

        assert_eq!(files.file(&absent_path), None);
    }

    #[test]
    fn the_manifest_of_a_set_of_files_names_every_one_of_them_in_order() {
        let path = FilePath::try_from("one.pak").unwrap();
        let next_path = FilePath::try_from("two.pak").unwrap();
        let files = Files::from(vec![
            File::from((path, Blob::from(vec![1]))),
            File::from((next_path, Blob::from(vec![2]))),
        ]);

        let manifest = files.manifest();

        assert_eq!(manifest.bytes(), Blob::from(b"one.pak\ntwo.pak".as_slice()));
    }

    #[test]
    fn an_archive_of_no_files_at_all_is_a_set_of_no_files() {
        let files = Files::from(Vec::new());

        let count = files.count();

        assert_eq!(count, 0);
    }

    #[test]
    fn a_set_of_files_goes_out_and_comes_back_as_the_files_it_keeps() {
        let path = FilePath::try_from("one.pak").unwrap();
        let files = Files::from(vec![File::from((path, Blob::from(vec![7])))]);

        let tokens = [
            Token::Seq { len: Some(1) },
            Token::Struct {
                name: "File",
                len: 2,
            },
            Token::Str("path"),
            Token::Str("one.pak"),
            Token::Str("bytes"),
            Token::Seq { len: Some(1) },
            Token::U8(7),
            Token::SeqEnd,
            Token::StructEnd,
            Token::SeqEnd,
        ];

        assert_tokens(&files, &tokens);
    }

    #[test]
    fn an_archive_of_no_files_goes_out_and_comes_back_as_no_files() {
        let files = Files::from(Vec::new());

        let tokens = [Token::Seq { len: Some(0) }, Token::SeqEnd];

        assert_tokens(&files, &tokens);
    }
}
