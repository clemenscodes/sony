pub(super) mod error;
mod packer;
mod version;

pub(super) use version::Version;

use crate::core::archive::header::packer::Packer;
use bytes::Bytes;
use bytes::Integer;
use bytes::blob::Blob;
use ddd::{DomainLayer, Layered, ValueObject};
use serde::{Deserialize, Serialize};

pub(super) use error::Error;

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Copy, Serialize, Deserialize)]
pub(super) struct Header {
    version: Version,
    packer: Packer,
    contents_offset: usize,
    record_length: usize,
    count: usize,
    block_length: usize,
    flags: Bytes<4>,
}

impl Layered for Header {
    type Layer = DomainLayer;
}

impl ValueObject for Header {}

impl Header {
    pub(super) const LENGTH: usize = 32;
    pub(super) const RECORD_LENGTH: usize = 30;
    pub(super) const BLOCK: usize = 65536;

    const MAGIC: [u8; 4] = *b"PSAR";
    const CEILING: usize = 256;

    pub(super) fn contents_offset(&self) -> usize {
        self.contents_offset
    }

    pub(super) fn record_length(&self) -> usize {
        self.record_length
    }

    pub(super) fn count(&self) -> usize {
        self.count
    }

    pub(super) fn block_length(&self) -> usize {
        self.block_length
    }

    pub(super) fn field_length(block_length: usize) -> usize {
        let mut field_length = 1;
        let mut ceiling = Self::CEILING;
        while block_length > ceiling {
            field_length += 1;
            ceiling = usize::saturating_mul(ceiling, Self::CEILING);
        }
        field_length
    }
}

impl From<&Header> for [u8; Header::LENGTH] {
    fn from(header: &Header) -> Self {
        let mut bytes = [0; Header::LENGTH];
        bytes[0x00..0x04].copy_from_slice(&Header::MAGIC);
        let major = <[u8; Version::FIELD_LENGTH]>::from(header.version.major().bytes());
        let minor = <[u8; Version::FIELD_LENGTH]>::from(header.version.minor().bytes());
        bytes[0x04..0x06].copy_from_slice(&major);
        bytes[0x06..0x08].copy_from_slice(&minor);
        bytes[0x08..0x0C].copy_from_slice(header.packer.bytes());
        let contents_offset = header.contents_offset as u32;
        let record_length = header.record_length as u32;
        let count = header.count as u32;
        let block_length = header.block_length as u32;
        bytes[0x0C..0x10].copy_from_slice(&contents_offset.to_be_bytes());
        bytes[0x10..0x14].copy_from_slice(&record_length.to_be_bytes());
        bytes[0x14..0x18].copy_from_slice(&count.to_be_bytes());
        bytes[0x18..0x1C].copy_from_slice(&block_length.to_be_bytes());
        let flags = <[u8; 4]>::from(header.flags);
        bytes[0x1C..0x20].copy_from_slice(&flags);
        bytes
    }
}

impl TryFrom<&Blob> for Header {
    type Error = Error;

    fn try_from(archive: &Blob) -> Result<Self, Self::Error> {
        let magic = archive.chunk(0, Self::MAGIC.len());
        if magic != Some(Blob::from(Self::MAGIC)) {
            let magic = Vec::<u8>::from(&magic.unwrap_or_default());
            let text = String::from_utf8_lossy(&magic).to_string();
            return Err(Error::Archive { text });
        }
        let header_bytes = archive.try_bytes::<{ Header::LENGTH }>(0);
        let Ok(header_bytes) = header_bytes else {
            let bytes = archive.length();
            return Err(Error::Length { bytes });
        };
        let header_bytes = <[u8; Header::LENGTH]>::from(header_bytes);
        let mut version_bytes = [0; Version::FIELD_LENGTH];
        version_bytes.copy_from_slice(&header_bytes[0x04..0x06]);
        let major = Integer::from(version_bytes);
        version_bytes.copy_from_slice(&header_bytes[0x06..0x08]);
        let minor = Integer::from(version_bytes);
        let version = Version::try_from((major, minor))?;
        let mut packer_bytes = [0; Packer::LENGTH];
        packer_bytes.copy_from_slice(&header_bytes[0x08..0x0C]);
        let packer = Packer::try_from(packer_bytes)?;
        let mut field_bytes = [0; 4];
        field_bytes.copy_from_slice(&header_bytes[0x0C..0x10]);
        let contents_offset = Integer::from(field_bytes);
        field_bytes.copy_from_slice(&header_bytes[0x10..0x14]);
        let record_length = Integer::from(field_bytes);
        field_bytes.copy_from_slice(&header_bytes[0x14..0x18]);
        let count = Integer::from(field_bytes);
        field_bytes.copy_from_slice(&header_bytes[0x18..0x1C]);
        let block_length = Integer::from(field_bytes);
        let contents_offset = usize::from(contents_offset);
        let record_length = usize::from(record_length);
        let count = usize::from(count);
        let block_length = usize::from(block_length);
        field_bytes.copy_from_slice(&header_bytes[0x1C..Self::LENGTH]);
        let flags = Bytes::from(field_bytes);
        Ok(Self {
            version,
            packer,
            contents_offset,
            record_length,
            count,
            block_length,
            flags,
        })
    }
}

impl From<(Version, usize, usize, usize)> for Header {
    fn from(
        (version, contents_offset, count, block_length): (Version, usize, usize, usize),
    ) -> Self {
        let packer = Packer::default();
        let record_length = Self::RECORD_LENGTH;
        let flags = Bytes::default();
        Self {
            version,
            packer,
            contents_offset,
            record_length,
            count,
            block_length,
            flags,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_test::{Token, assert_de_tokens_error, assert_tokens};

    #[test]
    fn the_header_of_the_archive_the_store_serves_is_read_field_for_field() {
        let mut archive = Blob::zeroes(Header::LENGTH);
        archive.try_write(0, *b"PSAR").unwrap();
        archive.try_write(0x04, [0x00, 0x01, 0x00, 0x04]).unwrap();
        archive.try_write(0x08, *b"zlib").unwrap();
        archive.try_write(0x0C, u32::to_be_bytes(100)).unwrap();
        archive.try_write(0x10, u32::to_be_bytes(30)).unwrap();
        archive.try_write(0x14, u32::to_be_bytes(2)).unwrap();
        archive.try_write(0x18, u32::to_be_bytes(65536)).unwrap();

        let header = Header::try_from(&archive).unwrap();

        let version = Version::try_from((Integer::from([0, 1]), Integer::from([0, 4]))).unwrap();
        let contents_offset: usize = 100;
        let count: usize = 2;
        let block_length: usize = 65536;
        assert_eq!(
            header,
            Header::from((version, contents_offset, count, block_length))
        );
        assert_eq!(header.contents_offset(), contents_offset);
        assert_eq!(header.record_length(), 30);
        assert_eq!(header.count(), count);
        assert_eq!(header.block_length(), block_length);
    }

    #[test]
    fn an_archive_version_this_repository_has_never_read_is_an_error() {
        let mut bytes = vec![0; Header::LENGTH];
        bytes[0x00..0x04].copy_from_slice(b"PSAR");
        bytes[0x04..0x06].copy_from_slice(&u16::to_be_bytes(1));
        bytes[0x06..0x08].copy_from_slice(&u16::to_be_bytes(9));
        bytes[0x08..0x0C].copy_from_slice(b"zlib");
        let archive = Blob::from(bytes);

        let header = Header::try_from(&archive);

        assert!(header.is_err());
    }

    #[test]
    fn a_packer_this_crate_cannot_unpack_is_an_error() {
        let mut bytes = vec![0; Header::LENGTH];
        bytes[0x00..0x04].copy_from_slice(b"PSAR");
        bytes[0x04..0x06].copy_from_slice(&u16::to_be_bytes(1));
        bytes[0x06..0x08].copy_from_slice(&u16::to_be_bytes(4));
        bytes[0x08..0x0C].copy_from_slice(b"lzma");
        let archive = Blob::from(bytes);

        let header = Header::try_from(&archive);

        assert!(header.is_err());
    }

    #[test]
    fn a_file_that_does_not_begin_with_psar_is_no_archive() {
        let archive = Blob::from(*b"PSAF and more");

        let header = Header::try_from(&archive);

        let text = String::from("PSAF");
        assert_eq!(header.unwrap_err(), Error::Archive { text });
    }

    #[test]
    fn a_file_shorter_than_a_header_is_an_error() {
        let archive = Blob::from(*b"PSAR\0\x01\0\x03zlib");

        let header = Header::try_from(&archive);

        let bytes: usize = 12;
        assert_eq!(header.unwrap_err(), Error::Length { bytes });
    }

    #[test]
    fn a_block_of_sixty_five_thousand_bytes_needs_two_bytes_for_its_length() {
        let block_length: usize = 65536;

        let field_length = Header::field_length(block_length);

        assert_eq!(field_length, 2);
    }

    #[test]
    fn a_block_of_two_hundred_and_fifty_six_bytes_needs_one_byte_for_its_length() {
        let block_length: usize = 256;

        let field_length = Header::field_length(block_length);

        assert_eq!(field_length, 1);
    }

    #[test]
    fn a_block_of_one_byte_more_than_the_ceiling_of_two_bytes_needs_three() {
        let block_length: usize = 65537;

        let field_length = Header::field_length(block_length);

        assert_eq!(field_length, 3);
    }

    #[test]
    fn a_header_goes_out_to_bytes_and_comes_back_the_same() {
        let version = Version::try_from((Integer::from([0, 1]), Integer::from([0, 4]))).unwrap();
        let block_length = Header::BLOCK;
        let header = Header::from((version, 100, 2, block_length));

        let bytes = Blob::from(<[u8; Header::LENGTH]>::from(&header));

        let same_header = Header::try_from(&bytes).unwrap();
        assert_eq!(same_header, header);
    }

    #[test]
    fn a_header_goes_out_and_comes_back_field_for_field() {
        let version = Version::try_from((Integer::from([0, 1]), Integer::from([0, 4]))).unwrap();
        let block_length = Header::BLOCK;
        let header = Header::from((version, 100, 2, block_length));

        let tokens = [
            Token::Struct {
                name: "Header",
                len: 7,
            },
            Token::Str("version"),
            Token::Tuple { len: 2 },
            Token::Str("0001"),
            Token::Str("0004"),
            Token::TupleEnd,
            Token::Str("packer"),
            Token::Tuple {
                len: Packer::LENGTH,
            },
            Token::U8(b'z'),
            Token::U8(b'l'),
            Token::U8(b'i'),
            Token::U8(b'b'),
            Token::TupleEnd,
            Token::Str("contents_offset"),
            Token::U64(100),
            Token::Str("record_length"),
            Token::U64(30),
            Token::Str("count"),
            Token::U64(2),
            Token::Str("block_length"),
            Token::U64(65536),
            Token::Str("flags"),
            Token::Str("00000000"),
            Token::StructEnd,
        ];

        assert_tokens(&header, &tokens);
    }

    #[test]
    fn a_header_of_an_archive_packed_with_lzma_is_an_error_on_the_way_back_too() {
        let text = String::from("lzma");
        let tokens = [
            Token::Struct {
                name: "Header",
                len: 7,
            },
            Token::Str("version"),
            Token::Tuple { len: 2 },
            Token::Str("0001"),
            Token::Str("0004"),
            Token::TupleEnd,
            Token::Str("packer"),
            Token::Tuple {
                len: Packer::LENGTH,
            },
            Token::U8(b'l'),
            Token::U8(b'z'),
            Token::U8(b'm'),
            Token::U8(b'a'),
            Token::TupleEnd,
        ];

        let error_text = packer::Error::Packer { text }.to_string();

        assert_de_tokens_error::<Header>(&tokens, &error_text);
    }
}
