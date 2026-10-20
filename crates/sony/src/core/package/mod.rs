pub mod error;

pub mod cursor;

mod file;
mod file_path;
mod header;
mod metadata;
mod record;

pub use cursor::Cursor;
pub use error::Error;
pub use file::File;
pub use file_path::FilePath;

use crate::core::content_identifier::ContentIdentifier;
use crate::core::package::header::Header;
use crate::core::package::metadata::Metadata;
use crate::core::package::record::Record;
use aes::Aes128;
use aes::cipher::{KeyIvInit, StreamCipher};
use bytes::blob::Blob;
use ctr::Ctr128BE;
use ddd::{DomainLayer, Layered, ValueObject};

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Package {
    header: Header,
    metadata: Metadata,
    contents: Blob,
    records: Vec<Record>,
}

impl Layered for Package {
    type Layer = DomainLayer;
}

impl ValueObject for Package {}

impl Package {
    const LETTERS: usize = 256;
    const KEY: [u8; 16] = [
        0x2E, 0x7B, 0x71, 0xD7, 0xC9, 0xC9, 0xA1, 0x4E, 0xA3, 0x22, 0x1F, 0x18, 0x88, 0x28, 0xB8,
        0xF8,
    ];

    pub fn content_identifier(&self) -> &ContentIdentifier {
        self.header.content_identifier()
    }

    pub fn record_count(&self) -> usize {
        self.records.len()
    }

    pub fn length(&self) -> usize {
        self.contents.length()
    }

    pub fn entry_count(&self) -> usize {
        self.metadata.count()
    }

    pub fn file(&self, index: usize) -> Option<File> {
        let record = self.records.get(index)?;
        if record.is_directory() {
            return None;
        }
        let contents = self.contents.bytes();
        let name_offset = record.name_offset();
        let letters = record.name_length();
        let name = Blob::from(contents[name_offset..name_offset + letters].to_vec());
        let path = FilePath::try_from(&name)
            .expect("a package names every file it keeps with a file path");
        let offset = record.offset();
        let length = record.length();
        let bytes = Blob::from(contents[offset..offset + length].to_vec());
        let file = File::from((path, bytes));
        Some(file)
    }

    fn try_decrypt(package: &Blob, header: &Header) -> Result<Blob, Error> {
        let offset = header.contents_offset();
        let length = header.contents_length();
        let chunk = package.chunk(offset, length);
        let Some(chunk) = chunk else {
            let bytes = package.length();
            return Err(Error::Length { bytes });
        };
        let mut bytes = Vec::<u8>::from(&chunk);
        let key = Self::KEY;
        let vector = <[u8; Header::VECTOR]>::from(header.vector());
        let mut stream = Ctr128BE::<Aes128>::new((&key).into(), (&vector).into());
        stream.apply_keystream(&mut bytes);
        Ok(Blob::from(bytes))
    }

    fn try_records(header: &Header, contents: &Blob) -> Result<Vec<Record>, Error> {
        let count = header.file_count();
        let table = usize::saturating_mul(count, Record::LENGTH);
        if table > contents.length() {
            let offset = 0;
            let length = table;
            return Err(Error::Contents { offset, length });
        }
        let mut records = Vec::with_capacity(count);
        for index in 0..count {
            let offset = usize::saturating_mul(index, Record::LENGTH);
            let bytes: Vec<u8> = contents.bytes()[offset..offset + Record::LENGTH]
                .iter()
                .copied()
                .map(u8::from)
                .collect();
            let mut record_bytes = [0; Record::LENGTH];
            record_bytes.copy_from_slice(&bytes);
            let record = Record::try_from(record_bytes)?;
            Self::try_fit(&record, contents)?;
            records.push(record);
        }
        Ok(records)
    }

    fn try_fit(record: &Record, contents: &Blob) -> Result<(), Error> {
        let letters = record.name_length();
        if letters > Self::LETTERS {
            return Err(Error::Name { letters });
        }
        let name_offset = record.name_offset();
        let ceiling = usize::saturating_add(name_offset, letters);
        if ceiling > contents.length() {
            let offset = name_offset;
            let length = letters;
            return Err(Error::Contents { offset, length });
        }
        let name = Blob::from(contents.bytes()[name_offset..ceiling].to_vec());
        let length = record.length();
        if length != 0 {
            let offset = record.offset();
            let ceiling = usize::saturating_add(offset, length);
            if ceiling > contents.length() {
                return Err(Error::Contents { offset, length });
            }
            if name_offset == offset {
                return Err(Error::Offset { offset });
            }
        }
        if record.is_directory() {
            return Ok(());
        }
        FilePath::try_from(&name)?;
        Ok(())
    }
}

impl From<&[File]> for Package {
    fn from(files: &[File]) -> Self {
        let count = files.len();
        let records_length = usize::saturating_mul(count, Record::LENGTH);
        let mut names = Vec::<u8>::new();
        for file in files {
            names.extend(file.path().letters().as_bytes().iter().copied());
        }
        let files_offset = usize::saturating_add(records_length, names.len());
        let mut bytes = Vec::<u8>::new();
        let mut records = Vec::with_capacity(count);
        let mut name_offset = records_length;
        for file in files {
            let name_length = file.path().letters().len();
            let offset = usize::saturating_add(files_offset, bytes.len());
            let length = file.length();
            records.push(Record::from((name_offset, name_length, offset, length)));
            name_offset = usize::saturating_add(name_offset, name_length);
            bytes.extend(Vec::<u8>::from(file.bytes()));
        }
        let mut contents = Vec::<u8>::new();
        for record in &records {
            contents.extend(<[u8; Record::LENGTH]>::from(record));
        }
        contents.extend(names);
        contents.extend(bytes);
        let contents = Blob::from(contents);
        let header = Header::from((count, &contents));
        let metadata = Metadata::default();
        Self {
            header,
            metadata,
            contents,
            records,
        }
    }
}

impl From<&Package> for Blob {
    fn from(package: &Package) -> Self {
        let mut contents = Vec::<u8>::from(&package.contents);
        let key = Package::KEY;
        let vector = <[u8; Header::VECTOR]>::from(package.header.vector());
        let mut stream = Ctr128BE::<Aes128>::new((&key).into(), (&vector).into());
        stream.apply_keystream(&mut contents);
        let header = <[u8; Header::LENGTH]>::from(&package.header);
        let length = usize::saturating_add(Header::LENGTH, contents.len());
        let mut bytes = Vec::<u8>::with_capacity(length);
        bytes.extend(header);
        bytes.extend(contents);
        Self::from(bytes)
    }
}

impl TryFrom<&Blob> for Package {
    type Error = Error;

    fn try_from(package: &Blob) -> Result<Self, Self::Error> {
        let header = Header::try_from(package)?;
        let metadata = Metadata::try_from((package, &header))?;
        let contents = Self::try_decrypt(package, &header)?;
        let records = Self::try_records(&header, &contents)?;
        Ok(Self {
            header,
            metadata,
            contents,
            records,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_package_built_from_files_reads_back_as_the_files_it_was_built_from() {
        let text_path = FilePath::try_from("one.txt").unwrap();
        let text_file = File::from((text_path, Blob::from(vec![1, 2, 3])));
        let binary_path = FilePath::try_from("deeper/other.bin").unwrap();
        let binary_file = File::from((binary_path, Blob::from(vec![7; 9])));
        let files = [text_file.clone(), binary_file.clone()];

        let package = Package::from(files.as_slice());

        let bytes = Blob::from(&package);
        let same_package = Package::try_from(&bytes).unwrap();
        let cursor = Cursor::from(&same_package);
        let same_files: Vec<File> = cursor.collect();
        assert_eq!(same_files, vec![text_file, binary_file]);
        assert_eq!(same_package.record_count(), 2);
    }

    #[test]
    fn a_record_whose_name_is_no_file_path_is_an_error_before_any_file_comes_out() {
        let mut contents = vec![0; 0x28];
        contents[0x08..0x10].copy_from_slice(&u64::to_be_bytes(0x24));
        contents[0x10..0x18].copy_from_slice(&u64::to_be_bytes(4));
        contents[0x18..0x1C].copy_from_slice(&u32::to_be_bytes(3));
        contents[0x24..0x28].copy_from_slice(&[1, 2, 3, 4]);
        let length = contents.len();
        let key = Package::KEY;
        let vector = [0x11; Header::VECTOR];
        let mut stream = Ctr128BE::<Aes128>::new((&key).into(), (&vector).into());
        stream.apply_keystream(&mut contents);
        let mut bytes = Blob::zeroes(Header::LENGTH + length);
        bytes.try_write(0x00, [0x7F, 0x50, 0x4B, 0x47]).unwrap();
        bytes.try_write(0x04, [0x80, 0x00]).unwrap();
        bytes.try_write(0x06, [0x00, 0x01]).unwrap();
        bytes.try_write(0x14, u32::to_be_bytes(1)).unwrap();
        bytes.try_write(0x20, u64::to_be_bytes(0x80)).unwrap();
        bytes
            .try_write(0x28, u64::to_be_bytes(u64::try_from(length).unwrap()))
            .unwrap();
        bytes.try_write(0x70, vector).unwrap();
        bytes.try_write(0x80, contents).unwrap();

        let package = Package::try_from(&bytes);

        let letters = String::new();
        let file_path = file_path::Error::FilePath { letters };
        assert_eq!(package.unwrap_err(), Error::FilePath { file_path });
    }

    #[test]
    fn an_index_no_record_of_the_package_has_names_no_file() {
        let text_path = FilePath::try_from("one.txt").unwrap();
        let text_file = File::from((text_path, Blob::from(vec![1, 2, 3])));
        let package = Package::from([text_file].as_slice());

        let file = package.file(9);

        assert_eq!(file, None);
    }

    #[test]
    fn the_files_of_a_package_come_out_and_a_directory_is_none() {
        let mut contents = vec![0; 0x58];
        contents[0x00..0x04].copy_from_slice(&u32::to_be_bytes(0x40));
        contents[0x04..0x08].copy_from_slice(&u32::to_be_bytes(6));
        contents[0x18..0x1C].copy_from_slice(&u32::to_be_bytes(4));
        contents[0x20..0x24].copy_from_slice(&u32::to_be_bytes(0x46));
        contents[0x24..0x28].copy_from_slice(&u32::to_be_bytes(14));
        contents[0x28..0x30].copy_from_slice(&u64::to_be_bytes(0x54));
        contents[0x30..0x38].copy_from_slice(&u64::to_be_bytes(4));
        contents[0x38..0x3C].copy_from_slice(&u32::to_be_bytes(3));
        contents[0x40..0x46].copy_from_slice(b"USRDIR");
        contents[0x46..0x54].copy_from_slice(b"USRDIR/one.bin");
        contents[0x54..0x58].copy_from_slice(&[1, 2, 3, 4]);
        let length = contents.len();
        let key = Package::KEY;
        let vector = [0x11; 16];
        let mut stream = Ctr128BE::<Aes128>::new((&key).into(), (&vector).into());
        stream.apply_keystream(&mut contents);
        let mut bytes = Blob::zeroes(0x80 + length);
        bytes.try_write(0x00, [0x7F, 0x50, 0x4B, 0x47]).unwrap();
        bytes.try_write(0x04, [0x80, 0x00]).unwrap();
        bytes.try_write(0x06, [0x00, 0x01]).unwrap();
        bytes.try_write(0x14, u32::to_be_bytes(2)).unwrap();
        bytes.try_write(0x20, u64::to_be_bytes(0x80)).unwrap();
        bytes
            .try_write(0x28, u64::to_be_bytes(u64::try_from(length).unwrap()))
            .unwrap();
        bytes.try_write(0x70, vector).unwrap();
        bytes.try_write(0x80, contents).unwrap();
        let package = Package::try_from(&bytes).unwrap();

        let cursor = Cursor::from(&package);

        let files: Vec<File> = cursor.collect();
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].path().letters(), "USRDIR/one.bin");
        assert_eq!(files[0].bytes(), &Blob::from(vec![1, 2, 3, 4]));
    }

    #[test]
    fn a_package_of_no_files_at_all_reads_back_as_no_files_at_all() {
        let files: [File; 0] = [];

        let package = Package::from(files.as_slice());

        let bytes = Blob::from(&package);
        let same_package = Package::try_from(&bytes).unwrap();
        assert_eq!(same_package.record_count(), 0);
        assert_eq!(same_package.entry_count(), 0);
    }

    #[test]
    fn a_package_states_how_many_entries_its_metadata_names() {
        let mut contents = vec![0; Record::LENGTH];
        contents[0x18..0x1C].copy_from_slice(&u32::to_be_bytes(4));
        let key = Package::KEY;
        let vector = [0x11; Header::VECTOR];
        let mut stream = Ctr128BE::<Aes128>::new((&key).into(), (&vector).into());
        stream.apply_keystream(&mut contents);
        let mut bytes = Blob::zeroes(Header::LENGTH + Record::LENGTH);
        bytes.try_write(0x00, [0x7F, 0x50, 0x4B, 0x47]).unwrap();
        bytes.try_write(0x04, [0x80, 0x00]).unwrap();
        bytes.try_write(0x06, [0x00, 0x01]).unwrap();
        bytes.try_write(0x14, u32::to_be_bytes(1)).unwrap();
        bytes.try_write(0x20, u64::to_be_bytes(0x80)).unwrap();
        bytes
            .try_write(
                0x28,
                u64::to_be_bytes(u64::try_from(Record::LENGTH).unwrap()),
            )
            .unwrap();
        bytes.try_write(0x70, vector).unwrap();
        bytes.try_write(0x80, contents).unwrap();

        let package = Package::try_from(&bytes).unwrap();

        assert_eq!(package.entry_count(), 0);
    }

    #[test]
    fn a_package_that_ends_before_its_contents_do_is_an_error() {
        let mut bytes = Blob::zeroes(Header::LENGTH);
        bytes.try_write(0x00, [0x7F, 0x50, 0x4B, 0x47]).unwrap();
        bytes.try_write(0x04, [0x80, 0x00]).unwrap();
        bytes.try_write(0x06, [0x00, 0x01]).unwrap();
        bytes.try_write(0x20, u64::to_be_bytes(0x80)).unwrap();
        bytes.try_write(0x28, u64::to_be_bytes(0x1000)).unwrap();

        let package = Package::try_from(&bytes);

        let length: usize = 0x80;
        assert_eq!(package.unwrap_err(), Error::Length { bytes: length });
    }

    #[test]
    fn a_record_of_a_kind_no_package_of_this_disc_knows_is_an_error() {
        let mut contents = vec![0; Record::LENGTH];
        contents[0x18..0x1C].copy_from_slice(&u32::to_be_bytes(99));
        let key = Package::KEY;
        let vector = [0x11; Header::VECTOR];
        let mut stream = Ctr128BE::<Aes128>::new((&key).into(), (&vector).into());
        stream.apply_keystream(&mut contents);
        let mut bytes = Blob::zeroes(Header::LENGTH + Record::LENGTH);
        bytes.try_write(0x00, [0x7F, 0x50, 0x4B, 0x47]).unwrap();
        bytes.try_write(0x04, [0x80, 0x00]).unwrap();
        bytes.try_write(0x06, [0x00, 0x01]).unwrap();
        bytes.try_write(0x14, u32::to_be_bytes(1)).unwrap();
        bytes.try_write(0x20, u64::to_be_bytes(0x80)).unwrap();
        bytes
            .try_write(
                0x28,
                u64::to_be_bytes(u64::try_from(Record::LENGTH).unwrap()),
            )
            .unwrap();
        bytes.try_write(0x70, vector).unwrap();
        bytes.try_write(0x80, contents).unwrap();

        let package = Package::try_from(&bytes);

        assert!(package.is_err());
    }

    #[test]
    fn a_record_whose_name_is_longer_than_any_name_of_a_package_is_an_error() {
        let mut contents = vec![0; Record::LENGTH];
        contents[0x04..0x08].copy_from_slice(&u32::to_be_bytes(300));
        contents[0x18..0x1C].copy_from_slice(&u32::to_be_bytes(3));
        let key = Package::KEY;
        let vector = [0x11; Header::VECTOR];
        let mut stream = Ctr128BE::<Aes128>::new((&key).into(), (&vector).into());
        stream.apply_keystream(&mut contents);
        let mut bytes = Blob::zeroes(Header::LENGTH + Record::LENGTH);
        bytes.try_write(0x00, [0x7F, 0x50, 0x4B, 0x47]).unwrap();
        bytes.try_write(0x04, [0x80, 0x00]).unwrap();
        bytes.try_write(0x06, [0x00, 0x01]).unwrap();
        bytes.try_write(0x14, u32::to_be_bytes(1)).unwrap();
        bytes.try_write(0x20, u64::to_be_bytes(0x80)).unwrap();
        bytes
            .try_write(
                0x28,
                u64::to_be_bytes(u64::try_from(Record::LENGTH).unwrap()),
            )
            .unwrap();
        bytes.try_write(0x70, vector).unwrap();
        bytes.try_write(0x80, contents).unwrap();

        let package = Package::try_from(&bytes);

        let letters: usize = 300;
        assert_eq!(package.unwrap_err(), Error::Name { letters });
    }

    #[test]
    fn a_record_whose_name_reaches_past_the_contents_is_an_error() {
        let mut contents = vec![0; Record::LENGTH];
        contents[0x00..0x04].copy_from_slice(&u32::to_be_bytes(0x100));
        contents[0x04..0x08].copy_from_slice(&u32::to_be_bytes(4));
        contents[0x18..0x1C].copy_from_slice(&u32::to_be_bytes(3));
        let key = Package::KEY;
        let vector = [0x11; Header::VECTOR];
        let mut stream = Ctr128BE::<Aes128>::new((&key).into(), (&vector).into());
        stream.apply_keystream(&mut contents);
        let mut bytes = Blob::zeroes(Header::LENGTH + Record::LENGTH);
        bytes.try_write(0x00, [0x7F, 0x50, 0x4B, 0x47]).unwrap();
        bytes.try_write(0x04, [0x80, 0x00]).unwrap();
        bytes.try_write(0x06, [0x00, 0x01]).unwrap();
        bytes.try_write(0x14, u32::to_be_bytes(1)).unwrap();
        bytes.try_write(0x20, u64::to_be_bytes(0x80)).unwrap();
        bytes
            .try_write(
                0x28,
                u64::to_be_bytes(u64::try_from(Record::LENGTH).unwrap()),
            )
            .unwrap();
        bytes.try_write(0x70, vector).unwrap();
        bytes.try_write(0x80, contents).unwrap();

        let package = Package::try_from(&bytes);

        let offset: usize = 0x100;
        let length: usize = 4;
        assert_eq!(package.unwrap_err(), Error::Contents { offset, length });
    }

    #[test]
    fn a_record_that_names_its_name_and_its_file_at_one_offset_is_an_error() {
        let mut contents = vec![0; Record::LENGTH];
        contents[0x10..0x18].copy_from_slice(&u64::to_be_bytes(8));
        contents[0x18..0x1C].copy_from_slice(&u32::to_be_bytes(3));
        let key = Package::KEY;
        let vector = [0x11; Header::VECTOR];
        let mut stream = Ctr128BE::<Aes128>::new((&key).into(), (&vector).into());
        stream.apply_keystream(&mut contents);
        let mut bytes = Blob::zeroes(Header::LENGTH + Record::LENGTH);
        bytes.try_write(0x00, [0x7F, 0x50, 0x4B, 0x47]).unwrap();
        bytes.try_write(0x04, [0x80, 0x00]).unwrap();
        bytes.try_write(0x06, [0x00, 0x01]).unwrap();
        bytes.try_write(0x14, u32::to_be_bytes(1)).unwrap();
        bytes.try_write(0x20, u64::to_be_bytes(0x80)).unwrap();
        bytes
            .try_write(
                0x28,
                u64::to_be_bytes(u64::try_from(Record::LENGTH).unwrap()),
            )
            .unwrap();
        bytes.try_write(0x70, vector).unwrap();
        bytes.try_write(0x80, contents).unwrap();

        let package = Package::try_from(&bytes);

        let offset: usize = 0;
        assert_eq!(package.unwrap_err(), Error::Offset { offset });
    }

    #[test]
    fn a_package_whose_metadata_reaches_past_its_end_is_an_error() {
        let mut bytes = Blob::zeroes(Header::LENGTH);
        bytes.try_write(0x00, [0x7F, 0x50, 0x4B, 0x47]).unwrap();
        bytes.try_write(0x04, [0x80, 0x00]).unwrap();
        bytes.try_write(0x06, [0x00, 0x01]).unwrap();
        bytes.try_write(0x08, u32::to_be_bytes(0x1000)).unwrap();
        bytes.try_write(0x0C, u32::to_be_bytes(1)).unwrap();
        bytes.try_write(0x10, u32::to_be_bytes(0x10)).unwrap();

        let package = Package::try_from(&bytes);

        assert!(package.is_err());
    }

    #[test]
    fn a_package_whose_last_entry_is_cut_short_is_an_error() {
        let mut bytes = Blob::zeroes(Header::LENGTH);
        bytes.try_write(0x00, [0x7F, 0x50, 0x4B, 0x47]).unwrap();
        bytes.try_write(0x04, [0x80, 0x00]).unwrap();
        bytes.try_write(0x06, [0x00, 0x01]).unwrap();
        bytes.try_write(0x08, u32::to_be_bytes(0x7C)).unwrap();
        bytes.try_write(0x0C, u32::to_be_bytes(1)).unwrap();
        bytes.try_write(0x10, u32::to_be_bytes(0x10)).unwrap();

        let package = Package::try_from(&bytes);

        assert!(package.is_err());
    }

    #[test]
    fn a_package_states_the_content_identifier_its_license_names() {
        let mut contents = vec![0; Record::LENGTH];
        contents[0x18..0x1C].copy_from_slice(&u32::to_be_bytes(4));
        let key = Package::KEY;
        let vector = [0x11; Header::VECTOR];
        let mut stream = Ctr128BE::<Aes128>::new((&key).into(), (&vector).into());
        stream.apply_keystream(&mut contents);
        let mut bytes = Blob::zeroes(Header::LENGTH + Record::LENGTH);
        bytes.try_write(0x00, [0x7F, 0x50, 0x4B, 0x47]).unwrap();
        bytes.try_write(0x04, [0x80, 0x00]).unwrap();
        bytes.try_write(0x06, [0x00, 0x01]).unwrap();
        bytes.try_write(0x14, u32::to_be_bytes(1)).unwrap();
        bytes.try_write(0x20, u64::to_be_bytes(0x80)).unwrap();
        bytes
            .try_write(
                0x28,
                u64::to_be_bytes(u64::try_from(Record::LENGTH).unwrap()),
            )
            .unwrap();
        bytes
            .try_write(0x30, *b"EP0000-XBRK00000_00-XBREAKTESTLABEL0")
            .unwrap();
        bytes.try_write(0x70, vector).unwrap();
        bytes.try_write(0x80, contents).unwrap();

        let package = Package::try_from(&bytes).unwrap();

        let content_identifier = package.content_identifier().to_string();
        assert_eq!(content_identifier, "EP0000-XBRK00000_00-XBREAKTESTLABEL0");
    }

    #[test]
    fn the_records_of_a_package_come_out_from_under_the_key_every_package_shares() {
        let mut contents = vec![0; 0x34];
        contents[0x00..0x04].copy_from_slice(&u32::to_be_bytes(0x20));
        contents[0x04..0x08].copy_from_slice(&u32::to_be_bytes(7));
        contents[0x08..0x10].copy_from_slice(&u64::to_be_bytes(0x30));
        contents[0x10..0x18].copy_from_slice(&u64::to_be_bytes(4));
        contents[0x18..0x1C].copy_from_slice(&u32::to_be_bytes(3));
        contents[0x20..0x27].copy_from_slice(b"one.bin");
        let length = contents.len();
        let key = Package::KEY;
        let vector = [0x11; Header::VECTOR];
        let mut stream = Ctr128BE::<Aes128>::new((&key).into(), (&vector).into());
        stream.apply_keystream(&mut contents);
        let mut bytes = Blob::zeroes(Header::LENGTH + length);
        bytes.try_write(0x00, [0x7F, 0x50, 0x4B, 0x47]).unwrap();
        bytes.try_write(0x04, [0x80, 0x00]).unwrap();
        bytes.try_write(0x06, [0x00, 0x01]).unwrap();
        bytes.try_write(0x14, u32::to_be_bytes(1)).unwrap();
        bytes.try_write(0x20, u64::to_be_bytes(0x80)).unwrap();
        bytes
            .try_write(0x28, u64::to_be_bytes(u64::try_from(length).unwrap()))
            .unwrap();
        bytes.try_write(0x70, vector).unwrap();
        bytes.try_write(0x80, contents).unwrap();

        let package = Package::try_from(&bytes).unwrap();

        assert_eq!(package.record_count(), 1);
        assert_eq!(package.length(), 0x34);
    }

    #[test]
    fn a_record_whose_file_reaches_past_the_contents_is_an_error() {
        let mut contents = vec![0; Record::LENGTH];
        contents[0x00..0x04].copy_from_slice(&u32::to_be_bytes(0x20));
        contents[0x08..0x10].copy_from_slice(&u64::to_be_bytes(0x20));
        contents[0x10..0x18].copy_from_slice(&u64::to_be_bytes(1024));
        contents[0x18..0x1C].copy_from_slice(&u32::to_be_bytes(3));
        let length = contents.len();
        let key = Package::KEY;
        let vector = [0x11; Header::VECTOR];
        let mut stream = Ctr128BE::<Aes128>::new((&key).into(), (&vector).into());
        stream.apply_keystream(&mut contents);
        let mut bytes = Blob::zeroes(Header::LENGTH + length);
        bytes.try_write(0x00, [0x7F, 0x50, 0x4B, 0x47]).unwrap();
        bytes.try_write(0x04, [0x80, 0x00]).unwrap();
        bytes.try_write(0x06, [0x00, 0x01]).unwrap();
        bytes.try_write(0x14, u32::to_be_bytes(1)).unwrap();
        bytes.try_write(0x20, u64::to_be_bytes(0x80)).unwrap();
        bytes
            .try_write(0x28, u64::to_be_bytes(u64::try_from(length).unwrap()))
            .unwrap();
        bytes.try_write(0x70, vector).unwrap();
        bytes.try_write(0x80, contents).unwrap();

        let package = Package::try_from(&bytes);

        let offset: usize = 0x20;
        let length: usize = 1024;
        assert_eq!(package.unwrap_err(), Error::Contents { offset, length });
    }

    #[test]
    fn a_package_whose_table_of_contents_does_not_fit_in_its_contents_is_an_error() {
        let mut bytes = Blob::zeroes(Header::LENGTH + 8);
        bytes.try_write(0x00, [0x7F, 0x50, 0x4B, 0x47]).unwrap();
        bytes.try_write(0x04, [0x80, 0x00]).unwrap();
        bytes.try_write(0x06, [0x00, 0x01]).unwrap();
        bytes.try_write(0x14, u32::to_be_bytes(1)).unwrap();
        bytes.try_write(0x20, u64::to_be_bytes(0x80)).unwrap();
        bytes.try_write(0x28, u64::to_be_bytes(8)).unwrap();

        let package = Package::try_from(&bytes);

        let offset: usize = 0;
        let length: usize = Record::LENGTH;
        assert_eq!(package.unwrap_err(), Error::Contents { offset, length });
    }

    #[test]
    fn a_file_that_is_no_package_is_an_error_by_the_package_and_not_by_its_header_alone() {
        let bytes = Blob::from([0x7F, 0x50, 0x4B, 0x47]);

        let package = Package::try_from(&bytes);

        let header = header::Error::Length { bytes: 4 };
        assert_eq!(package.unwrap_err(), Error::Header { header });
    }
}
