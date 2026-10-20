use crate::core::package::file_path::FilePath;
use bytes::blob::Blob;
use ddd::{DomainLayer, Layered, ValueObject};

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
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

    #[test]
    fn a_file_of_a_package_is_its_path_and_the_bytes_that_belong_to_it() {
        let path = FilePath::try_from("USRDIR/one.bin").unwrap();
        let bytes = Blob::from(vec![1, 2, 3]);

        let file = File::from((path, bytes));

        assert_eq!(file.path().letters(), "USRDIR/one.bin");
        assert_eq!(file.bytes(), &Blob::from(vec![1, 2, 3]));
        assert_eq!(file.length(), 3);
    }

    #[test]
    fn a_file_of_no_bytes_is_a_file_all_the_same() {
        let path = FilePath::try_from("USRDIR/one.bin").unwrap();
        let bytes = Blob::default();

        let file = File::from((path, bytes));

        assert_eq!(file.length(), 0);
    }
}
