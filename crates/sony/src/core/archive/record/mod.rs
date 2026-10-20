mod digest;
pub(super) mod error;

pub(super) use digest::Digest;

use bytes::Integer;
use bytes::blob::Blob;
use ddd::{DomainLayer, Layered, ValueObject};
use serde::{Deserialize, Serialize};

pub(super) use error::Error;

#[derive(
    Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Copy, Default, Serialize, Deserialize,
)]
pub(super) struct Record {
    digest: Digest,
    first_block: usize,
    file_length: usize,
    offset: usize,
}

impl Layered for Record {
    type Layer = DomainLayer;
}

impl ValueObject for Record {}

impl Record {
    pub(super) const LENGTH: usize = 30;

    const FIRST_BLOCK: usize = 0x10;
    const FILE_LENGTH: usize = 0x14;
    const OFFSET: usize = 0x19;
    const FIRST_BLOCK_LENGTH: usize = 4;
    const FIELD_LENGTH: usize = 5;

    pub(super) fn digest(&self) -> &Digest {
        &self.digest
    }

    pub(super) fn first_block(&self) -> usize {
        self.first_block
    }

    pub(super) fn file_length(&self) -> usize {
        self.file_length
    }

    pub(super) fn offset(&self) -> usize {
        self.offset
    }
}

impl From<&Record> for [u8; Record::LENGTH] {
    fn from(record: &Record) -> Self {
        let mut bytes = [0; Record::LENGTH];
        let digest = <[u8; Digest::LENGTH]>::from(record.digest.bytes());
        bytes[..Digest::LENGTH].copy_from_slice(&digest);
        let first_block = (record.first_block as u32).to_be_bytes();
        bytes[Record::FIRST_BLOCK..Record::FIRST_BLOCK + Record::FIRST_BLOCK_LENGTH]
            .copy_from_slice(&first_block);
        let file_length = (record.file_length as u64).to_be_bytes();
        bytes[Record::FILE_LENGTH..Record::OFFSET]
            .copy_from_slice(&file_length[file_length.len() - Record::FIELD_LENGTH..]);
        let offset = (record.offset as u64).to_be_bytes();
        bytes[Record::OFFSET..Record::LENGTH]
            .copy_from_slice(&offset[offset.len() - Record::FIELD_LENGTH..]);
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
        Ok(Self::from(record_bytes))
    }
}

impl From<[u8; Record::LENGTH]> for Record {
    fn from(record_bytes: [u8; Record::LENGTH]) -> Self {
        let mut digest_bytes = [0; Digest::LENGTH];
        digest_bytes.copy_from_slice(&record_bytes[..Digest::LENGTH]);
        let digest = Digest::from(digest_bytes);
        let mut first_block = [0; Record::FIRST_BLOCK_LENGTH];
        let last_block = Self::FIRST_BLOCK + Record::FIRST_BLOCK_LENGTH;
        first_block.copy_from_slice(&record_bytes[Self::FIRST_BLOCK..last_block]);
        let mut file_length = [0; Record::FIELD_LENGTH];
        file_length.copy_from_slice(&record_bytes[Self::FILE_LENGTH..Self::OFFSET]);
        let mut offset = [0; Record::FIELD_LENGTH];
        offset.copy_from_slice(&record_bytes[Self::OFFSET..Self::LENGTH]);
        let first_block = Integer::from(first_block);
        let file_length = Integer::from(file_length);
        let offset = Integer::from(offset);
        let first_block = usize::from(first_block);
        let file_length = usize::from(file_length);
        let offset = usize::from(offset);
        Self {
            digest,
            first_block,
            file_length,
            offset,
        }
    }
}

impl From<(Digest, usize, usize, usize)> for Record {
    fn from((digest, first_block, file_length, offset): (Digest, usize, usize, usize)) -> Self {
        Self {
            digest,
            first_block,
            file_length,
            offset,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::archive::file_path::FilePath;
    use serde_test::{Token, assert_tokens};

    #[test]
    fn the_record_the_store_wrote_for_its_one_file_is_read_field_for_field() {
        let mut record_bytes = Blob::zeroes(Record::LENGTH);
        let path = FilePath::try_from("net-patch9.bin").unwrap();
        let digest = Digest::from(&path);
        record_bytes
            .try_write(0, <[u8; Digest::LENGTH]>::from(digest.bytes()))
            .unwrap();
        record_bytes.try_write(0x10, u32::to_be_bytes(1)).unwrap();
        record_bytes
            .try_write(0x14, [0x00, 0x00, 0x02, 0xAF, 0xF8])
            .unwrap();
        record_bytes
            .try_write(0x19, [0x00, 0x00, 0x00, 0x00, 0x72])
            .unwrap();

        let record = Record::try_from(&record_bytes).unwrap();

        assert_eq!(record.digest(), &digest);
        assert_eq!(record.first_block(), 1);
        assert_eq!(record.file_length(), 176120);
        assert_eq!(record.offset(), 114);
    }

    #[test]
    fn a_length_beyond_the_ceiling_of_four_bytes_is_read_out_of_five() {
        let mut record_bytes = Blob::zeroes(Record::LENGTH);
        record_bytes
            .try_write(0x14, [0x01, 0x00, 0x00, 0x00, 0x00])
            .unwrap();

        let record = Record::try_from(&record_bytes).unwrap();

        assert_eq!(record.file_length(), 4294967296);
    }

    #[test]
    fn an_offset_beyond_four_bytes_is_read_as_well_because_one_archive_is_that_long() {
        let mut record_bytes = Blob::zeroes(Record::LENGTH);
        record_bytes
            .try_write(0x19, [0x01, 0x4D, 0x92, 0xF7, 0x71])
            .unwrap();

        let record = Record::try_from(&record_bytes).unwrap();

        assert_eq!(record.offset(), 5596444529);
    }

    #[test]
    fn a_record_goes_out_to_bytes_and_comes_back_the_same() {
        let path = FilePath::try_from("build/main/net.pak").unwrap();
        let digest = Digest::from(&path);
        let record = Record::from((digest, 7, 176120, 5596444529));

        let bytes = Blob::from(<[u8; Record::LENGTH]>::from(&record));

        let same_record = Record::try_from(&bytes).unwrap();
        assert_eq!(same_record, record);
    }

    #[test]
    fn fewer_bytes_than_a_record_is_no_record() {
        let record_bytes = Blob::zeroes(29);

        let record = Record::try_from(&record_bytes);

        let bytes: usize = 29;
        assert_eq!(record.unwrap_err(), Error::Length { bytes });
    }

    #[test]
    fn the_record_of_the_manifest_names_nothing() {
        let record_bytes = Blob::zeroes(Record::LENGTH);

        let record = Record::try_from(&record_bytes).unwrap();

        assert!(record.digest().is_nameless());
    }

    #[test]
    fn a_record_goes_out_and_comes_back_field_for_field() {
        let path = FilePath::try_from("net-patch9.bin").unwrap();
        let digest = Digest::from(&path);
        let first_block: usize = 1;
        let file_length: usize = 176120;
        let record = Record::from((digest, first_block, file_length, 114));

        let tokens = [
            Token::Struct {
                name: "Record",
                len: 4,
            },
            Token::Str("digest"),
            Token::Str("1AA7B23FF54BF508C1D86EAA35340A14"),
            Token::Str("first_block"),
            Token::U64(1),
            Token::Str("file_length"),
            Token::U64(176120),
            Token::Str("offset"),
            Token::U64(114),
            Token::StructEnd,
        ];

        assert_tokens(&record, &tokens);
    }
}
