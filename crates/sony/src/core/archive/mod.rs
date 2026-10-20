pub mod error;
pub mod files;

pub mod cursor;
mod file_path;

mod block;
mod header;
mod manifest;
mod record;

pub use cursor::Cursor;
pub use error::Error;
pub use file_path::FilePath;

use crate::core::archive::block::Block;
use crate::core::archive::files::Files;
use crate::core::archive::header::Header;
use crate::core::archive::header::Version;
use crate::core::archive::record::Digest;
use crate::core::archive::record::Record;
use bytes::Integer;
use bytes::blob::Blob;
use bytes::byte::Byte;
use ddd::{DomainLayer, Layered, ValueObject};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(into = "Blob", try_from = "Blob")]
pub struct Archive {
    header: Header,
    records: Vec<Record>,
    blocks: Vec<Block>,
    remnant: Blob,
}

impl Layered for Archive {
    type Layer = DomainLayer;
}

impl ValueObject for Archive {}

impl Archive {
    const MANIFEST: usize = 0;
    const FIELD_LENGTH: usize = size_of::<usize>();

    fn count(&self) -> usize {
        self.records.len()
    }

    fn try_record(&self, index: usize) -> Result<&Record, Error> {
        let record = self.records.get(index);
        let Some(record) = record else {
            let count = index;
            let error = manifest::Error::Manifest { count };
            return Err(Error::from(error));
        };
        Ok(record)
    }

    fn offset(&self, first_block: usize) -> Option<usize> {
        if first_block > self.blocks.len() {
            return None;
        }
        let blocks = &self.blocks[..first_block];
        let length: usize = blocks.iter().map(Block::length).sum();
        let contents_offset = self.header.contents_offset();
        Some(contents_offset + length)
    }

    fn try_unpack(&self, index: usize) -> Result<Blob, Error> {
        let record = self.try_record(index)?;
        let length = record.file_length();
        let block_length = self.header.block_length();
        let mut blob = Blob::with_capacity(length);
        let mut block_index = record.first_block();
        while blob.length() < length {
            let block = self.blocks.get(block_index);
            let Some(block) = block else {
                let bytes = block_index;
                return Err(Error::Unpack { bytes });
            };
            let chunk = block.try_unpack(block_length)?;
            blob.extend(chunk);
            block_index += 1;
        }
        blob.truncate(length);
        Ok(blob)
    }

    fn field(length_bytes: &[Byte], index: usize, field_length: usize) -> usize {
        let offset = index * field_length;
        let padding = Self::FIELD_LENGTH - field_length;
        let bytes: Vec<u8> = length_bytes[offset..offset + field_length]
            .iter()
            .copied()
            .map(u8::from)
            .collect();
        let mut field = [0; Archive::FIELD_LENGTH];
        field[padding..].copy_from_slice(&bytes);
        let field = Integer::from(field);
        usize::from(field)
    }

    fn chunks(bytes: &Blob, block_length: usize) -> Vec<Blob> {
        let mut chunks = Vec::new();
        for chunk in bytes.bytes().chunks(block_length) {
            let chunk = Blob::from(chunk.to_vec());
            chunks.push(chunk);
        }
        chunks
    }
}

impl TryFrom<&Blob> for Archive {
    type Error = Error;

    fn try_from(archive: &Blob) -> Result<Self, Self::Error> {
        let header = Header::try_from(archive)?;
        let record_length = header.record_length();
        let count = header.count();
        let mut records = Vec::with_capacity(count);
        for index in 0..count {
            let offset = Header::LENGTH + index * record_length;
            let record_bytes = archive.try_bytes::<{ Record::LENGTH }>(offset);
            let Ok(record_bytes) = record_bytes else {
                let bytes = archive.length();
                return Err(Error::Length { bytes });
            };
            let record = Record::from(<[u8; Record::LENGTH]>::from(record_bytes));
            records.push(record);
        }
        let first_length = Header::LENGTH + count * record_length;
        let contents_offset = header.contents_offset();
        if contents_offset < first_length || archive.length() < contents_offset {
            let bytes = archive.length();
            return Err(Error::Length { bytes });
        }
        let field_length = Header::field_length(header.block_length());
        let length_bytes = &archive.bytes()[first_length..contents_offset];
        let block_count = length_bytes.len() / field_length;
        let mut blocks = Vec::with_capacity(block_count);
        let mut offset = contents_offset;
        for index in 0..block_count {
            let field = Self::field(length_bytes, index, field_length);
            let length = match field {
                0 => header.block_length(),
                length => length,
            };
            let chunk = archive.chunk(offset, length);
            let Some(chunk) = chunk else {
                let bytes = archive.length();
                return Err(Error::Length { bytes });
            };
            let block = Block::from(chunk);
            blocks.push(block);
            offset += length;
        }
        let remnant = archive.blob(offset).unwrap_or_default();
        Ok(Self {
            header,
            records,
            blocks,
            remnant,
        })
    }
}

impl TryFrom<Blob> for Archive {
    type Error = Error;

    fn try_from(archive: Blob) -> Result<Self, Self::Error> {
        Self::try_from(&archive)
    }
}

impl From<Archive> for Blob {
    fn from(archive: Archive) -> Self {
        Self::from(&archive)
    }
}

impl From<&Archive> for Blob {
    fn from(archive: &Archive) -> Self {
        let mut bytes = Self::default();
        let header_bytes: [u8; Header::LENGTH] = (&archive.header).into();
        bytes.extend(Self::from(header_bytes));
        for record in &archive.records {
            let record_bytes: [u8; Record::LENGTH] = record.into();
            bytes.extend(Self::from(record_bytes));
        }
        let block_length = archive.header.block_length();
        let field_length = Header::field_length(block_length);
        for block in &archive.blocks {
            let length = match block.length() == block_length {
                true => 0,
                false => block.length(),
            };
            let length_bytes = (length as u64).to_be_bytes();
            let offset = length_bytes.len() - field_length;
            bytes.extend(Self::from(&length_bytes[offset..]));
        }
        for block in &archive.blocks {
            bytes.extend(block.bytes().clone());
        }
        bytes.extend(archive.remnant.clone());
        bytes
    }
}

impl From<(&Files, Version)> for Archive {
    fn from((files, version): (&Files, Version)) -> Self {
        let block_length = Header::BLOCK;
        let manifest = files.manifest();
        let bytes = manifest.bytes();
        let mut blocks = Vec::new();
        let mut records = Vec::new();
        let digest = Digest::default();
        let file_length = bytes.length();
        let first_block = blocks.len();
        let record = Record::from((digest, first_block, file_length, 0));
        records.push(record);
        for chunk in Self::chunks(&bytes, block_length) {
            let block = Block::from(&chunk);
            blocks.push(block);
        }
        for file in files {
            let digest = Digest::from(file.path());
            let file_length = file.length();
            let first_block = blocks.len();
            let record = Record::from((digest, first_block, file_length, 0));
            records.push(record);
            for chunk in Self::chunks(file.bytes(), block_length) {
                let block = Block::from(&chunk);
                blocks.push(block);
            }
        }
        let count = records.len();
        let field_length = Header::field_length(block_length);
        let records_length = Header::RECORD_LENGTH * count;
        let blocks_length = field_length * blocks.len();
        let contents_offset = Header::LENGTH + records_length + blocks_length;
        let header = Header::from((version, contents_offset, count, block_length));
        for record in &mut records {
            let first_block = record.first_block();
            let blocks = blocks.get(..first_block).unwrap_or_default();
            let length: usize = blocks.iter().map(Block::length).sum();
            let offset = contents_offset + length;
            let digest = *record.digest();
            let file_length = record.file_length();
            *record = Record::from((digest, first_block, file_length, offset));
        }
        let remnant = Blob::default();
        Self {
            header,
            records,
            blocks,
            remnant,
        }
    }
}

impl From<&Files> for Archive {
    fn from(files: &Files) -> Self {
        let version = Version::default();
        Self::from((files, version))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::archive::file_path::FilePath;
    use crate::core::archive::files::File;
    use bytes::byte::Byte;
    use serde_test::{Token, assert_de_tokens_error, assert_tokens};

    #[test]
    fn an_index_no_record_of_the_archive_has_is_an_error() {
        let path = FilePath::try_from("net-patch9.bin").unwrap();
        let file = File::from((path, Blob::from(vec![7; 200])));
        let files = Files::from(vec![file]);
        let archive = Archive::from(&files);

        let record = archive.try_record(9);

        let count: usize = 9;
        let error = manifest::Error::Manifest { count };
        assert_eq!(record.unwrap_err(), Error::from(error));
    }

    #[test]
    fn a_block_beyond_the_blocks_of_the_archive_begins_nowhere() {
        let path = FilePath::try_from("net-patch9.bin").unwrap();
        let file = File::from((path, Blob::from(vec![7; 200])));
        let files = Files::from(vec![file]);
        let archive = Archive::from(&files);

        let offset = archive.offset(99);

        assert_eq!(offset, None);
    }

    #[test]
    fn an_index_no_record_names_unpacks_to_nothing() {
        let path = FilePath::try_from("net-patch9.bin").unwrap();
        let file = File::from((path, Blob::from(vec![7; 200])));
        let files = Files::from(vec![file]);
        let archive = Archive::from(&files);

        let blob = archive.try_unpack(9);

        let count: usize = 9;
        let error = manifest::Error::Manifest { count };
        assert_eq!(blob.unwrap_err(), Error::from(error));
    }

    #[test]
    fn a_record_whose_blocks_the_archive_does_not_have_is_an_error() {
        let path = FilePath::try_from("net-patch9.bin").unwrap();
        let file = File::from((path, Blob::from(vec![7; 200])));
        let files = Files::from(vec![file]);
        let mut archive = Archive::from(&files);
        archive.blocks.clear();

        let blob = archive.try_unpack(1);

        let bytes: usize = 1;
        assert_eq!(blob.unwrap_err(), Error::Unpack { bytes });
    }

    #[test]
    fn a_block_that_states_it_is_packed_and_is_not_is_an_error() {
        let path = FilePath::try_from("net-patch9.bin").unwrap();
        let file = File::from((path, Blob::from(vec![7; 200])));
        let files = Files::from(vec![file]);
        let mut archive = Archive::from(&files);
        let packing = Blob::from(vec![0x78, 0x9C, 0xFF, 0xFF, 0xFF, 0xFF]);
        archive.blocks[1] = Block::from(packing);

        let blob = archive.try_unpack(1);

        let bytes: usize = 6;
        let error = block::Error::Unpack { bytes };
        assert_eq!(blob.unwrap_err(), Error::from(error));
    }

    #[test]
    fn a_field_narrower_than_a_length_of_this_machine_is_padded_in_front() {
        let length_bytes = Blob::from(vec![0, 0, 0x01, 0x02]);

        let field = Archive::field(length_bytes.bytes(), 1, 2);

        assert_eq!(field, 0x0102);
    }

    #[test]
    fn an_archive_whose_contents_begin_inside_its_own_records_is_an_error() {
        let path = FilePath::try_from("net-patch9.bin").unwrap();
        let file = File::from((path, Blob::from(vec![7; 200])));
        let files = Files::from(vec![file]);
        let archive = Archive::from(&files);
        let mut bytes = Vec::<u8>::from(&Blob::from(&archive));
        bytes[0x0C..0x10].copy_from_slice(&u32::to_be_bytes(0x20));
        let bytes = Blob::from(bytes);

        let archive = Archive::try_from(&bytes);

        let length = bytes.length();
        assert_eq!(archive.unwrap_err(), Error::Length { bytes: length });
    }

    #[test]
    fn an_archive_that_ends_before_its_contents_begin_is_an_error() {
        let path = FilePath::try_from("net-patch9.bin").unwrap();
        let file = File::from((path, Blob::from(vec![7; 200])));
        let files = Files::from(vec![file]);
        let archive = Archive::from(&files);
        let mut bytes = Vec::<u8>::from(&Blob::from(&archive));
        bytes[0x0C..0x10].copy_from_slice(&u32::to_be_bytes(0xFFFF));
        let bytes = Blob::from(bytes);

        let archive = Archive::try_from(&bytes);

        let length = bytes.length();
        assert_eq!(archive.unwrap_err(), Error::Length { bytes: length });
    }

    #[test]
    fn an_archive_that_ends_before_a_block_it_names_is_an_error() {
        let path = FilePath::try_from("net-patch9.bin").unwrap();
        let file = File::from((path, Blob::from(vec![7; 200])));
        let files = Files::from(vec![file]);
        let archive = Archive::from(&files);
        let mut bytes = Vec::<u8>::from(&Blob::from(&archive));
        let first_length = Header::LENGTH + archive.count() * Header::RECORD_LENGTH;
        bytes[first_length..first_length + 2].copy_from_slice(&u16::to_be_bytes(0xFFFF));
        let bytes = Blob::from(bytes);

        let archive = Archive::try_from(&bytes);

        let length = bytes.length();
        assert_eq!(archive.unwrap_err(), Error::Length { bytes: length });
    }

    #[test]
    fn a_block_that_fills_all_of_a_block_states_a_length_of_zero_and_reads_back() {
        let path = FilePath::try_from("net-patch9.bin").unwrap();
        let mut content = Vec::with_capacity(Header::BLOCK);
        let mut state: u32 = 0x12345678;
        for _ in 0..Header::BLOCK {
            state = state.wrapping_mul(1664525).wrapping_add(1013904223);
            let byte = u8::try_from(state >> 24).unwrap();
            content.push(byte);
        }
        let file = File::from((path, Blob::from(content)));
        let files = Files::from(vec![file]);
        let archive = Archive::from(&files);

        let bytes = Blob::from(&archive);

        let same_archive = Archive::try_from(&bytes).unwrap();
        assert_eq!(same_archive, archive);
        assert_eq!(archive.blocks[1].length(), Header::BLOCK);
    }

    #[test]
    fn an_archive_of_one_file_goes_out_to_bytes_and_comes_back_the_same() {
        let path = FilePath::try_from("net-patch9.bin").unwrap();
        let file = File::from((path, Blob::from(vec![7; 200000])));
        let files = Files::from(vec![file]);
        let archive = Archive::from(&files);

        let bytes = Blob::from(&archive);

        let same_archive = Archive::try_from(&bytes).unwrap();
        assert_eq!(same_archive, archive);
    }

    #[test]
    fn the_files_that_went_in_are_the_files_that_come_out() {
        let root_path = FilePath::try_from("one.pak").unwrap();
        let deep_path = FilePath::try_from("build/main/two.pak").unwrap();
        let files = Files::from(vec![
            File::from((root_path, Blob::from(vec![1; 70000]))),
            File::from((deep_path, Blob::from(b"a short one".as_slice()))),
        ]);
        let archive = Archive::from(&files);

        let same_files = Files::try_from(&archive).unwrap();

        assert_eq!(same_files, files);
    }

    #[test]
    fn a_file_of_no_bytes_goes_in_and_comes_back_out() {
        let path = FilePath::try_from("empty.bin").unwrap();
        let files = Files::from(vec![File::from((path, Blob::default()))]);
        let archive = Archive::from(&files);

        let same_files = Files::try_from(&archive).unwrap();

        assert_eq!(same_files, files);
    }

    #[test]
    fn an_archive_of_no_files_at_all_keeps_nothing_but_its_manifest() {
        let files = Files::from(Vec::new());

        let archive = Archive::from(&files);

        assert_eq!(archive.count(), 1);
        assert_eq!(Files::try_from(&archive).unwrap(), files);
    }

    #[test]
    fn a_file_that_does_not_begin_with_psar_is_no_archive() {
        let bytes = Blob::from(*b"PSAF\0\0\0\0");

        let archive = Archive::try_from(&bytes);

        assert!(archive.is_err());
    }

    #[test]
    fn the_bytes_behind_the_last_block_of_an_archive_go_back_out_too() {
        let path = FilePath::try_from("one.pak").unwrap();
        let files = Files::from(vec![File::from((path, Blob::from(vec![1, 2, 3])))]);
        let archive = Archive::from(&files);
        let mut bytes = Blob::from(&archive);
        bytes.extend(Blob::from(*b"what no record reaches"));

        let bytes_archive = Archive::try_from(&bytes).unwrap();

        assert_eq!(Blob::from(&bytes_archive), bytes);
        assert_ne!(bytes_archive, archive);
    }

    #[test]
    fn the_files_of_an_archive_with_bytes_behind_its_last_block_are_the_files_that_went_in() {
        let path = FilePath::try_from("one.pak").unwrap();
        let files = Files::from(vec![File::from((path, Blob::from(vec![1, 2, 3])))]);
        let archive = Archive::from(&files);
        let mut bytes = Blob::from(&archive);
        bytes.extend(Blob::from(*b"what no record reaches"));
        let bytes_archive = Archive::try_from(&bytes).unwrap();

        let same_files = Files::try_from(&bytes_archive).unwrap();

        assert_eq!(same_files, files);
    }

    #[test]
    fn an_archive_of_no_bytes_behind_its_last_block_goes_back_out_with_none() {
        let path = FilePath::try_from("one.pak").unwrap();
        let files = Files::from(vec![File::from((path, Blob::from(vec![1, 2, 3])))]);
        let archive = Archive::from(&files);
        let bytes = Blob::from(&archive);

        let same_archive = Archive::try_from(&bytes).unwrap();

        assert_eq!(same_archive, archive);
        assert_eq!(Blob::from(&same_archive), bytes);
    }

    #[test]
    fn an_archive_that_ends_before_its_table_of_contents_is_an_error() {
        let path = FilePath::try_from("one.pak").unwrap();
        let files = Files::from(vec![File::from((path, Blob::from(vec![1, 2, 3])))]);
        let archive = Archive::from(&files);
        let bytes = Blob::from(&archive);
        let piece = bytes.chunk(0, 40).unwrap();

        let response = Archive::try_from(&piece);

        let bytes: usize = 40;
        assert_eq!(response.unwrap_err(), Error::Length { bytes });
    }

    #[test]
    fn a_record_whose_digest_is_not_the_digest_of_its_path_is_an_error() {
        let path = FilePath::try_from("one.pak").unwrap();
        let files = Files::from(vec![File::from((path, Blob::from(vec![1, 2, 3])))]);
        let archive = Archive::from(&files);
        let mut bytes = Blob::from(&archive);
        let digest = Header::LENGTH + Record::LENGTH;
        let byte = bytes.byte(digest).unwrap() ^ Byte::from(0xFF);
        bytes.try_write(digest, vec![byte]).unwrap();
        let bytes_archive = Archive::try_from(&bytes).unwrap();

        let files = Files::try_from(&bytes_archive);

        let letters = String::from("one.pak");
        assert_eq!(files.unwrap_err(), cursor::Error::Digest { letters });
    }

    #[test]
    fn where_a_block_begins_follows_from_the_blocks_in_front_of_it() {
        let path = FilePath::try_from("one.pak").unwrap();
        let files = Files::from(vec![File::from((path, Blob::from(vec![9; 100000])))]);
        let archive = Archive::from(&files);

        let offset = archive.offset(0);

        assert_eq!(offset, Some(archive.header.contents_offset()));
    }

    #[test]
    fn an_archive_goes_out_and_comes_back_as_the_bytes_of_the_file_it_is() {
        let path = FilePath::try_from("one.pak").unwrap();
        let files = Files::from(vec![File::from((path, Blob::from(vec![1, 2, 3])))]);
        let archive = Archive::from(&files);
        let bytes = Vec::<u8>::from(Blob::from(&archive));

        let mut tokens = vec![Token::Seq {
            len: Some(bytes.len()),
        }];
        tokens.extend(bytes.iter().copied().map(Token::U8));
        tokens.push(Token::SeqEnd);

        assert_tokens(&archive, &tokens);
    }

    #[test]
    fn a_file_that_does_not_begin_with_psar_is_no_archive_on_the_way_back_either() {
        let text = String::from("PSAF");
        let bytes = b"PSAF\0\0\0\0";
        let mut tokens = vec![Token::Seq {
            len: Some(bytes.len()),
        }];
        tokens.extend(bytes.iter().copied().map(Token::U8));
        tokens.push(Token::SeqEnd);

        let error_text = header::Error::Archive { text }.to_string();

        assert_de_tokens_error::<Archive>(&tokens, &error_text);
    }
}
