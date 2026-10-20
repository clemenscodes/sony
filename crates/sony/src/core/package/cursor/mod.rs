use crate::core::package::Package;
use crate::core::package::file::File;
use ddd::{DomainLayer, Layered, ValueObject};

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Cursor<'package> {
    package: &'package Package,
    index: usize,
}

impl Layered for Cursor<'_> {
    type Layer = DomainLayer;
}

impl ValueObject for Cursor<'_> {}

impl<'package> From<&'package Package> for Cursor<'package> {
    fn from(package: &'package Package) -> Self {
        let index = 0;
        Self { package, index }
    }
}

impl Iterator for Cursor<'_> {
    type Item = File;

    fn next(&mut self) -> Option<Self::Item> {
        while self.index < self.package.record_count() {
            let index = self.index;
            self.index = usize::saturating_add(index, 1);
            if let Some(file) = self.package.file(index) {
                return Some(file);
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::package::file_path::FilePath;
    use bytes::blob::Blob;

    #[test]
    fn every_file_of_a_package_comes_out_and_no_directory_does() {
        let text_path = FilePath::try_from("one.txt").unwrap();
        let text_file = File::from((text_path, Blob::from(vec![1, 2, 3])));
        let binary_path = FilePath::try_from("deeper/other.bin").unwrap();
        let binary_file = File::from((binary_path, Blob::from(vec![7; 9])));
        let package = Package::from([text_file.clone(), binary_file.clone()].as_slice());

        let cursor = Cursor::from(&package);

        let files: Vec<File> = cursor.collect();
        assert_eq!(files, vec![text_file, binary_file]);
    }

    #[test]
    fn a_package_of_no_records_at_all_yields_no_file() {
        let files: [File; 0] = [];
        let package = Package::from(files.as_slice());

        let mut cursor = Cursor::from(&package);

        assert_eq!(cursor.next(), None);
    }
}
