pub mod error;

mod kind;

pub(super) use error::Error;

use crate::core::package::record::kind::Kind;
use bytes::Bytes;
use bytes::Integer;
use bytes::blob::Blob;
use ddd::{DomainLayer, Layered, ValueObject};

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Copy)]
pub(super) struct Record {
    name_offset: usize,
    name_length: usize,
    offset: usize,
    length: usize,
    kind: Kind,
    padding: Bytes<4>,
}

impl Layered for Record {
    type Layer = DomainLayer;
}

impl ValueObject for Record {}

impl Record {
    pub(super) const LENGTH: usize = 0x20;

    pub(super) fn name_offset(&self) -> usize {
        self.name_offset
    }

    pub(super) fn name_length(&self) -> usize {
        self.name_length
    }

    pub(super) fn offset(&self) -> usize {
        self.offset
    }

    pub(super) fn length(&self) -> usize {
        self.length
    }

    pub(super) fn is_directory(&self) -> bool {
        self.kind.is_directory()
    }
}

impl From<(usize, usize, usize, usize)> for Record {
    fn from((name_offset, name_length, offset, length): (usize, usize, usize, usize)) -> Self {
        let kind = Kind::default();
        let padding = Bytes::default();
        Self {
            name_offset,
            name_length,
            offset,
            length,
            kind,
            padding,
        }
    }
}

impl From<&Record> for [u8; Record::LENGTH] {
    fn from(record: &Record) -> Self {
        let mut bytes = [0; Record::LENGTH];
        let name_offset = record.name_offset as u32;
        let name_length = record.name_length as u32;
        bytes[0x00..0x04].copy_from_slice(&name_offset.to_be_bytes());
        bytes[0x04..0x08].copy_from_slice(&name_length.to_be_bytes());
        let offset = record.offset as u64;
        let length = record.length as u64;
        bytes[0x08..0x10].copy_from_slice(&offset.to_be_bytes());
        bytes[0x10..0x18].copy_from_slice(&length.to_be_bytes());
        let kind = <[u8; 4]>::from(record.kind.bytes());
        let padding = <[u8; 4]>::from(record.padding);
        bytes[0x18..0x1C].copy_from_slice(&kind);
        bytes[0x1C..0x20].copy_from_slice(&padding);
        bytes
    }
}

impl TryFrom<&Blob> for Record {
    type Error = Error;

    fn try_from(record: &Blob) -> Result<Self, Self::Error> {
        let record_bytes = record.try_bytes::<{ Record::LENGTH }>(0);
        let Ok(record_bytes) = record_bytes else {
            let bytes = record.length();
            return Err(Error::Length { bytes });
        };
        let record_bytes = <[u8; Record::LENGTH]>::from(record_bytes);
        Self::try_from(record_bytes)
    }
}

impl TryFrom<[u8; Record::LENGTH]> for Record {
    type Error = Error;

    fn try_from(record_bytes: [u8; Record::LENGTH]) -> Result<Self, Self::Error> {
        let mut field_bytes = [0; 4];
        field_bytes.copy_from_slice(&record_bytes[0x00..0x04]);
        let name_offset = Integer::from(field_bytes);
        field_bytes.copy_from_slice(&record_bytes[0x04..0x08]);
        let name_length = Integer::from(field_bytes);
        let name_offset = usize::from(name_offset);
        let name_length = usize::from(name_length);
        let mut word_bytes = [0; 8];
        word_bytes.copy_from_slice(&record_bytes[0x08..0x10]);
        let offset = Integer::from(word_bytes);
        word_bytes.copy_from_slice(&record_bytes[0x10..0x18]);
        let length = Integer::from(word_bytes);
        let offset = usize::from(offset);
        let length = usize::from(length);
        field_bytes.copy_from_slice(&record_bytes[0x18..0x1C]);
        let kind = Kind::try_from(Bytes::from(field_bytes))?;
        field_bytes.copy_from_slice(&record_bytes[0x1C..Self::LENGTH]);
        let padding = Bytes::from(field_bytes);
        Ok(Self {
            name_offset,
            name_length,
            offset,
            length,
            kind,
            padding,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_record_names_where_its_name_is_and_where_its_file_is() {
        let mut record = Blob::zeroes(Record::LENGTH);
        record.try_write(0x00, u32::to_be_bytes(0x40)).unwrap();
        record.try_write(0x04, u32::to_be_bytes(7)).unwrap();
        record.try_write(0x08, u64::to_be_bytes(0x80)).unwrap();
        record.try_write(0x10, u64::to_be_bytes(1024)).unwrap();
        record.try_write(0x18, u32::to_be_bytes(3)).unwrap();

        let record = Record::try_from(&record).unwrap();

        assert_eq!(record.name_offset(), 0x40);
        assert_eq!(record.name_length(), 7);
        assert_eq!(record.offset(), 0x80);
        assert_eq!(record.length(), 1024);
    }

    #[test]
    fn a_record_is_read_back_as_the_bytes_it_came_from() {
        let mut bytes = Blob::zeroes(Record::LENGTH);
        bytes.try_write(0x00, u32::to_be_bytes(0x40)).unwrap();
        bytes.try_write(0x04, u32::to_be_bytes(7)).unwrap();
        bytes.try_write(0x08, u64::to_be_bytes(0x80)).unwrap();
        bytes.try_write(0x10, u64::to_be_bytes(1024)).unwrap();
        bytes.try_write(0x18, u32::to_be_bytes(3)).unwrap();

        let record = Record::try_from(&bytes).unwrap();

        let same_bytes = Blob::from(<[u8; Record::LENGTH]>::from(&record));
        assert_eq!(same_bytes, bytes);
    }

    #[test]
    fn bytes_that_end_before_a_record_does_are_errors() {
        let bytes = Blob::zeroes(12);

        let record = Record::try_from(&bytes);

        let bytes: usize = 12;
        assert_eq!(record.unwrap_err(), Error::Length { bytes });
    }
}
