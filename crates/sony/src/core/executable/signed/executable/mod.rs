pub mod error;
mod header;
mod license;
mod metadata;
mod vault;

use crate::core::executable::Executable;
use crate::core::executable::signed::Signed;

use crate::core::executable::signed::executable::header::Header;
use crate::core::executable::signed::executable::license::License;
use crate::core::executable::signed::executable::metadata::Metadata;
use crate::core::executable::signed::executable::vault::Vault;
use aes::Aes128;
use aes::cipher::{KeyIvInit, StreamCipher};
use bytes::Integer;
use bytes::blob::Blob;
use bytes::byte::Byte;
use ctr::Ctr128BE;
use ddd::{DomainLayer, Layered, ValueObject};
use flate2::read::ZlibDecoder;
use serde::{Deserialize, Serialize};
use std::io;

pub use error::Error;

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Default, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SignedExecutable {
    bytes: Blob,
}

impl Layered for SignedExecutable {
    type Layer = DomainLayer;
}

impl ValueObject for SignedExecutable {}

impl SignedExecutable {
    const ELF: usize = 0x40;
    const PROGRAM: usize = 0x38;
    const FIELD_LENGTH: usize = 2;

    fn try_programs(
        signed_bytes: &Blob,
        header: &Header,
        elf: &[u8; SignedExecutable::ELF],
    ) -> Result<Vec<[u8; SignedExecutable::PROGRAM]>, Error> {
        let mut count_bytes = [0; Self::FIELD_LENGTH];
        count_bytes.copy_from_slice(&elf[0x38..0x3A]);
        let program_count = usize::from(Integer::from(count_bytes));
        let programs_offset = header.programs_offset();
        let mut programs = Vec::with_capacity(program_count);
        for index in 0..program_count {
            let offset = programs_offset + index * Self::PROGRAM;
            let program = signed_bytes.try_bytes::<{ SignedExecutable::PROGRAM }>(offset);
            let Ok(program) = program else {
                let bytes = signed_bytes.length();
                return Err(Error::Length { bytes });
            };
            programs.push(<[u8; SignedExecutable::PROGRAM]>::from(program));
        }
        Ok(programs)
    }

    fn try_write_sections(
        signed_bytes: &Blob,
        header: &Header,
        elf: &[u8; SignedExecutable::ELF],
        executable_bytes: &mut [Byte],
    ) -> Result<(), Error> {
        let sections_offset = header.sections_offset();
        if sections_offset == 0 {
            return Ok(());
        }
        let mut word_bytes = [0; 8];
        word_bytes.copy_from_slice(&elf[0x28..0x30]);
        let offset = usize::from(Integer::from(word_bytes));
        let mut count_bytes = [0; Self::FIELD_LENGTH];
        count_bytes.copy_from_slice(&elf[0x3A..0x3C]);
        let length = usize::from(Integer::from(count_bytes));
        count_bytes.copy_from_slice(&elf[0x3C..0x3E]);
        let section_count = usize::from(Integer::from(count_bytes));
        let bytes = section_count * length;
        let section_bytes = signed_bytes.chunk(sections_offset, bytes);
        let Some(section_bytes) = section_bytes else {
            let bytes = signed_bytes.length();
            return Err(Error::Length { bytes });
        };
        if executable_bytes.len() < offset + bytes {
            let bytes = signed_bytes.length();
            return Err(Error::Length { bytes });
        }
        executable_bytes[offset..offset + bytes].copy_from_slice(section_bytes.bytes());
        Ok(())
    }
}

impl Signed for SignedExecutable {
    type Error = Error;

    fn try_decrypt(&self) -> Result<Executable, Self::Error> {
        let signed_bytes = &self.bytes;
        let header = Header::try_from(signed_bytes)?;
        let license = License::try_from((&header, signed_bytes))?;
        let vault = Vault::from(());
        let keyset = vault.try_keyset(&header)?;
        let metadata = Metadata::try_from((signed_bytes, &header, &license, &keyset))?;
        let executable_length = header.executable_length();
        let mut executable_bytes = vec![Byte::default(); executable_length];
        let elf = signed_bytes.try_bytes::<{ SignedExecutable::ELF }>(header.elf_offset());
        let Ok(elf) = elf else {
            let bytes = signed_bytes.length();
            return Err(Error::Length { bytes });
        };
        let elf = <[u8; SignedExecutable::ELF]>::from(elf);
        let elf_blob = Blob::from(elf);
        executable_bytes[..Self::ELF].copy_from_slice(elf_blob.bytes());
        let programs = Self::try_programs(signed_bytes, &header, &elf)?;
        let mut word_bytes = [0; 8];
        word_bytes.copy_from_slice(&elf[0x20..0x28]);
        let offset = usize::from(Integer::from(word_bytes));
        let length = programs.len() * Self::PROGRAM;
        if executable_bytes.len() < offset + length {
            let bytes = signed_bytes.length();
            return Err(Error::Length { bytes });
        }
        for (index, program) in programs.iter().enumerate() {
            let offset = offset + index * Self::PROGRAM;
            let program_blob = Blob::from(*program);
            executable_bytes[offset..offset + Self::PROGRAM].copy_from_slice(program_blob.bytes());
        }
        for section in metadata.sections() {
            if !section.is_program() || !section.has_key() {
                continue;
            }
            let section_bytes = signed_bytes.chunk(section.offset(), section.length());
            let Some(section_bytes) = section_bytes else {
                let bytes = signed_bytes.length();
                return Err(Error::Length { bytes });
            };
            let key = metadata.key(section.key_index());
            let vector = metadata.key(section.vector_index());
            let (Some(key), Some(vector)) = (key, vector) else {
                let error = metadata::Error::Metadata;
                return Err(Error::from(error));
            };
            let mut bytes = Vec::<u8>::from(&section_bytes);
            let mut stream = Ctr128BE::<Aes128>::new(key.into(), vector.into());
            stream.apply_keystream(&mut bytes);
            let program_index = section.program_index();
            let program = programs.get(program_index);
            let Some(program) = program else {
                let error = metadata::Error::Metadata;
                return Err(Error::from(error));
            };
            let mut word_bytes = [0; 8];
            word_bytes.copy_from_slice(&program[0x08..0x10]);
            let offset = usize::from(Integer::from(word_bytes));
            word_bytes.copy_from_slice(&program[0x20..0x28]);
            let length = usize::from(Integer::from(word_bytes));
            let bytes = match section.has_packing() {
                true => {
                    let mut decoder = ZlibDecoder::new(bytes.as_slice());
                    let mut blob = Vec::new();
                    if io::copy(&mut decoder, &mut blob).is_err() {
                        let bytes = bytes.len();
                        return Err(Error::Unpack { bytes });
                    }
                    blob
                }
                false => bytes,
            };
            let mut bytes = Blob::from(bytes);
            bytes.truncate(length);
            let length = bytes.length();
            if executable_bytes.len() < offset + length {
                let bytes = signed_bytes.length();
                return Err(Error::Length { bytes });
            }
            executable_bytes[offset..offset + length].copy_from_slice(bytes.bytes());
        }
        Self::try_write_sections(signed_bytes, &header, &elf, &mut executable_bytes)?;
        Ok(Executable::from(Blob::from(executable_bytes)))
    }
}

impl From<Blob> for SignedExecutable {
    fn from(bytes: Blob) -> Self {
        Self { bytes }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use aes::Aes256;
    use cbc::Encryptor;
    use cbc::cipher::BlockModeEncrypt;
    use flate2::Compression;
    use flate2::write::ZlibEncoder;
    use serde_test::{Token, assert_tokens};
    use std::io::Write;

    #[test]
    fn a_signed_executable_without_sections_yields_its_elf_and_its_programs() {
        let mut signed_bytes = vec![0; 0x400];
        signed_bytes[..4].copy_from_slice(b"SCE\0");
        signed_bytes[0x08..0x0A].copy_from_slice(&u16::to_be_bytes(1));
        signed_bytes[0x0C..0x10].copy_from_slice(&u32::to_be_bytes(0x100));
        signed_bytes[0x18..0x20].copy_from_slice(&u64::to_be_bytes(0x200));
        signed_bytes[0x30..0x38].copy_from_slice(&u64::to_be_bytes(0x200));
        signed_bytes[0x38..0x40].copy_from_slice(&u64::to_be_bytes(0x240));
        signed_bytes[0x7C..0x80].copy_from_slice(&u32::to_be_bytes(8));
        signed_bytes[0x220..0x228].copy_from_slice(&u64::to_be_bytes(0x40));
        signed_bytes[0x238..0x23A].copy_from_slice(&u16::to_be_bytes(1));
        signed_bytes[0x240] = 0x77;
        let header = Header::try_from(&Blob::from(signed_bytes.clone())).unwrap();
        let keyset = Vault::from(()).try_keyset(&header).unwrap();
        let mut key_and_vector = [0; 0x40];
        let mut encryptor = Encryptor::<Aes256>::new(keyset.key().into(), keyset.vector().into());
        let (blocks, _) = key_and_vector.as_chunks_mut::<16>();
        for block in blocks {
            let block = block.into();
            encryptor.encrypt_block(block);
        }
        signed_bytes[0x120..0x160].copy_from_slice(&key_and_vector);
        let mut headers = vec![0; 0x20];
        let mut stream = Ctr128BE::<Aes128>::new((&[0; 16]).into(), (&[0; 16]).into());
        stream.apply_keystream(&mut headers);
        signed_bytes[0x160..0x180].copy_from_slice(&headers);
        signed_bytes[0x10..0x18].copy_from_slice(&u64::to_be_bytes(0x180));
        let signed_executable = SignedExecutable::from(Blob::from(signed_bytes));

        let executable = signed_executable.try_decrypt().unwrap();

        let bytes = executable.bytes();
        assert_eq!(bytes.length(), 0x200);
        assert_eq!(bytes.byte(0x40), Some(Byte::from(0x77)));
    }

    #[test]
    fn a_section_with_a_key_is_decrypted_into_the_offset_its_program_names() {
        let mut signed_bytes = vec![0; 0x400];
        signed_bytes[..4].copy_from_slice(b"SCE\0");
        signed_bytes[0x08..0x0A].copy_from_slice(&u16::to_be_bytes(1));
        signed_bytes[0x0C..0x10].copy_from_slice(&u32::to_be_bytes(0x100));
        signed_bytes[0x18..0x20].copy_from_slice(&u64::to_be_bytes(0x200));
        signed_bytes[0x30..0x38].copy_from_slice(&u64::to_be_bytes(0x200));
        signed_bytes[0x38..0x40].copy_from_slice(&u64::to_be_bytes(0x240));
        signed_bytes[0x7C..0x80].copy_from_slice(&u32::to_be_bytes(8));
        signed_bytes[0x220..0x228].copy_from_slice(&u64::to_be_bytes(0x40));
        signed_bytes[0x238..0x23A].copy_from_slice(&u16::to_be_bytes(1));
        signed_bytes[0x248..0x250].copy_from_slice(&u64::to_be_bytes(0x80));
        signed_bytes[0x260..0x268].copy_from_slice(&u64::to_be_bytes(0x20));
        let header = Header::try_from(&Blob::from(signed_bytes.clone())).unwrap();
        let keyset = Vault::from(()).try_keyset(&header).unwrap();
        let mut key_and_vector = [0; 0x40];
        let mut encryptor = Encryptor::<Aes256>::new(keyset.key().into(), keyset.vector().into());
        let (blocks, _) = key_and_vector.as_chunks_mut::<16>();
        for block in blocks {
            let block = block.into();
            encryptor.encrypt_block(block);
        }
        signed_bytes[0x120..0x160].copy_from_slice(&key_and_vector);
        let section_key = [0x33; 16];
        let section_vector = [0x44; 16];
        let mut headers = vec![0; 0x70];
        headers[0x0C..0x10].copy_from_slice(&u32::to_be_bytes(1));
        headers[0x10..0x14].copy_from_slice(&u32::to_be_bytes(2));
        headers[0x20..0x28].copy_from_slice(&u64::to_be_bytes(0x300));
        headers[0x28..0x30].copy_from_slice(&u64::to_be_bytes(0x20));
        headers[0x30..0x34].copy_from_slice(&u32::to_be_bytes(2));
        headers[0x40..0x44].copy_from_slice(&u32::to_be_bytes(3));
        headers[0x48..0x4C].copy_from_slice(&u32::to_be_bytes(1));
        headers[0x50..0x60].copy_from_slice(&section_key);
        headers[0x60..0x70].copy_from_slice(&section_vector);
        let mut stream = Ctr128BE::<Aes128>::new((&[0; 16]).into(), (&[0; 16]).into());
        stream.apply_keystream(&mut headers);
        signed_bytes[0x160..0x1D0].copy_from_slice(&headers);
        signed_bytes[0x10..0x18].copy_from_slice(&u64::to_be_bytes(0x1D0));
        let mut section = vec![0xAB; 0x20];
        let mut stream = Ctr128BE::<Aes128>::new((&section_key).into(), (&section_vector).into());
        stream.apply_keystream(&mut section);
        signed_bytes[0x300..0x320].copy_from_slice(&section);
        let signed_executable = SignedExecutable::from(Blob::from(signed_bytes));

        let executable = signed_executable.try_decrypt().unwrap();

        let bytes = executable.bytes();
        assert_eq!(bytes.byte(0x80), Some(Byte::from(0xAB)));
        assert_eq!(bytes.byte(0x9F), Some(Byte::from(0xAB)));
        assert_eq!(bytes.byte(0xA0), Some(Byte::default()));
    }

    #[test]
    fn a_packed_section_is_unpacked_into_the_offset_its_program_names() {
        let mut packing = ZlibEncoder::new(Vec::new(), Compression::new(9));
        packing.write_all(&[0xCD; 0x20]).unwrap();
        let packing = packing.finish().unwrap();
        let mut signed_bytes = vec![0; 0x400];
        signed_bytes[..4].copy_from_slice(b"SCE\0");
        signed_bytes[0x08..0x0A].copy_from_slice(&u16::to_be_bytes(1));
        signed_bytes[0x0C..0x10].copy_from_slice(&u32::to_be_bytes(0x100));
        signed_bytes[0x18..0x20].copy_from_slice(&u64::to_be_bytes(0x200));
        signed_bytes[0x30..0x38].copy_from_slice(&u64::to_be_bytes(0x200));
        signed_bytes[0x38..0x40].copy_from_slice(&u64::to_be_bytes(0x240));
        signed_bytes[0x7C..0x80].copy_from_slice(&u32::to_be_bytes(8));
        signed_bytes[0x220..0x228].copy_from_slice(&u64::to_be_bytes(0x40));
        signed_bytes[0x238..0x23A].copy_from_slice(&u16::to_be_bytes(1));
        signed_bytes[0x248..0x250].copy_from_slice(&u64::to_be_bytes(0x80));
        signed_bytes[0x260..0x268].copy_from_slice(&u64::to_be_bytes(0x20));
        let header = Header::try_from(&Blob::from(signed_bytes.clone())).unwrap();
        let keyset = Vault::from(()).try_keyset(&header).unwrap();
        let mut key_and_vector = [0; 0x40];
        let mut encryptor = Encryptor::<Aes256>::new(keyset.key().into(), keyset.vector().into());
        let (blocks, _) = key_and_vector.as_chunks_mut::<16>();
        for block in blocks {
            let block = block.into();
            encryptor.encrypt_block(block);
        }
        signed_bytes[0x120..0x160].copy_from_slice(&key_and_vector);
        let section_key = [0x33; 16];
        let section_vector = [0x44; 16];
        let mut headers = vec![0; 0x70];
        headers[0x0C..0x10].copy_from_slice(&u32::to_be_bytes(1));
        headers[0x10..0x14].copy_from_slice(&u32::to_be_bytes(2));
        headers[0x20..0x28].copy_from_slice(&u64::to_be_bytes(0x300));
        let packing_length = u64::try_from(packing.len()).unwrap();
        headers[0x28..0x30].copy_from_slice(&u64::to_be_bytes(packing_length));
        headers[0x30..0x34].copy_from_slice(&u32::to_be_bytes(2));
        headers[0x40..0x44].copy_from_slice(&u32::to_be_bytes(3));
        headers[0x48..0x4C].copy_from_slice(&u32::to_be_bytes(1));
        headers[0x4C..0x50].copy_from_slice(&u32::to_be_bytes(2));
        headers[0x50..0x60].copy_from_slice(&section_key);
        headers[0x60..0x70].copy_from_slice(&section_vector);
        let mut stream = Ctr128BE::<Aes128>::new((&[0; 16]).into(), (&[0; 16]).into());
        stream.apply_keystream(&mut headers);
        signed_bytes[0x160..0x1D0].copy_from_slice(&headers);
        signed_bytes[0x10..0x18].copy_from_slice(&u64::to_be_bytes(0x1D0));
        let mut section = packing.clone();
        let mut stream = Ctr128BE::<Aes128>::new((&section_key).into(), (&section_vector).into());
        stream.apply_keystream(&mut section);
        signed_bytes[0x300..0x300 + section.len()].copy_from_slice(&section);
        let signed_executable = SignedExecutable::from(Blob::from(signed_bytes));

        let executable = signed_executable.try_decrypt().unwrap();

        let bytes = executable.bytes();
        assert_eq!(bytes.byte(0x80), Some(Byte::from(0xCD)));
        assert_eq!(bytes.byte(0x9F), Some(Byte::from(0xCD)));
    }

    #[test]
    fn a_packed_section_that_does_not_unpack_is_an_error() {
        let mut signed_bytes = vec![0; 0x400];
        signed_bytes[..4].copy_from_slice(b"SCE\0");
        signed_bytes[0x08..0x0A].copy_from_slice(&u16::to_be_bytes(1));
        signed_bytes[0x0C..0x10].copy_from_slice(&u32::to_be_bytes(0x100));
        signed_bytes[0x18..0x20].copy_from_slice(&u64::to_be_bytes(0x200));
        signed_bytes[0x30..0x38].copy_from_slice(&u64::to_be_bytes(0x200));
        signed_bytes[0x38..0x40].copy_from_slice(&u64::to_be_bytes(0x240));
        signed_bytes[0x7C..0x80].copy_from_slice(&u32::to_be_bytes(8));
        signed_bytes[0x220..0x228].copy_from_slice(&u64::to_be_bytes(0x40));
        signed_bytes[0x238..0x23A].copy_from_slice(&u16::to_be_bytes(1));
        signed_bytes[0x248..0x250].copy_from_slice(&u64::to_be_bytes(0x80));
        signed_bytes[0x260..0x268].copy_from_slice(&u64::to_be_bytes(0x20));
        let header = Header::try_from(&Blob::from(signed_bytes.clone())).unwrap();
        let keyset = Vault::from(()).try_keyset(&header).unwrap();
        let mut key_and_vector = [0; 0x40];
        let mut encryptor = Encryptor::<Aes256>::new(keyset.key().into(), keyset.vector().into());
        let (blocks, _) = key_and_vector.as_chunks_mut::<16>();
        for block in blocks {
            let block = block.into();
            encryptor.encrypt_block(block);
        }
        signed_bytes[0x120..0x160].copy_from_slice(&key_and_vector);
        let section_key = [0x33; 16];
        let section_vector = [0x44; 16];
        let mut headers = vec![0; 0x70];
        headers[0x0C..0x10].copy_from_slice(&u32::to_be_bytes(1));
        headers[0x10..0x14].copy_from_slice(&u32::to_be_bytes(2));
        headers[0x20..0x28].copy_from_slice(&u64::to_be_bytes(0x300));
        headers[0x28..0x30].copy_from_slice(&u64::to_be_bytes(0x20));
        headers[0x30..0x34].copy_from_slice(&u32::to_be_bytes(2));
        headers[0x40..0x44].copy_from_slice(&u32::to_be_bytes(3));
        headers[0x48..0x4C].copy_from_slice(&u32::to_be_bytes(1));
        headers[0x4C..0x50].copy_from_slice(&u32::to_be_bytes(2));
        headers[0x50..0x60].copy_from_slice(&section_key);
        headers[0x60..0x70].copy_from_slice(&section_vector);
        let mut stream = Ctr128BE::<Aes128>::new((&[0; 16]).into(), (&[0; 16]).into());
        stream.apply_keystream(&mut headers);
        signed_bytes[0x160..0x1D0].copy_from_slice(&headers);
        signed_bytes[0x10..0x18].copy_from_slice(&u64::to_be_bytes(0x1D0));
        let mut section = vec![0xAB; 0x20];
        let mut stream = Ctr128BE::<Aes128>::new((&section_key).into(), (&section_vector).into());
        stream.apply_keystream(&mut section);
        signed_bytes[0x300..0x320].copy_from_slice(&section);
        let signed_executable = SignedExecutable::from(Blob::from(signed_bytes));

        let executable = signed_executable.try_decrypt();

        let bytes: usize = 0x20;
        assert_eq!(executable.unwrap_err(), Error::Unpack { bytes });
    }

    #[test]
    fn a_section_whose_program_reaches_past_the_executable_is_an_error() {
        let mut signed_bytes = vec![0; 0x400];
        signed_bytes[..4].copy_from_slice(b"SCE\0");
        signed_bytes[0x08..0x0A].copy_from_slice(&u16::to_be_bytes(1));
        signed_bytes[0x0C..0x10].copy_from_slice(&u32::to_be_bytes(0x100));
        signed_bytes[0x18..0x20].copy_from_slice(&u64::to_be_bytes(0x200));
        signed_bytes[0x30..0x38].copy_from_slice(&u64::to_be_bytes(0x200));
        signed_bytes[0x38..0x40].copy_from_slice(&u64::to_be_bytes(0x240));
        signed_bytes[0x7C..0x80].copy_from_slice(&u32::to_be_bytes(8));
        signed_bytes[0x220..0x228].copy_from_slice(&u64::to_be_bytes(0x40));
        signed_bytes[0x238..0x23A].copy_from_slice(&u16::to_be_bytes(1));
        signed_bytes[0x248..0x250].copy_from_slice(&u64::to_be_bytes(0x1F0));
        signed_bytes[0x260..0x268].copy_from_slice(&u64::to_be_bytes(0x20));
        let header = Header::try_from(&Blob::from(signed_bytes.clone())).unwrap();
        let keyset = Vault::from(()).try_keyset(&header).unwrap();
        let mut key_and_vector = [0; 0x40];
        let mut encryptor = Encryptor::<Aes256>::new(keyset.key().into(), keyset.vector().into());
        let (blocks, _) = key_and_vector.as_chunks_mut::<16>();
        for block in blocks {
            let block = block.into();
            encryptor.encrypt_block(block);
        }
        signed_bytes[0x120..0x160].copy_from_slice(&key_and_vector);
        let mut headers = vec![0; 0x70];
        headers[0x0C..0x10].copy_from_slice(&u32::to_be_bytes(1));
        headers[0x10..0x14].copy_from_slice(&u32::to_be_bytes(2));
        headers[0x20..0x28].copy_from_slice(&u64::to_be_bytes(0x300));
        headers[0x28..0x30].copy_from_slice(&u64::to_be_bytes(0x20));
        headers[0x30..0x34].copy_from_slice(&u32::to_be_bytes(2));
        headers[0x40..0x44].copy_from_slice(&u32::to_be_bytes(3));
        headers[0x48..0x4C].copy_from_slice(&u32::to_be_bytes(1));
        let mut stream = Ctr128BE::<Aes128>::new((&[0; 16]).into(), (&[0; 16]).into());
        stream.apply_keystream(&mut headers);
        signed_bytes[0x160..0x1D0].copy_from_slice(&headers);
        signed_bytes[0x10..0x18].copy_from_slice(&u64::to_be_bytes(0x1D0));
        let signed_executable = SignedExecutable::from(Blob::from(signed_bytes));

        let executable = signed_executable.try_decrypt();

        let bytes: usize = 0x400;
        assert_eq!(executable.unwrap_err(), Error::Length { bytes });
    }

    #[test]
    fn the_section_headers_of_an_elf_are_put_where_the_elf_names() {
        let mut signed_bytes = vec![0; 0x400];
        signed_bytes[..4].copy_from_slice(b"SCE\0");
        signed_bytes[0x08..0x0A].copy_from_slice(&u16::to_be_bytes(1));
        signed_bytes[0x0C..0x10].copy_from_slice(&u32::to_be_bytes(0x100));
        signed_bytes[0x18..0x20].copy_from_slice(&u64::to_be_bytes(0x200));
        signed_bytes[0x30..0x38].copy_from_slice(&u64::to_be_bytes(0x200));
        signed_bytes[0x38..0x40].copy_from_slice(&u64::to_be_bytes(0x240));
        signed_bytes[0x40..0x48].copy_from_slice(&u64::to_be_bytes(0x380));
        signed_bytes[0x7C..0x80].copy_from_slice(&u32::to_be_bytes(8));
        signed_bytes[0x220..0x228].copy_from_slice(&u64::to_be_bytes(0x40));
        signed_bytes[0x228..0x230].copy_from_slice(&u64::to_be_bytes(0xC0));
        signed_bytes[0x238..0x23A].copy_from_slice(&u16::to_be_bytes(1));
        signed_bytes[0x23A..0x23C].copy_from_slice(&u16::to_be_bytes(0x10));
        signed_bytes[0x23C..0x23E].copy_from_slice(&u16::to_be_bytes(2));
        signed_bytes[0x380] = 0x5A;
        let header = Header::try_from(&Blob::from(signed_bytes.clone())).unwrap();
        let keyset = Vault::from(()).try_keyset(&header).unwrap();
        let mut key_and_vector = [0; 0x40];
        let mut encryptor = Encryptor::<Aes256>::new(keyset.key().into(), keyset.vector().into());
        let (blocks, _) = key_and_vector.as_chunks_mut::<16>();
        for block in blocks {
            let block = block.into();
            encryptor.encrypt_block(block);
        }
        signed_bytes[0x120..0x160].copy_from_slice(&key_and_vector);
        let mut headers = vec![0; 0x20];
        let mut stream = Ctr128BE::<Aes128>::new((&[0; 16]).into(), (&[0; 16]).into());
        stream.apply_keystream(&mut headers);
        signed_bytes[0x160..0x180].copy_from_slice(&headers);
        signed_bytes[0x10..0x18].copy_from_slice(&u64::to_be_bytes(0x180));
        let signed_executable = SignedExecutable::from(Blob::from(signed_bytes));

        let executable = signed_executable.try_decrypt().unwrap();

        let bytes = executable.bytes();
        assert_eq!(bytes.byte(0xC0), Some(Byte::from(0x5A)));
    }

    #[test]
    fn section_headers_that_begin_past_the_end_of_the_file_are_errors() {
        let mut signed_bytes = vec![0; 0x400];
        signed_bytes[..4].copy_from_slice(b"SCE\0");
        signed_bytes[0x08..0x0A].copy_from_slice(&u16::to_be_bytes(1));
        signed_bytes[0x0C..0x10].copy_from_slice(&u32::to_be_bytes(0x100));
        signed_bytes[0x18..0x20].copy_from_slice(&u64::to_be_bytes(0x200));
        signed_bytes[0x30..0x38].copy_from_slice(&u64::to_be_bytes(0x200));
        signed_bytes[0x38..0x40].copy_from_slice(&u64::to_be_bytes(0x240));
        signed_bytes[0x40..0x48].copy_from_slice(&u64::to_be_bytes(0x3F0));
        signed_bytes[0x7C..0x80].copy_from_slice(&u32::to_be_bytes(8));
        signed_bytes[0x220..0x228].copy_from_slice(&u64::to_be_bytes(0x40));
        signed_bytes[0x228..0x230].copy_from_slice(&u64::to_be_bytes(0xC0));
        signed_bytes[0x238..0x23A].copy_from_slice(&u16::to_be_bytes(1));
        signed_bytes[0x23A..0x23C].copy_from_slice(&u16::to_be_bytes(0x10));
        signed_bytes[0x23C..0x23E].copy_from_slice(&u16::to_be_bytes(2));
        let header = Header::try_from(&Blob::from(signed_bytes.clone())).unwrap();
        let keyset = Vault::from(()).try_keyset(&header).unwrap();
        let mut key_and_vector = [0; 0x40];
        let mut encryptor = Encryptor::<Aes256>::new(keyset.key().into(), keyset.vector().into());
        let (blocks, _) = key_and_vector.as_chunks_mut::<16>();
        for block in blocks {
            let block = block.into();
            encryptor.encrypt_block(block);
        }
        signed_bytes[0x120..0x160].copy_from_slice(&key_and_vector);
        let mut headers = vec![0; 0x20];
        let mut stream = Ctr128BE::<Aes128>::new((&[0; 16]).into(), (&[0; 16]).into());
        stream.apply_keystream(&mut headers);
        signed_bytes[0x160..0x180].copy_from_slice(&headers);
        signed_bytes[0x10..0x18].copy_from_slice(&u64::to_be_bytes(0x180));
        let signed_executable = SignedExecutable::from(Blob::from(signed_bytes));

        let executable = signed_executable.try_decrypt();

        let bytes: usize = 0x400;
        assert_eq!(executable.unwrap_err(), Error::Length { bytes });
    }

    #[test]
    fn section_headers_that_reach_past_the_executable_are_errors() {
        let mut signed_bytes = vec![0; 0x400];
        signed_bytes[..4].copy_from_slice(b"SCE\0");
        signed_bytes[0x08..0x0A].copy_from_slice(&u16::to_be_bytes(1));
        signed_bytes[0x0C..0x10].copy_from_slice(&u32::to_be_bytes(0x100));
        signed_bytes[0x18..0x20].copy_from_slice(&u64::to_be_bytes(0x200));
        signed_bytes[0x30..0x38].copy_from_slice(&u64::to_be_bytes(0x200));
        signed_bytes[0x38..0x40].copy_from_slice(&u64::to_be_bytes(0x240));
        signed_bytes[0x40..0x48].copy_from_slice(&u64::to_be_bytes(0x380));
        signed_bytes[0x7C..0x80].copy_from_slice(&u32::to_be_bytes(8));
        signed_bytes[0x220..0x228].copy_from_slice(&u64::to_be_bytes(0x40));
        signed_bytes[0x228..0x230].copy_from_slice(&u64::to_be_bytes(0x1F0));
        signed_bytes[0x238..0x23A].copy_from_slice(&u16::to_be_bytes(1));
        signed_bytes[0x23A..0x23C].copy_from_slice(&u16::to_be_bytes(0x10));
        signed_bytes[0x23C..0x23E].copy_from_slice(&u16::to_be_bytes(2));
        let header = Header::try_from(&Blob::from(signed_bytes.clone())).unwrap();
        let keyset = Vault::from(()).try_keyset(&header).unwrap();
        let mut key_and_vector = [0; 0x40];
        let mut encryptor = Encryptor::<Aes256>::new(keyset.key().into(), keyset.vector().into());
        let (blocks, _) = key_and_vector.as_chunks_mut::<16>();
        for block in blocks {
            let block = block.into();
            encryptor.encrypt_block(block);
        }
        signed_bytes[0x120..0x160].copy_from_slice(&key_and_vector);
        let mut headers = vec![0; 0x20];
        let mut stream = Ctr128BE::<Aes128>::new((&[0; 16]).into(), (&[0; 16]).into());
        stream.apply_keystream(&mut headers);
        signed_bytes[0x160..0x180].copy_from_slice(&headers);
        signed_bytes[0x10..0x18].copy_from_slice(&u64::to_be_bytes(0x180));
        let signed_executable = SignedExecutable::from(Blob::from(signed_bytes));

        let executable = signed_executable.try_decrypt();

        let bytes: usize = 0x400;
        assert_eq!(executable.unwrap_err(), Error::Length { bytes });
    }

    #[test]
    fn a_section_that_is_no_program_is_passed_over() {
        let mut signed_bytes = vec![0; 0x400];
        signed_bytes[..4].copy_from_slice(b"SCE\0");
        signed_bytes[0x08..0x0A].copy_from_slice(&u16::to_be_bytes(1));
        signed_bytes[0x0C..0x10].copy_from_slice(&u32::to_be_bytes(0x100));
        signed_bytes[0x18..0x20].copy_from_slice(&u64::to_be_bytes(0x200));
        signed_bytes[0x30..0x38].copy_from_slice(&u64::to_be_bytes(0x200));
        signed_bytes[0x38..0x40].copy_from_slice(&u64::to_be_bytes(0x240));
        signed_bytes[0x7C..0x80].copy_from_slice(&u32::to_be_bytes(8));
        signed_bytes[0x220..0x228].copy_from_slice(&u64::to_be_bytes(0x40));
        signed_bytes[0x238..0x23A].copy_from_slice(&u16::to_be_bytes(1));
        signed_bytes[0x248..0x250].copy_from_slice(&u64::to_be_bytes(0x80));
        signed_bytes[0x260..0x268].copy_from_slice(&u64::to_be_bytes(0x20));
        let header = Header::try_from(&Blob::from(signed_bytes.clone())).unwrap();
        let keyset = Vault::from(()).try_keyset(&header).unwrap();
        let mut key_and_vector = [0; 0x40];
        let mut encryptor = Encryptor::<Aes256>::new(keyset.key().into(), keyset.vector().into());
        let (blocks, _) = key_and_vector.as_chunks_mut::<16>();
        for block in blocks {
            let block = block.into();
            encryptor.encrypt_block(block);
        }
        signed_bytes[0x120..0x160].copy_from_slice(&key_and_vector);
        let mut headers = vec![0; 0x70];
        headers[0x0C..0x10].copy_from_slice(&u32::to_be_bytes(1));
        headers[0x10..0x14].copy_from_slice(&u32::to_be_bytes(2));
        headers[0x20..0x28].copy_from_slice(&u64::to_be_bytes(0x300));
        headers[0x28..0x30].copy_from_slice(&u64::to_be_bytes(0x20));
        headers[0x40..0x44].copy_from_slice(&u32::to_be_bytes(3));
        let mut stream = Ctr128BE::<Aes128>::new((&[0; 16]).into(), (&[0; 16]).into());
        stream.apply_keystream(&mut headers);
        signed_bytes[0x160..0x1D0].copy_from_slice(&headers);
        signed_bytes[0x10..0x18].copy_from_slice(&u64::to_be_bytes(0x1D0));
        signed_bytes[0x300..0x320].copy_from_slice(&[0xAB; 0x20]);
        let signed_executable = SignedExecutable::from(Blob::from(signed_bytes));

        let executable = signed_executable.try_decrypt().unwrap();

        let bytes = executable.bytes();
        assert_eq!(bytes.byte(0x80), Some(Byte::default()));
    }

    #[test]
    fn a_section_that_begins_past_the_end_of_the_file_is_an_error() {
        let mut signed_bytes = vec![0; 0x400];
        signed_bytes[..4].copy_from_slice(b"SCE\0");
        signed_bytes[0x08..0x0A].copy_from_slice(&u16::to_be_bytes(1));
        signed_bytes[0x0C..0x10].copy_from_slice(&u32::to_be_bytes(0x100));
        signed_bytes[0x18..0x20].copy_from_slice(&u64::to_be_bytes(0x200));
        signed_bytes[0x30..0x38].copy_from_slice(&u64::to_be_bytes(0x200));
        signed_bytes[0x38..0x40].copy_from_slice(&u64::to_be_bytes(0x240));
        signed_bytes[0x7C..0x80].copy_from_slice(&u32::to_be_bytes(8));
        signed_bytes[0x220..0x228].copy_from_slice(&u64::to_be_bytes(0x40));
        signed_bytes[0x238..0x23A].copy_from_slice(&u16::to_be_bytes(1));
        let header = Header::try_from(&Blob::from(signed_bytes.clone())).unwrap();
        let keyset = Vault::from(()).try_keyset(&header).unwrap();
        let mut key_and_vector = [0; 0x40];
        let mut encryptor = Encryptor::<Aes256>::new(keyset.key().into(), keyset.vector().into());
        let (blocks, _) = key_and_vector.as_chunks_mut::<16>();
        for block in blocks {
            let block = block.into();
            encryptor.encrypt_block(block);
        }
        signed_bytes[0x120..0x160].copy_from_slice(&key_and_vector);
        let mut headers = vec![0; 0x70];
        headers[0x0C..0x10].copy_from_slice(&u32::to_be_bytes(1));
        headers[0x10..0x14].copy_from_slice(&u32::to_be_bytes(2));
        headers[0x20..0x28].copy_from_slice(&u64::to_be_bytes(0x3F0));
        headers[0x28..0x30].copy_from_slice(&u64::to_be_bytes(0x40));
        headers[0x30..0x34].copy_from_slice(&u32::to_be_bytes(2));
        headers[0x40..0x44].copy_from_slice(&u32::to_be_bytes(3));
        let mut stream = Ctr128BE::<Aes128>::new((&[0; 16]).into(), (&[0; 16]).into());
        stream.apply_keystream(&mut headers);
        signed_bytes[0x160..0x1D0].copy_from_slice(&headers);
        signed_bytes[0x10..0x18].copy_from_slice(&u64::to_be_bytes(0x1D0));
        let signed_executable = SignedExecutable::from(Blob::from(signed_bytes));

        let executable = signed_executable.try_decrypt();

        let bytes: usize = 0x400;
        assert_eq!(executable.unwrap_err(), Error::Length { bytes });
    }

    #[test]
    fn a_section_that_names_a_key_the_metadata_does_not_list_is_an_error() {
        let mut signed_bytes = vec![0; 0x400];
        signed_bytes[..4].copy_from_slice(b"SCE\0");
        signed_bytes[0x08..0x0A].copy_from_slice(&u16::to_be_bytes(1));
        signed_bytes[0x0C..0x10].copy_from_slice(&u32::to_be_bytes(0x100));
        signed_bytes[0x18..0x20].copy_from_slice(&u64::to_be_bytes(0x200));
        signed_bytes[0x30..0x38].copy_from_slice(&u64::to_be_bytes(0x200));
        signed_bytes[0x38..0x40].copy_from_slice(&u64::to_be_bytes(0x240));
        signed_bytes[0x7C..0x80].copy_from_slice(&u32::to_be_bytes(8));
        signed_bytes[0x220..0x228].copy_from_slice(&u64::to_be_bytes(0x40));
        signed_bytes[0x238..0x23A].copy_from_slice(&u16::to_be_bytes(1));
        let header = Header::try_from(&Blob::from(signed_bytes.clone())).unwrap();
        let keyset = Vault::from(()).try_keyset(&header).unwrap();
        let mut key_and_vector = [0; 0x40];
        let mut encryptor = Encryptor::<Aes256>::new(keyset.key().into(), keyset.vector().into());
        let (blocks, _) = key_and_vector.as_chunks_mut::<16>();
        for block in blocks {
            let block = block.into();
            encryptor.encrypt_block(block);
        }
        signed_bytes[0x120..0x160].copy_from_slice(&key_and_vector);
        let mut headers = vec![0; 0x70];
        headers[0x0C..0x10].copy_from_slice(&u32::to_be_bytes(1));
        headers[0x10..0x14].copy_from_slice(&u32::to_be_bytes(2));
        headers[0x20..0x28].copy_from_slice(&u64::to_be_bytes(0x300));
        headers[0x28..0x30].copy_from_slice(&u64::to_be_bytes(0x20));
        headers[0x30..0x34].copy_from_slice(&u32::to_be_bytes(2));
        headers[0x40..0x44].copy_from_slice(&u32::to_be_bytes(3));
        headers[0x44..0x48].copy_from_slice(&u32::to_be_bytes(5));
        let mut stream = Ctr128BE::<Aes128>::new((&[0; 16]).into(), (&[0; 16]).into());
        stream.apply_keystream(&mut headers);
        signed_bytes[0x160..0x1D0].copy_from_slice(&headers);
        signed_bytes[0x10..0x18].copy_from_slice(&u64::to_be_bytes(0x1D0));
        let signed_executable = SignedExecutable::from(Blob::from(signed_bytes));

        let executable = signed_executable.try_decrypt();

        let metadata = metadata::Error::Metadata;
        assert_eq!(executable.unwrap_err(), Error::Metadata { metadata });
    }

    #[test]
    fn a_section_that_names_a_program_the_file_does_not_list_is_an_error() {
        let mut signed_bytes = vec![0; 0x400];
        signed_bytes[..4].copy_from_slice(b"SCE\0");
        signed_bytes[0x08..0x0A].copy_from_slice(&u16::to_be_bytes(1));
        signed_bytes[0x0C..0x10].copy_from_slice(&u32::to_be_bytes(0x100));
        signed_bytes[0x18..0x20].copy_from_slice(&u64::to_be_bytes(0x200));
        signed_bytes[0x30..0x38].copy_from_slice(&u64::to_be_bytes(0x200));
        signed_bytes[0x38..0x40].copy_from_slice(&u64::to_be_bytes(0x240));
        signed_bytes[0x7C..0x80].copy_from_slice(&u32::to_be_bytes(8));
        signed_bytes[0x220..0x228].copy_from_slice(&u64::to_be_bytes(0x40));
        signed_bytes[0x238..0x23A].copy_from_slice(&u16::to_be_bytes(1));
        let header = Header::try_from(&Blob::from(signed_bytes.clone())).unwrap();
        let keyset = Vault::from(()).try_keyset(&header).unwrap();
        let mut key_and_vector = [0; 0x40];
        let mut encryptor = Encryptor::<Aes256>::new(keyset.key().into(), keyset.vector().into());
        let (blocks, _) = key_and_vector.as_chunks_mut::<16>();
        for block in blocks {
            let block = block.into();
            encryptor.encrypt_block(block);
        }
        signed_bytes[0x120..0x160].copy_from_slice(&key_and_vector);
        let mut headers = vec![0; 0x70];
        headers[0x0C..0x10].copy_from_slice(&u32::to_be_bytes(1));
        headers[0x10..0x14].copy_from_slice(&u32::to_be_bytes(2));
        headers[0x20..0x28].copy_from_slice(&u64::to_be_bytes(0x300));
        headers[0x28..0x30].copy_from_slice(&u64::to_be_bytes(0x20));
        headers[0x30..0x34].copy_from_slice(&u32::to_be_bytes(2));
        headers[0x34..0x38].copy_from_slice(&u32::to_be_bytes(5));
        headers[0x40..0x44].copy_from_slice(&u32::to_be_bytes(3));
        headers[0x48..0x4C].copy_from_slice(&u32::to_be_bytes(1));
        let mut stream = Ctr128BE::<Aes128>::new((&[0; 16]).into(), (&[0; 16]).into());
        stream.apply_keystream(&mut headers);
        signed_bytes[0x160..0x1D0].copy_from_slice(&headers);
        signed_bytes[0x10..0x18].copy_from_slice(&u64::to_be_bytes(0x1D0));
        let signed_executable = SignedExecutable::from(Blob::from(signed_bytes));

        let executable = signed_executable.try_decrypt();

        let metadata = metadata::Error::Metadata;
        assert_eq!(executable.unwrap_err(), Error::Metadata { metadata });
    }

    #[test]
    fn a_signed_executable_whose_control_information_is_broken_is_an_error() {
        let mut signed_bytes = vec![0; 0x400];
        signed_bytes[..4].copy_from_slice(b"SCE\0");
        signed_bytes[0x08..0x0A].copy_from_slice(&u16::to_be_bytes(1));
        signed_bytes[0x58..0x60].copy_from_slice(&u64::to_be_bytes(0x80));
        signed_bytes[0x60..0x68].copy_from_slice(&u64::to_be_bytes(0x40));
        signed_bytes[0x80..0x84].copy_from_slice(&u32::to_be_bytes(3));
        signed_bytes[0x84..0x88].copy_from_slice(&u32::to_be_bytes(8));
        signed_bytes[0x7C..0x80].copy_from_slice(&u32::to_be_bytes(8));
        let signed_executable = SignedExecutable::from(Blob::from(signed_bytes));

        let executable = signed_executable.try_decrypt();

        let bytes: usize = 0x400;
        let license = license::Error::Length { bytes };
        assert_eq!(executable.unwrap_err(), Error::License { license });
    }

    #[test]
    fn a_signed_executable_of_a_key_revision_the_vault_has_no_keyset_for_is_an_error() {
        let mut signed_bytes = vec![0; 0x400];
        signed_bytes[..4].copy_from_slice(b"SCE\0");
        signed_bytes[0x08..0x0A].copy_from_slice(&u16::to_be_bytes(7));
        signed_bytes[0x7C..0x80].copy_from_slice(&u32::to_be_bytes(8));
        let signed_executable = SignedExecutable::from(Blob::from(signed_bytes));

        let executable = signed_executable.try_decrypt();

        let revision = bytes::Integer::from([0, 7]);
        let program = bytes::Integer::from([0, 0, 0, 8]);
        let vault = vault::Error::Keyset { revision, program };
        assert_eq!(executable.unwrap_err(), Error::Vault { vault });
    }

    #[test]
    fn a_signed_executable_whose_metadata_the_keyset_does_not_open_is_an_error() {
        let mut signed_bytes = vec![0; 0x400];
        signed_bytes[..4].copy_from_slice(b"SCE\0");
        signed_bytes[0x08..0x0A].copy_from_slice(&u16::to_be_bytes(1));
        signed_bytes[0x0C..0x10].copy_from_slice(&u32::to_be_bytes(0x100));
        signed_bytes[0x7C..0x80].copy_from_slice(&u32::to_be_bytes(8));
        let signed_executable = SignedExecutable::from(Blob::from(signed_bytes));

        let executable = signed_executable.try_decrypt();

        let metadata = metadata::Error::Metadata;
        assert_eq!(executable.unwrap_err(), Error::Metadata { metadata });
    }

    #[test]
    fn a_signed_executable_whose_elf_begins_past_its_end_is_an_error() {
        let mut signed_bytes = vec![0; 0x400];
        signed_bytes[..4].copy_from_slice(b"SCE\0");
        signed_bytes[0x08..0x0A].copy_from_slice(&u16::to_be_bytes(1));
        signed_bytes[0x0C..0x10].copy_from_slice(&u32::to_be_bytes(0x100));
        signed_bytes[0x18..0x20].copy_from_slice(&u64::to_be_bytes(0x200));
        signed_bytes[0x30..0x38].copy_from_slice(&u64::to_be_bytes(0x3F0));
        signed_bytes[0x7C..0x80].copy_from_slice(&u32::to_be_bytes(8));
        let header = Header::try_from(&Blob::from(signed_bytes.clone())).unwrap();
        let keyset = Vault::from(()).try_keyset(&header).unwrap();
        let mut key_and_vector = [0; 0x40];
        let mut encryptor = Encryptor::<Aes256>::new(keyset.key().into(), keyset.vector().into());
        let (blocks, _) = key_and_vector.as_chunks_mut::<16>();
        for block in blocks {
            let block = block.into();
            encryptor.encrypt_block(block);
        }
        signed_bytes[0x120..0x160].copy_from_slice(&key_and_vector);
        let mut headers = vec![0; 0x20];
        let mut stream = Ctr128BE::<Aes128>::new((&[0; 16]).into(), (&[0; 16]).into());
        stream.apply_keystream(&mut headers);
        signed_bytes[0x160..0x180].copy_from_slice(&headers);
        signed_bytes[0x10..0x18].copy_from_slice(&u64::to_be_bytes(0x180));
        let signed_executable = SignedExecutable::from(Blob::from(signed_bytes));

        let executable = signed_executable.try_decrypt();

        let bytes: usize = 0x400;
        assert_eq!(executable.unwrap_err(), Error::Length { bytes });
    }

    #[test]
    fn a_signed_executable_whose_programs_begin_past_its_end_is_an_error() {
        let mut signed_bytes = vec![0; 0x400];
        signed_bytes[..4].copy_from_slice(b"SCE\0");
        signed_bytes[0x08..0x0A].copy_from_slice(&u16::to_be_bytes(1));
        signed_bytes[0x0C..0x10].copy_from_slice(&u32::to_be_bytes(0x100));
        signed_bytes[0x18..0x20].copy_from_slice(&u64::to_be_bytes(0x200));
        signed_bytes[0x30..0x38].copy_from_slice(&u64::to_be_bytes(0x200));
        signed_bytes[0x38..0x40].copy_from_slice(&u64::to_be_bytes(0x3F0));
        signed_bytes[0x7C..0x80].copy_from_slice(&u32::to_be_bytes(8));
        signed_bytes[0x238..0x23A].copy_from_slice(&u16::to_be_bytes(1));
        let header = Header::try_from(&Blob::from(signed_bytes.clone())).unwrap();
        let keyset = Vault::from(()).try_keyset(&header).unwrap();
        let mut key_and_vector = [0; 0x40];
        let mut encryptor = Encryptor::<Aes256>::new(keyset.key().into(), keyset.vector().into());
        let (blocks, _) = key_and_vector.as_chunks_mut::<16>();
        for block in blocks {
            let block = block.into();
            encryptor.encrypt_block(block);
        }
        signed_bytes[0x120..0x160].copy_from_slice(&key_and_vector);
        let mut headers = vec![0; 0x20];
        let mut stream = Ctr128BE::<Aes128>::new((&[0; 16]).into(), (&[0; 16]).into());
        stream.apply_keystream(&mut headers);
        signed_bytes[0x160..0x180].copy_from_slice(&headers);
        signed_bytes[0x10..0x18].copy_from_slice(&u64::to_be_bytes(0x180));
        let signed_executable = SignedExecutable::from(Blob::from(signed_bytes));

        let executable = signed_executable.try_decrypt();

        let bytes: usize = 0x400;
        assert_eq!(executable.unwrap_err(), Error::Length { bytes });
    }

    #[test]
    fn a_signed_executable_too_short_for_the_programs_it_names_is_an_error() {
        let mut signed_bytes = vec![0; 0x400];
        signed_bytes[..4].copy_from_slice(b"SCE\0");
        signed_bytes[0x08..0x0A].copy_from_slice(&u16::to_be_bytes(1));
        signed_bytes[0x0C..0x10].copy_from_slice(&u32::to_be_bytes(0x100));
        signed_bytes[0x18..0x20].copy_from_slice(&u64::to_be_bytes(0x40));
        signed_bytes[0x30..0x38].copy_from_slice(&u64::to_be_bytes(0x200));
        signed_bytes[0x38..0x40].copy_from_slice(&u64::to_be_bytes(0x240));
        signed_bytes[0x7C..0x80].copy_from_slice(&u32::to_be_bytes(8));
        signed_bytes[0x220..0x228].copy_from_slice(&u64::to_be_bytes(0x40));
        signed_bytes[0x238..0x23A].copy_from_slice(&u16::to_be_bytes(1));
        let header = Header::try_from(&Blob::from(signed_bytes.clone())).unwrap();
        let keyset = Vault::from(()).try_keyset(&header).unwrap();
        let mut key_and_vector = [0; 0x40];
        let mut encryptor = Encryptor::<Aes256>::new(keyset.key().into(), keyset.vector().into());
        let (blocks, _) = key_and_vector.as_chunks_mut::<16>();
        for block in blocks {
            let block = block.into();
            encryptor.encrypt_block(block);
        }
        signed_bytes[0x120..0x160].copy_from_slice(&key_and_vector);
        let mut headers = vec![0; 0x20];
        let mut stream = Ctr128BE::<Aes128>::new((&[0; 16]).into(), (&[0; 16]).into());
        stream.apply_keystream(&mut headers);
        signed_bytes[0x160..0x180].copy_from_slice(&headers);
        signed_bytes[0x10..0x18].copy_from_slice(&u64::to_be_bytes(0x180));
        let signed_executable = SignedExecutable::from(Blob::from(signed_bytes));

        let executable = signed_executable.try_decrypt();

        let bytes: usize = 0x400;
        assert_eq!(executable.unwrap_err(), Error::Length { bytes });
    }

    #[test]
    fn a_file_that_is_no_signed_executable_is_an_error() {
        let signed_bytes = SignedExecutable::from(Blob::from(b"ELF\0nothing here".as_slice()));

        let executable = signed_bytes.try_decrypt();

        assert!(executable.is_err());
    }

    #[test]
    fn a_file_of_no_bytes_is_an_error() {
        let signed_bytes = SignedExecutable::from(Blob::default());

        let executable = signed_bytes.try_decrypt();

        let text = String::new();
        let header = header::Error::SignedExecutable { text };
        assert_eq!(executable.unwrap_err(), Error::Header { header });
    }

    #[test]
    fn a_signed_executable_goes_out_and_comes_back_as_the_bytes_it_keeps() {
        let signed_executable = SignedExecutable::from(Blob::from(b"SCE\0".as_slice()));

        let tokens = [
            Token::Seq { len: Some(4) },
            Token::U8(b'S'),
            Token::U8(b'C'),
            Token::U8(b'E'),
            Token::U8(0),
            Token::SeqEnd,
        ];

        assert_tokens(&signed_executable, &tokens);
    }

    #[test]
    fn a_signed_executable_of_no_bytes_goes_out_and_comes_back_as_no_bytes() {
        let signed_executable = SignedExecutable::default();

        let tokens = [Token::Seq { len: Some(0) }, Token::SeqEnd];

        assert_tokens(&signed_executable, &tokens);
    }
}
