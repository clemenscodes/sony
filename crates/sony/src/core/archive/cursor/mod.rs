pub mod error;

use crate::core::archive::Archive;
use crate::core::archive::files::File;
use crate::core::archive::manifest;
use crate::core::archive::manifest::Manifest;
use crate::core::archive::record::Digest;
use ddd::{DomainLayer, Layered, ValueObject};

pub use error::Error;

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Cursor<'archive> {
    archive: &'archive Archive,
    manifest: Manifest,
    index: usize,
}

impl Layered for Cursor<'_> {
    type Layer = DomainLayer;
}

impl ValueObject for Cursor<'_> {}

impl Cursor<'_> {
    pub fn file_count(&self) -> usize {
        self.manifest.count()
    }

    fn try_file(&self, index: usize) -> Result<File, Error> {
        let path = self.manifest.path(index);
        let Some(path) = path else {
            let count = index;
            let error = manifest::Error::Manifest { count };
            return Err(Error::from(error));
        };
        let record = self.archive.try_record(index)?;
        let digest = Digest::from(path);
        if record.digest() != &digest {
            let letters = path.letters().to_string();
            return Err(Error::Digest { letters });
        }
        let offset = self.archive.offset(record.first_block());
        if offset != Some(record.offset()) {
            let letters = path.letters().to_string();
            return Err(Error::FirstBlock { letters });
        }
        let bytes = self.archive.try_unpack(index)?;
        let path = path.clone();
        let file = File::from((path, bytes));
        Ok(file)
    }
}

impl<'archive> TryFrom<&'archive Archive> for Cursor<'archive> {
    type Error = Error;

    fn try_from(archive: &'archive Archive) -> Result<Self, Self::Error> {
        let index = Archive::MANIFEST;
        let record = archive.try_record(index)?;
        if !record.digest().is_nameless() {
            let letters = String::new();
            return Err(Error::Digest { letters });
        }
        let blob = archive.try_unpack(index)?;
        let manifest = Manifest::try_from(&blob)?;
        let count = archive.count().saturating_sub(1);
        if manifest.count() != count {
            let count = manifest.count();
            let error = manifest::Error::Manifest { count };
            return Err(Error::from(error));
        }
        Ok(Self {
            archive,
            manifest,
            index,
        })
    }
}

impl Iterator for Cursor<'_> {
    type Item = Result<File, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        self.index += 1;
        if self.index >= self.archive.count() {
            return None;
        }
        let file = self.try_file(self.index);
        Some(file)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::archive::file_path::FilePath;
    use crate::core::archive::files::Files;
    use crate::core::archive::record::Record;
    use bytes::blob::Blob;

    #[test]
    fn a_path_of_the_manifest_that_no_record_of_the_archive_names_is_an_error() {
        let path = FilePath::try_from("one.pak").unwrap();
        let next_path = FilePath::try_from("two.pak").unwrap();
        let files = Files::from(vec![
            File::from((path, Blob::from(vec![1; 100]))),
            File::from((next_path, Blob::from(vec![2; 100]))),
        ]);
        let archive = Archive::from(&files);
        let cursor = Cursor::try_from(&archive).unwrap();
        let mut short_archive = Archive::from(&files);
        short_archive.records.truncate(2);
        let short_cursor = Cursor {
            archive: &short_archive,
            manifest: cursor.manifest.clone(),
            index: 0,
        };

        let file = short_cursor.try_file(2);

        let count: usize = 2;
        let manifest_error = manifest::Error::Manifest { count };
        let archive_error = crate::core::archive::Error::from(manifest_error);
        assert_eq!(file.unwrap_err(), Error::from(archive_error));
    }

    #[test]
    fn an_archive_whose_manifest_block_is_gone_has_no_cursor() {
        let path = FilePath::try_from("one.pak").unwrap();
        let files = Files::from(vec![File::from((path, Blob::from(vec![1; 100])))]);
        let mut archive = Archive::from(&files);
        archive.blocks.clear();

        let cursor = Cursor::try_from(&archive);

        let bytes: usize = 0;
        let archive_error = crate::core::archive::Error::Unpack { bytes };
        assert_eq!(cursor.unwrap_err(), Error::from(archive_error));
    }

    #[test]
    fn an_archive_whose_manifest_names_no_path_has_no_cursor() {
        let path = FilePath::try_from("one.pak").unwrap();
        let files = Files::from(vec![File::from((path, Blob::from(vec![1; 100])))]);
        let mut archive = Archive::from(&files);
        let manifest_bytes = Blob::from(b"/onepak".as_slice());
        archive.blocks[0] = crate::core::archive::block::Block::from(manifest_bytes);

        let cursor = Cursor::try_from(&archive);

        let letters = String::from("/onepak");
        let file_path = crate::core::archive::file_path::Error::FilePath { letters };
        let manifest_error = manifest::Error::from(file_path);
        assert_eq!(cursor.unwrap_err(), Error::from(manifest_error));
    }

    #[test]
    fn an_index_the_manifest_does_not_name_is_an_error() {
        let path = FilePath::try_from("one.pak").unwrap();
        let files = Files::from(vec![File::from((path, Blob::from(vec![1; 100])))]);
        let archive = Archive::from(&files);
        let cursor = Cursor::try_from(&archive).unwrap();

        let file = cursor.try_file(9);

        let count: usize = 9;
        let manifest_error = manifest::Error::Manifest { count };
        assert_eq!(file.unwrap_err(), Error::from(manifest_error));
    }

    #[test]
    fn a_record_whose_offset_is_not_where_its_first_block_begins_is_an_error() {
        let path = FilePath::try_from("one.pak").unwrap();
        let files = Files::from(vec![File::from((path, Blob::from(vec![1; 100])))]);
        let mut archive = Archive::from(&files);
        let record = archive.records[1];
        let digest = *record.digest();
        let first_block = record.first_block();
        let file_length = record.file_length();
        archive.records[1] = Record::from((digest, first_block, file_length, 0x7777));
        let cursor = Cursor::try_from(&archive).unwrap();

        let file = cursor.try_file(1);

        let letters = String::from("one.pak");
        assert_eq!(file.unwrap_err(), Error::FirstBlock { letters });
    }

    #[test]
    fn a_file_whose_blocks_the_archive_does_not_have_is_an_error() {
        let path = FilePath::try_from("one.pak").unwrap();
        let files = Files::from(vec![File::from((path, Blob::from(vec![1; 100])))]);
        let archive = Archive::from(&files);
        let cursor = Cursor::try_from(&archive).unwrap();
        let mut archive = Archive::from(&files);
        archive.blocks.truncate(1);
        let short_cursor = Cursor {
            archive: &archive,
            manifest: cursor.manifest.clone(),
            index: 0,
        };

        let file = short_cursor.try_file(1);

        let bytes: usize = 1;
        let archive_error = crate::core::archive::Error::Unpack { bytes };
        assert_eq!(file.unwrap_err(), Error::from(archive_error));
    }

    #[test]
    fn an_archive_without_records_has_no_cursor() {
        let path = FilePath::try_from("one.pak").unwrap();
        let files = Files::from(vec![File::from((path, Blob::from(vec![1; 100])))]);
        let mut archive = Archive::from(&files);
        archive.records.clear();

        let cursor = Cursor::try_from(&archive);

        let count: usize = 0;
        let manifest_error = manifest::Error::Manifest { count };
        let archive_error = crate::core::archive::Error::from(manifest_error);
        assert_eq!(cursor.unwrap_err(), Error::from(archive_error));
    }

    #[test]
    fn an_archive_whose_earliest_record_names_a_file_has_no_cursor() {
        let path = FilePath::try_from("one.pak").unwrap();
        let same_path = path.clone();
        let files = Files::from(vec![File::from((path, Blob::from(vec![1; 100])))]);
        let mut archive = Archive::from(&files);
        let record = archive.records[0];
        let digest = Digest::from(&same_path);
        let first_block = record.first_block();
        let file_length = record.file_length();
        let offset = record.offset();
        archive.records[0] = Record::from((digest, first_block, file_length, offset));

        let cursor = Cursor::try_from(&archive);

        let letters = String::new();
        assert_eq!(cursor.unwrap_err(), Error::Digest { letters });
    }

    #[test]
    fn an_archive_whose_manifest_names_another_count_of_files_has_no_cursor() {
        let path = FilePath::try_from("one.pak").unwrap();
        let next_path = FilePath::try_from("two.pak").unwrap();
        let files = Files::from(vec![
            File::from((path, Blob::from(vec![1; 100]))),
            File::from((next_path, Blob::from(vec![2; 100]))),
        ]);
        let mut archive = Archive::from(&files);
        archive.records.truncate(2);

        let cursor = Cursor::try_from(&archive);

        let count: usize = 2;
        let manifest_error = manifest::Error::Manifest { count };
        assert_eq!(cursor.unwrap_err(), Error::from(manifest_error));
    }

    #[test]
    fn a_cursor_hands_out_one_file_after_the_other_in_the_order_of_the_manifest() {
        let path = FilePath::try_from("one.pak").unwrap();
        let next_path = FilePath::try_from("two.pak").unwrap();
        let files = Files::from(vec![
            File::from((path, Blob::from(vec![1; 100]))),
            File::from((next_path, Blob::from(vec![2; 100]))),
        ]);
        let archive = Archive::from(&files);

        let cursor = Cursor::try_from(&archive).unwrap();

        let files: Vec<File> = cursor.map(Result::unwrap).collect();
        assert_eq!(files.len(), 2);
        assert_eq!(files[0].path().letters(), "one.pak");
        assert_eq!(files[1].path().letters(), "two.pak");
    }

    #[test]
    fn a_cursor_states_how_many_files_there_are_before_one_is_read() {
        let path = FilePath::try_from("one.pak").unwrap();
        let files = Files::from(vec![File::from((path, Blob::from(vec![1])))]);
        let archive = Archive::from(&files);

        let cursor = Cursor::try_from(&archive).unwrap();

        assert_eq!(cursor.file_count(), 1);
    }

    #[test]
    fn the_manifest_is_no_file_of_the_archive_and_is_never_handed_out() {
        let files = Files::from(Vec::new());
        let archive = Archive::from(&files);

        let mut cursor = Cursor::try_from(&archive).unwrap();

        assert_eq!(cursor.next(), None);
    }
}
