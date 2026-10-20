use crate::core::archive::file_path::FilePath;
use bytes::blob::Blob;
use ddd::{DomainLayer, Layered, ValueObject};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct File {
    path: FilePath,
    bytes: Blob,
}

impl Layered for File {
    type Layer = DomainLayer;
}

impl ValueObject for File {}

impl File {
    pub fn path(&self) -> &FilePath {
        &self.path
    }

    pub fn bytes(&self) -> &Blob {
        &self.bytes
    }

    pub fn length(&self) -> usize {
        self.bytes.length()
    }
}

impl From<(FilePath, Blob)> for File {
    fn from((path, bytes): (FilePath, Blob)) -> Self {
        Self { path, bytes }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::archive::file_path;
    use serde_test::{Token, assert_de_tokens_error, assert_tokens};

    #[test]
    fn a_file_is_its_path_and_the_bytes_that_belong_to_it() {
        let path = FilePath::try_from("build/main/net.pak").unwrap();
        let bytes = Blob::from(vec![1, 2, 3]);

        let file = File::from((path, bytes));

        assert_eq!(file.path().letters(), "build/main/net.pak");
        assert_eq!(file.bytes(), &Blob::from(vec![1, 2, 3]));
    }

    #[test]
    fn a_file_of_no_bytes_is_a_file_too() {
        let path = FilePath::try_from("empty.bin").unwrap();

        let file = File::from((path, Blob::default()));

        assert_eq!(file.length(), 0);
    }

    #[test]
    fn two_files_of_the_same_path_and_the_same_bytes_are_the_same_file() {
        let path = FilePath::try_from("net.pak").unwrap();
        let file = File::from((path.clone(), Blob::from(vec![7])));

        let same_file = File::from((path, Blob::from(vec![7])));

        assert_eq!(file, same_file);
    }

    #[test]
    fn a_file_goes_out_and_comes_back_as_its_path_and_its_bytes() {
        let path = FilePath::try_from("build/main/net.pak").unwrap();
        let file = File::from((path, Blob::from(vec![1, 2, 3])));

        let tokens = [
            Token::Struct {
                name: "File",
                len: 2,
            },
            Token::Str("path"),
            Token::Str("build/main/net.pak"),
            Token::Str("bytes"),
            Token::Seq { len: Some(3) },
            Token::U8(1),
            Token::U8(2),
            Token::U8(3),
            Token::SeqEnd,
            Token::StructEnd,
        ];

        assert_tokens(&file, &tokens);
    }

    #[test]
    fn a_file_under_a_path_that_is_no_path_of_an_archive_is_an_error_on_the_way_back() {
        let letters = String::from("/net.pak");
        let tokens = [
            Token::Struct {
                name: "File",
                len: 2,
            },
            Token::Str("path"),
            Token::Str("/net.pak"),
        ];

        let error_text = file_path::Error::FilePath { letters }.to_string();

        assert_de_tokens_error::<File>(&tokens, &error_text);
    }
}
