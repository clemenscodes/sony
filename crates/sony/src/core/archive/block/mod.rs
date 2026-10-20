pub(super) mod error;

use bytes::blob::Blob;
use bytes::byte::Byte;
use ddd::{DomainLayer, Layered, ValueObject};
use flate2::Compression;
use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;
use serde::{Deserialize, Serialize};
use std::io;
use std::io::{Read, Write};

pub(super) use error::Error;

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Default, Serialize, Deserialize)]
#[serde(transparent)]
pub(super) struct Block {
    bytes: Blob,
}

impl Layered for Block {
    type Layer = DomainLayer;
}

impl ValueObject for Block {}

impl Block {
    const STREAM: Byte = Byte::new(0x78);
    const EFFORT: u32 = 9;

    pub(super) fn bytes(&self) -> &Blob {
        &self.bytes
    }

    pub(super) fn length(&self) -> usize {
        self.bytes.length()
    }

    pub(super) fn has_packing(&self) -> bool {
        self.bytes.byte(0) == Some(Self::STREAM)
    }

    pub(super) fn try_unpack(&self, block_length: usize) -> Result<Blob, Error> {
        if !self.has_packing() {
            return Ok(self.bytes.clone());
        }
        let bytes = Vec::<u8>::from(&self.bytes);
        let ceiling = u64::try_from(block_length)
            .unwrap_or(u64::MAX)
            .saturating_add(1);
        let mut decoder = ZlibDecoder::new(bytes.as_slice()).take(ceiling);
        let mut blob = Vec::with_capacity(block_length);
        if io::copy(&mut decoder, &mut blob).is_err() || blob.len() > block_length {
            let bytes = self.bytes.length();
            return Err(Error::Unpack { bytes });
        }
        Ok(Blob::from(blob))
    }
}

impl From<Blob> for Block {
    fn from(bytes: Blob) -> Self {
        Self { bytes }
    }
}

impl From<&Blob> for Block {
    fn from(chunk: &Blob) -> Self {
        let chunk_bytes = Vec::<u8>::from(chunk);
        let effort = Compression::new(Block::EFFORT);
        let mut encoder = ZlibEncoder::new(Vec::new(), effort);
        let packing = encoder
            .write_all(&chunk_bytes)
            .and_then(|()| encoder.finish());
        let bytes = match packing {
            Ok(blob) if blob.len() < chunk_bytes.len() => Blob::from(blob),
            _ => chunk.clone(),
        };
        Self { bytes }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_test::{Token, assert_tokens};

    #[test]
    fn a_block_that_was_packed_comes_back_out_of_its_packing() {
        let chunk = Blob::from(vec![7; 4096]);
        let block = Block::from(&chunk);

        let blob = block.try_unpack(65536).unwrap();

        assert_eq!(blob, chunk);
    }

    #[test]
    fn a_chunk_that_would_grow_by_packing_goes_out_as_it_is() {
        let chunk = Blob::from(b"net-patch9.bin".as_slice());

        let block = Block::from(&chunk);

        assert_eq!(block.bytes(), &chunk);
        assert!(!block.has_packing());
    }

    #[test]
    fn a_block_that_was_not_packed_is_taken_as_it_is() {
        let bytes = Blob::from(b"net-patch9.bin".as_slice());
        let block = Block::from(bytes.clone());

        let blob = block.try_unpack(65536).unwrap();

        assert_eq!(blob, bytes);
    }

    #[test]
    fn a_block_of_no_bytes_at_all_is_taken_as_it_is() {
        let block = Block::from(Blob::default());

        let blob = block.try_unpack(65536).unwrap();

        assert!(blob.is_empty());
    }

    #[test]
    fn bytes_that_begin_like_a_stream_and_are_none_are_errors() {
        let bytes = Blob::from(vec![0x78, 0xDA, 0x00, 0x01, 0x02]);
        let block = Block::from(bytes);

        let blob = block.try_unpack(65536);

        let bytes: usize = 5;
        assert_eq!(blob.unwrap_err(), Error::Unpack { bytes });
    }

    #[test]
    fn a_block_may_never_unpack_to_more_than_one_block() {
        let chunk = Blob::from(vec![0; 65536]);
        let block = Block::from(&chunk);

        let blob = block.try_unpack(1024);

        assert!(blob.is_err());
    }

    #[test]
    fn a_block_goes_out_and_comes_back_as_the_bytes_it_keeps() {
        let block = Block::from(Blob::from(b"net-patch9.bin".as_slice()));
        let bytes = Vec::<u8>::from(block.bytes());

        let mut tokens = vec![Token::Seq {
            len: Some(bytes.len()),
        }];
        tokens.extend(bytes.iter().copied().map(Token::U8));
        tokens.push(Token::SeqEnd);

        assert_tokens(&block, &tokens);
    }

    #[test]
    fn a_block_of_no_bytes_goes_out_and_comes_back_as_no_bytes() {
        let block = Block::default();

        let tokens = [Token::Seq { len: Some(0) }, Token::SeqEnd];

        assert_tokens(&block, &tokens);
    }
}
