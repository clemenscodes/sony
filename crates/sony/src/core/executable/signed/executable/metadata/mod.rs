pub(super) mod error;
mod section;

use crate::core::executable::signed::executable::header::Header;
use crate::core::executable::signed::executable::license::License;
use crate::core::executable::signed::executable::metadata::section::Section;
use crate::core::executable::signed::executable::vault::Keyset;
use aes::{Aes128, Aes256};
use bytes::Integer;
use bytes::blob::Blob;
use cbc::Decryptor;
use cbc::cipher::{BlockModeDecrypt, KeyIvInit, StreamCipher};
use ctr::Ctr128BE;
use ddd::{DomainLayer, Layered, ValueObject};
use serde::{Deserialize, Serialize};

pub(super) use error::Error;

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Default, Serialize, Deserialize)]
pub(super) struct Metadata {
    sections: Vec<Section>,
    keys: Vec<[u8; Metadata::KEY]>,
}

impl Layered for Metadata {
    type Layer = DomainLayer;
}

impl ValueObject for Metadata {}

impl Metadata {
    const KEY: usize = 16;
    const KEY_AND_VECTOR: usize = 0x40;
    const HEADER: usize = 0x20;

    pub(super) fn sections(&self) -> &[Section] {
        &self.sections
    }

    pub(super) fn key(&self, index: usize) -> Option<&[u8; Self::KEY]> {
        self.keys.get(index)
    }

    fn try_key_and_vector(
        signed_bytes: &Blob,
        header: &Header,
        license: &License,
        keyset: &Keyset,
    ) -> Result<[u8; Metadata::KEY_AND_VECTOR], Error> {
        let offset = header.metadata_offset() + Header::LENGTH;
        let field_bytes = signed_bytes.try_bytes::<{ Self::KEY_AND_VECTOR }>(offset);
        let Ok(field_bytes) = field_bytes else {
            let bytes = signed_bytes.length();
            return Err(Error::Length { bytes });
        };
        let mut key_and_vector = <[u8; Self::KEY_AND_VECTOR]>::from(field_bytes);
        let key = license.key();
        if let Some(key) = key {
            let key = (&key).into();
            let vector = (&[0; 16]).into();
            let mut decryptor = Decryptor::<Aes128>::new(key, vector);
            let (blocks, _) = key_and_vector.as_chunks_mut::<16>();
            for block in blocks {
                let block = block.into();
                decryptor.decrypt_block(block);
            }
        }
        let key = keyset.key().into();
        let vector = keyset.vector().into();
        let mut decryptor = Decryptor::<Aes256>::new(key, vector);
        let (blocks, _) = key_and_vector.as_chunks_mut::<16>();
        for block in blocks {
            let block = block.into();
            decryptor.decrypt_block(block);
        }
        let padding = key_and_vector[16..32].iter().chain(&key_and_vector[48..64]);
        let zero = padding.into_iter().all(|byte| *byte == 0);
        if !zero {
            return Err(Error::Metadata);
        }
        Ok(key_and_vector)
    }
}

impl TryFrom<(&Blob, &Header, &License, &Keyset)> for Metadata {
    type Error = Error;

    fn try_from(
        (signed_bytes, header, license, keyset): (&Blob, &Header, &License, &Keyset),
    ) -> Result<Self, Self::Error> {
        let key_and_vector = Self::try_key_and_vector(signed_bytes, header, license, keyset)?;
        let metadata_offset = header.metadata_offset();
        let offset = metadata_offset + Header::LENGTH + Self::KEY_AND_VECTOR;
        let length = header.length();
        if length < offset {
            let bytes = signed_bytes.length();
            return Err(Error::Length { bytes });
        }
        let header_bytes = signed_bytes.chunk(offset, length - offset);
        let Some(header_bytes) = header_bytes else {
            let bytes = signed_bytes.length();
            return Err(Error::Length { bytes });
        };
        let mut headers = Vec::<u8>::from(&header_bytes);
        let key = key_and_vector[..16].try_into().unwrap_or([0; 16]);
        let vector = key_and_vector[32..48].try_into().unwrap_or([0; 16]);
        let key: [u8; 16] = key;
        let vector: [u8; 16] = vector;
        let mut stream = Ctr128BE::<Aes128>::new((&key).into(), (&vector).into());
        stream.apply_keystream(&mut headers);
        let headers = Blob::from(headers);
        let section_count_bytes = headers.try_bytes::<4>(0x0C);
        let key_count_bytes = headers.try_bytes::<4>(0x10);
        let (Ok(section_count_bytes), Ok(key_count_bytes)) = (section_count_bytes, key_count_bytes)
        else {
            let bytes = signed_bytes.length();
            return Err(Error::Length { bytes });
        };
        let section_count = Integer::<4>::from(section_count_bytes);
        let key_count = Integer::<4>::from(key_count_bytes);
        let section_count = usize::from(section_count);
        let key_count = usize::from(key_count);
        let mut sections = Vec::with_capacity(section_count);
        for index in 0..section_count {
            let offset = Self::HEADER + Section::LENGTH * index;
            let section_bytes = headers.try_bytes::<{ Section::LENGTH }>(offset);
            let Ok(section_bytes) = section_bytes else {
                let bytes = signed_bytes.length();
                return Err(Error::Length { bytes });
            };
            let bytes = <[u8; Section::LENGTH]>::from(section_bytes);
            let section = Section::from(&bytes);
            sections.push(section);
        }
        let mut keys = Vec::with_capacity(key_count);
        let offset = Self::HEADER + Section::LENGTH * section_count;
        for index in 0..key_count {
            let offset = offset + Self::KEY * index;
            let key_bytes = headers.try_bytes::<{ Self::KEY }>(offset);
            let Ok(key_bytes) = key_bytes else {
                let bytes = signed_bytes.length();
                return Err(Error::Length { bytes });
            };
            let key = <[u8; Self::KEY]>::from(key_bytes);
            keys.push(key);
        }
        Ok(Self { sections, keys })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cbc::Encryptor;
    use cbc::cipher::BlockModeEncrypt;
    use serde_test::{Token, assert_tokens};

    #[test]
    fn metadata_a_keyset_opens_yields_the_sections_and_the_keys_it_states() {
        let keyset_key = [7; Keyset::KEY];
        let keyset_vector = [9; Keyset::VECTOR];
        let revision = Integer::from([0, 1]);
        let program = Integer::from([0, 0, 0, 8]);
        let keyset = Keyset::from((revision, program, keyset_key, keyset_vector));
        let mut key_and_vector = [0; Metadata::KEY_AND_VECTOR];
        key_and_vector[..16].copy_from_slice(&[0x11; 16]);
        key_and_vector[32..48].copy_from_slice(&[0x22; 16]);
        let section_key: [u8; 16] = key_and_vector[..16].try_into().unwrap();
        let section_vector: [u8; 16] = key_and_vector[32..48].try_into().unwrap();
        let mut encryptor = Encryptor::<Aes256>::new((&keyset_key).into(), (&keyset_vector).into());
        let (blocks, _) = key_and_vector.as_chunks_mut::<16>();
        for block in blocks {
            let block = block.into();
            encryptor.encrypt_block(block);
        }
        let mut headers = vec![0; Metadata::HEADER + Section::LENGTH + Metadata::KEY * 2];
        headers[0x0C..0x10].copy_from_slice(&u32::to_be_bytes(1));
        headers[0x10..0x14].copy_from_slice(&u32::to_be_bytes(2));
        let section = Metadata::HEADER;
        headers[section..section + 8].copy_from_slice(&u64::to_be_bytes(0xA80));
        headers[section + 0x08..section + 0x10].copy_from_slice(&u64::to_be_bytes(0x40));
        let keys = Metadata::HEADER + Section::LENGTH;
        headers[keys..keys + Metadata::KEY].copy_from_slice(&[0x33; Metadata::KEY]);
        headers[keys + Metadata::KEY..].copy_from_slice(&[0x44; Metadata::KEY]);
        let mut stream = Ctr128BE::<Aes128>::new((&section_key).into(), (&section_vector).into());
        stream.apply_keystream(&mut headers);
        let metadata_offset = 0x100;
        let headers_offset = metadata_offset + Header::LENGTH + Metadata::KEY_AND_VECTOR;
        let mut signed_bytes = vec![0; 0x200];
        signed_bytes[..4].copy_from_slice(b"SCE\0");
        signed_bytes[0x0C..0x10]
            .copy_from_slice(&u32::to_be_bytes(u32::try_from(metadata_offset).unwrap()));
        let length = headers_offset + headers.len();
        signed_bytes[0x10..0x18].copy_from_slice(&u64::to_be_bytes(u64::try_from(length).unwrap()));
        let field = metadata_offset + Header::LENGTH;
        signed_bytes[field..field + Metadata::KEY_AND_VECTOR].copy_from_slice(&key_and_vector);
        signed_bytes[headers_offset..headers_offset + headers.len()].copy_from_slice(&headers);
        let signed_bytes = Blob::from(signed_bytes);
        let header = Header::try_from(&signed_bytes).unwrap();

        let metadata =
            Metadata::try_from((&signed_bytes, &header, &License::None, &keyset)).unwrap();

        assert_eq!(metadata.sections().len(), 1);
        assert_eq!(metadata.sections()[0].offset(), 0xA80);
        assert_eq!(metadata.key(0), Some(&[0x33; Metadata::KEY]));
        assert_eq!(metadata.key(1), Some(&[0x44; Metadata::KEY]));
        assert_eq!(metadata.key(2), None);
    }

    #[test]
    fn metadata_of_a_file_with_a_free_license_is_opened_with_that_key_first() {
        let keyset_key = [7; Keyset::KEY];
        let keyset_vector = [9; Keyset::VECTOR];
        let revision = Integer::from([0, 1]);
        let program = Integer::from([0, 0, 0, 8]);
        let keyset = Keyset::from((revision, program, keyset_key, keyset_vector));
        let mut key_and_vector = [0; Metadata::KEY_AND_VECTOR];
        key_and_vector[..16].copy_from_slice(&[0x11; 16]);
        key_and_vector[32..48].copy_from_slice(&[0x22; 16]);
        let section_key: [u8; 16] = key_and_vector[..16].try_into().unwrap();
        let section_vector: [u8; 16] = key_and_vector[32..48].try_into().unwrap();
        let mut encryptor = Encryptor::<Aes256>::new((&keyset_key).into(), (&keyset_vector).into());
        let (blocks, _) = key_and_vector.as_chunks_mut::<16>();
        for block in blocks {
            let block = block.into();
            encryptor.encrypt_block(block);
        }
        let license_key = License::Free.key().unwrap();
        let mut encryptor = Encryptor::<Aes128>::new((&license_key).into(), (&[0; 16]).into());
        let (blocks, _) = key_and_vector.as_chunks_mut::<16>();
        for block in blocks {
            let block = block.into();
            encryptor.encrypt_block(block);
        }
        let mut headers = vec![0; Metadata::HEADER];
        let mut stream = Ctr128BE::<Aes128>::new((&section_key).into(), (&section_vector).into());
        stream.apply_keystream(&mut headers);
        let metadata_offset = 0x100;
        let headers_offset = metadata_offset + Header::LENGTH + Metadata::KEY_AND_VECTOR;
        let mut signed_bytes = vec![0; 0x200];
        signed_bytes[..4].copy_from_slice(b"SCE\0");
        signed_bytes[0x0C..0x10]
            .copy_from_slice(&u32::to_be_bytes(u32::try_from(metadata_offset).unwrap()));
        let length = headers_offset + headers.len();
        signed_bytes[0x10..0x18].copy_from_slice(&u64::to_be_bytes(u64::try_from(length).unwrap()));
        let field = metadata_offset + Header::LENGTH;
        signed_bytes[field..field + Metadata::KEY_AND_VECTOR].copy_from_slice(&key_and_vector);
        signed_bytes[headers_offset..headers_offset + headers.len()].copy_from_slice(&headers);
        let signed_bytes = Blob::from(signed_bytes);
        let header = Header::try_from(&signed_bytes).unwrap();

        let metadata =
            Metadata::try_from((&signed_bytes, &header, &License::Free, &keyset)).unwrap();

        assert!(metadata.sections().is_empty());
        assert_eq!(metadata.key(0), None);
    }

    #[test]
    fn metadata_whose_padding_is_not_zero_states_the_keyset_is_the_wrong_one() {
        let keyset_key = [7; Keyset::KEY];
        let keyset_vector = [9; Keyset::VECTOR];
        let revision = Integer::from([0, 1]);
        let program = Integer::from([0, 0, 0, 8]);
        let keyset = Keyset::from((revision, program, keyset_key, keyset_vector));
        let mut key_and_vector = [0; Metadata::KEY_AND_VECTOR];
        key_and_vector[16] = 1;
        let mut encryptor = Encryptor::<Aes256>::new((&keyset_key).into(), (&keyset_vector).into());
        let (blocks, _) = key_and_vector.as_chunks_mut::<16>();
        for block in blocks {
            let block = block.into();
            encryptor.encrypt_block(block);
        }
        let metadata_offset = 0x100;
        let mut signed_bytes = vec![0; 0x200];
        signed_bytes[..4].copy_from_slice(b"SCE\0");
        signed_bytes[0x0C..0x10]
            .copy_from_slice(&u32::to_be_bytes(u32::try_from(metadata_offset).unwrap()));
        let field = metadata_offset + Header::LENGTH;
        signed_bytes[field..field + Metadata::KEY_AND_VECTOR].copy_from_slice(&key_and_vector);
        let signed_bytes = Blob::from(signed_bytes);
        let header = Header::try_from(&signed_bytes).unwrap();

        let metadata = Metadata::try_from((&signed_bytes, &header, &License::None, &keyset));

        assert_eq!(metadata.unwrap_err(), Error::Metadata);
    }

    #[test]
    fn a_file_that_ends_before_the_key_and_the_vector_do_is_an_error() {
        let keyset_key = [7; Keyset::KEY];
        let keyset_vector = [9; Keyset::VECTOR];
        let revision = Integer::from([0, 1]);
        let program = Integer::from([0, 0, 0, 8]);
        let keyset = Keyset::from((revision, program, keyset_key, keyset_vector));
        let mut signed_bytes = vec![0; 0x100];
        signed_bytes[..4].copy_from_slice(b"SCE\0");
        signed_bytes[0x0C..0x10].copy_from_slice(&u32::to_be_bytes(0x100));
        let signed_bytes = Blob::from(signed_bytes);
        let header = Header::try_from(&signed_bytes).unwrap();

        let metadata = Metadata::try_from((&signed_bytes, &header, &License::None, &keyset));

        let bytes: usize = 0x100;
        assert_eq!(metadata.unwrap_err(), Error::Length { bytes });
    }

    #[test]
    fn a_file_that_states_it_is_shorter_than_its_own_metadata_is_an_error() {
        let keyset_key = [7; Keyset::KEY];
        let keyset_vector = [9; Keyset::VECTOR];
        let revision = Integer::from([0, 1]);
        let program = Integer::from([0, 0, 0, 8]);
        let keyset = Keyset::from((revision, program, keyset_key, keyset_vector));
        let mut key_and_vector = [0; Metadata::KEY_AND_VECTOR];
        let mut encryptor = Encryptor::<Aes256>::new((&keyset_key).into(), (&keyset_vector).into());
        let (blocks, _) = key_and_vector.as_chunks_mut::<16>();
        for block in blocks {
            let block = block.into();
            encryptor.encrypt_block(block);
        }
        let metadata_offset = 0x100;
        let mut signed_bytes = vec![0; 0x200];
        signed_bytes[..4].copy_from_slice(b"SCE\0");
        signed_bytes[0x0C..0x10]
            .copy_from_slice(&u32::to_be_bytes(u32::try_from(metadata_offset).unwrap()));
        signed_bytes[0x10..0x18].copy_from_slice(&u64::to_be_bytes(0x100));
        let field = metadata_offset + Header::LENGTH;
        signed_bytes[field..field + Metadata::KEY_AND_VECTOR].copy_from_slice(&key_and_vector);
        let signed_bytes = Blob::from(signed_bytes);
        let header = Header::try_from(&signed_bytes).unwrap();

        let metadata = Metadata::try_from((&signed_bytes, &header, &License::None, &keyset));

        let bytes: usize = 0x200;
        assert_eq!(metadata.unwrap_err(), Error::Length { bytes });
    }

    #[test]
    fn a_file_that_states_its_metadata_reaches_past_its_own_end_is_an_error() {
        let keyset_key = [7; Keyset::KEY];
        let keyset_vector = [9; Keyset::VECTOR];
        let revision = Integer::from([0, 1]);
        let program = Integer::from([0, 0, 0, 8]);
        let keyset = Keyset::from((revision, program, keyset_key, keyset_vector));
        let mut key_and_vector = [0; Metadata::KEY_AND_VECTOR];
        let mut encryptor = Encryptor::<Aes256>::new((&keyset_key).into(), (&keyset_vector).into());
        let (blocks, _) = key_and_vector.as_chunks_mut::<16>();
        for block in blocks {
            let block = block.into();
            encryptor.encrypt_block(block);
        }
        let metadata_offset = 0x100;
        let mut signed_bytes = vec![0; 0x200];
        signed_bytes[..4].copy_from_slice(b"SCE\0");
        signed_bytes[0x0C..0x10]
            .copy_from_slice(&u32::to_be_bytes(u32::try_from(metadata_offset).unwrap()));
        signed_bytes[0x10..0x18].copy_from_slice(&u64::to_be_bytes(0x1000));
        let field = metadata_offset + Header::LENGTH;
        signed_bytes[field..field + Metadata::KEY_AND_VECTOR].copy_from_slice(&key_and_vector);
        let signed_bytes = Blob::from(signed_bytes);
        let header = Header::try_from(&signed_bytes).unwrap();

        let metadata = Metadata::try_from((&signed_bytes, &header, &License::None, &keyset));

        let bytes: usize = 0x200;
        assert_eq!(metadata.unwrap_err(), Error::Length { bytes });
    }

    #[test]
    fn metadata_too_short_for_the_counts_it_must_state_is_an_error() {
        let keyset_key = [7; Keyset::KEY];
        let keyset_vector = [9; Keyset::VECTOR];
        let revision = Integer::from([0, 1]);
        let program = Integer::from([0, 0, 0, 8]);
        let keyset = Keyset::from((revision, program, keyset_key, keyset_vector));
        let mut key_and_vector = [0; Metadata::KEY_AND_VECTOR];
        let mut encryptor = Encryptor::<Aes256>::new((&keyset_key).into(), (&keyset_vector).into());
        let (blocks, _) = key_and_vector.as_chunks_mut::<16>();
        for block in blocks {
            let block = block.into();
            encryptor.encrypt_block(block);
        }
        let metadata_offset = 0x100;
        let headers_offset = metadata_offset + Header::LENGTH + Metadata::KEY_AND_VECTOR;
        let mut signed_bytes = vec![0; 0x200];
        signed_bytes[..4].copy_from_slice(b"SCE\0");
        signed_bytes[0x0C..0x10]
            .copy_from_slice(&u32::to_be_bytes(u32::try_from(metadata_offset).unwrap()));
        let length = headers_offset + 0x10;
        signed_bytes[0x10..0x18].copy_from_slice(&u64::to_be_bytes(u64::try_from(length).unwrap()));
        let field = metadata_offset + Header::LENGTH;
        signed_bytes[field..field + Metadata::KEY_AND_VECTOR].copy_from_slice(&key_and_vector);
        let signed_bytes = Blob::from(signed_bytes);
        let header = Header::try_from(&signed_bytes).unwrap();

        let metadata = Metadata::try_from((&signed_bytes, &header, &License::None, &keyset));

        let bytes: usize = 0x200;
        assert_eq!(metadata.unwrap_err(), Error::Length { bytes });
    }

    #[test]
    fn metadata_that_names_more_sections_than_it_lists_is_an_error() {
        let keyset_key = [7; Keyset::KEY];
        let keyset_vector = [9; Keyset::VECTOR];
        let revision = Integer::from([0, 1]);
        let program = Integer::from([0, 0, 0, 8]);
        let keyset = Keyset::from((revision, program, keyset_key, keyset_vector));
        let mut key_and_vector = [0; Metadata::KEY_AND_VECTOR];
        key_and_vector[..16].copy_from_slice(&[0x11; 16]);
        key_and_vector[32..48].copy_from_slice(&[0x22; 16]);
        let section_key: [u8; 16] = key_and_vector[..16].try_into().unwrap();
        let section_vector: [u8; 16] = key_and_vector[32..48].try_into().unwrap();
        let mut encryptor = Encryptor::<Aes256>::new((&keyset_key).into(), (&keyset_vector).into());
        let (blocks, _) = key_and_vector.as_chunks_mut::<16>();
        for block in blocks {
            let block = block.into();
            encryptor.encrypt_block(block);
        }
        let mut headers = vec![0; Metadata::HEADER + Section::LENGTH];
        headers[0x0C..0x10].copy_from_slice(&u32::to_be_bytes(2));
        let mut stream = Ctr128BE::<Aes128>::new((&section_key).into(), (&section_vector).into());
        stream.apply_keystream(&mut headers);
        let metadata_offset = 0x100;
        let headers_offset = metadata_offset + Header::LENGTH + Metadata::KEY_AND_VECTOR;
        let mut signed_bytes = vec![0; 0x200];
        signed_bytes[..4].copy_from_slice(b"SCE\0");
        signed_bytes[0x0C..0x10]
            .copy_from_slice(&u32::to_be_bytes(u32::try_from(metadata_offset).unwrap()));
        let length = headers_offset + headers.len();
        signed_bytes[0x10..0x18].copy_from_slice(&u64::to_be_bytes(u64::try_from(length).unwrap()));
        let field = metadata_offset + Header::LENGTH;
        signed_bytes[field..field + Metadata::KEY_AND_VECTOR].copy_from_slice(&key_and_vector);
        signed_bytes[headers_offset..headers_offset + headers.len()].copy_from_slice(&headers);
        let signed_bytes = Blob::from(signed_bytes);
        let header = Header::try_from(&signed_bytes).unwrap();

        let metadata = Metadata::try_from((&signed_bytes, &header, &License::None, &keyset));

        let bytes: usize = 0x200;
        assert_eq!(metadata.unwrap_err(), Error::Length { bytes });
    }

    #[test]
    fn metadata_that_names_more_keys_than_it_lists_is_an_error() {
        let keyset_key = [7; Keyset::KEY];
        let keyset_vector = [9; Keyset::VECTOR];
        let revision = Integer::from([0, 1]);
        let program = Integer::from([0, 0, 0, 8]);
        let keyset = Keyset::from((revision, program, keyset_key, keyset_vector));
        let mut key_and_vector = [0; Metadata::KEY_AND_VECTOR];
        key_and_vector[..16].copy_from_slice(&[0x11; 16]);
        key_and_vector[32..48].copy_from_slice(&[0x22; 16]);
        let section_key: [u8; 16] = key_and_vector[..16].try_into().unwrap();
        let section_vector: [u8; 16] = key_and_vector[32..48].try_into().unwrap();
        let mut encryptor = Encryptor::<Aes256>::new((&keyset_key).into(), (&keyset_vector).into());
        let (blocks, _) = key_and_vector.as_chunks_mut::<16>();
        for block in blocks {
            let block = block.into();
            encryptor.encrypt_block(block);
        }
        let mut headers = vec![0; Metadata::HEADER + Metadata::KEY];
        headers[0x10..0x14].copy_from_slice(&u32::to_be_bytes(2));
        let mut stream = Ctr128BE::<Aes128>::new((&section_key).into(), (&section_vector).into());
        stream.apply_keystream(&mut headers);
        let metadata_offset = 0x100;
        let headers_offset = metadata_offset + Header::LENGTH + Metadata::KEY_AND_VECTOR;
        let mut signed_bytes = vec![0; 0x200];
        signed_bytes[..4].copy_from_slice(b"SCE\0");
        signed_bytes[0x0C..0x10]
            .copy_from_slice(&u32::to_be_bytes(u32::try_from(metadata_offset).unwrap()));
        let length = headers_offset + headers.len();
        signed_bytes[0x10..0x18].copy_from_slice(&u64::to_be_bytes(u64::try_from(length).unwrap()));
        let field = metadata_offset + Header::LENGTH;
        signed_bytes[field..field + Metadata::KEY_AND_VECTOR].copy_from_slice(&key_and_vector);
        signed_bytes[headers_offset..headers_offset + headers.len()].copy_from_slice(&headers);
        let signed_bytes = Blob::from(signed_bytes);
        let header = Header::try_from(&signed_bytes).unwrap();

        let metadata = Metadata::try_from((&signed_bytes, &header, &License::None, &keyset));

        let bytes: usize = 0x200;
        assert_eq!(metadata.unwrap_err(), Error::Length { bytes });
    }

    #[test]
    fn metadata_goes_out_and_comes_back_as_its_sections_and_its_keys() {
        let mut bytes = [0; Section::LENGTH];
        bytes[0x00..0x08].copy_from_slice(&u64::to_be_bytes(0xA80));
        let sections = vec![Section::from(&bytes)];
        let key = [3; Metadata::KEY];
        let keys = vec![key];
        let metadata = Metadata { sections, keys };

        let mut tokens = vec![
            Token::Struct {
                name: "Metadata",
                len: 2,
            },
            Token::Str("sections"),
            Token::Seq { len: Some(1) },
            Token::Struct {
                name: "Section",
                len: 8,
            },
            Token::Str("offset"),
            Token::U64(0xA80),
            Token::Str("length"),
            Token::U64(0),
            Token::Str("kind"),
            Token::Str("00000000"),
            Token::Str("program_index"),
            Token::U64(0),
            Token::Str("key"),
            Token::Str("00000000"),
            Token::Str("key_index"),
            Token::U64(0),
            Token::Str("vector_index"),
            Token::U64(0),
            Token::Str("packing"),
            Token::Str("00000000"),
            Token::StructEnd,
            Token::SeqEnd,
            Token::Str("keys"),
            Token::Seq { len: Some(1) },
            Token::Tuple { len: Metadata::KEY },
        ];
        tokens.extend(key.iter().copied().map(Token::U8));
        tokens.extend([Token::TupleEnd, Token::SeqEnd, Token::StructEnd]);

        assert_tokens(&metadata, &tokens);
    }

    #[test]
    fn metadata_of_no_sections_and_no_keys_goes_out_and_comes_back_as_nothing() {
        let metadata = Metadata::default();

        let tokens = [
            Token::Struct {
                name: "Metadata",
                len: 2,
            },
            Token::Str("sections"),
            Token::Seq { len: Some(0) },
            Token::SeqEnd,
            Token::Str("keys"),
            Token::Seq { len: Some(0) },
            Token::SeqEnd,
            Token::StructEnd,
        ];

        assert_tokens(&metadata, &tokens);
    }
}
