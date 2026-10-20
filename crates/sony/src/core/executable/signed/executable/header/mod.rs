pub(super) mod error;

use bytes::Integer;
use bytes::blob::Blob;
use ddd::{DomainLayer, Layered, ValueObject};
use serde::{Deserialize, Serialize};

pub(super) use error::Error;

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Copy, Serialize, Deserialize)]
pub(super) struct Header {
    revision: Integer<{ Header::REVISION_LENGTH }>,
    metadata_offset: usize,
    length: usize,
    program: Integer<4>,
    elf_offset: usize,
    programs_offset: usize,
    sections_offset: usize,
    control_offset: usize,
    control_length: usize,
    executable_length: usize,
}

impl Layered for Header {
    type Layer = DomainLayer;
}

impl ValueObject for Header {}

impl Header {
    pub(super) const LENGTH: usize = 0x20;

    const MAGIC: &'static [u8] = b"SCE\0";
    const CEILING: usize = 0x80;
    const EXTENDED_HEADER: usize = 0x70;
    const FIELD_LENGTH: usize = 8;
    const REVISION_LENGTH: usize = 2;

    pub(super) fn revision(&self) -> Integer<{ Self::REVISION_LENGTH }> {
        self.revision
    }

    pub(super) fn metadata_offset(&self) -> usize {
        self.metadata_offset
    }

    pub(super) fn length(&self) -> usize {
        self.length
    }

    pub(super) fn program(&self) -> Integer<4> {
        self.program
    }

    pub(super) fn elf_offset(&self) -> usize {
        self.elf_offset
    }

    pub(super) fn programs_offset(&self) -> usize {
        self.programs_offset
    }

    pub(super) fn sections_offset(&self) -> usize {
        self.sections_offset
    }

    pub(super) fn control_offset(&self) -> usize {
        self.control_offset
    }

    pub(super) fn control_length(&self) -> usize {
        self.control_length
    }

    pub(super) fn executable_length(&self) -> usize {
        self.executable_length
    }
}

impl Default for Header {
    fn default() -> Self {
        let revision = Integer::from([0; Self::REVISION_LENGTH]);
        let program = Integer::default();
        let metadata_offset = 0;
        let length = 0;
        let elf_offset = 0;
        let programs_offset = 0;
        let sections_offset = 0;
        let control_offset = 0;
        let control_length = 0;
        let executable_length = 0;
        Self {
            revision,
            metadata_offset,
            length,
            program,
            elf_offset,
            programs_offset,
            sections_offset,
            control_offset,
            control_length,
            executable_length,
        }
    }
}

impl TryFrom<&Blob> for Header {
    type Error = Error;

    fn try_from(signed_bytes: &Blob) -> Result<Self, Self::Error> {
        let magic = signed_bytes.chunk(0, Self::MAGIC.len()).unwrap_or_default();
        if magic != Blob::from(Self::MAGIC) {
            let magic = Vec::<u8>::from(&magic);
            let text = String::from_utf8_lossy(&magic).to_string();
            return Err(Error::SignedExecutable { text });
        }
        let header_bytes = signed_bytes.try_bytes::<{ Header::CEILING }>(0);
        let Ok(header_bytes) = header_bytes else {
            let bytes = signed_bytes.length();
            return Err(Error::Length { bytes });
        };
        let header_bytes = <[u8; Header::CEILING]>::from(header_bytes);
        let mut revision_bytes = [0; Header::REVISION_LENGTH];
        revision_bytes.copy_from_slice(&header_bytes[0x08..0x0A]);
        let revision = Integer::from(revision_bytes);
        let mut field_bytes = [0; 4];
        field_bytes.copy_from_slice(&header_bytes[0x0C..0x10]);
        let metadata_offset = usize::from(Integer::from(field_bytes));
        let mut word_bytes = [0; Header::FIELD_LENGTH];
        word_bytes.copy_from_slice(&header_bytes[0x10..0x18]);
        let length = usize::from(Integer::from(word_bytes));
        word_bytes.copy_from_slice(&header_bytes[0x18..0x20]);
        let executable_length = usize::from(Integer::from(word_bytes));
        word_bytes.copy_from_slice(&header_bytes[0x30..0x38]);
        let elf_offset = usize::from(Integer::from(word_bytes));
        word_bytes.copy_from_slice(&header_bytes[0x38..0x40]);
        let programs_offset = usize::from(Integer::from(word_bytes));
        word_bytes.copy_from_slice(&header_bytes[0x40..0x48]);
        let sections_offset = usize::from(Integer::from(word_bytes));
        word_bytes.copy_from_slice(&header_bytes[0x58..0x60]);
        let control_offset = usize::from(Integer::from(word_bytes));
        word_bytes.copy_from_slice(&header_bytes[0x60..0x68]);
        let control_length = usize::from(Integer::from(word_bytes));
        let extended = Self::EXTENDED_HEADER + 0x0C;
        field_bytes.copy_from_slice(&header_bytes[extended..extended + 4]);
        let program = Integer::from(field_bytes);
        Ok(Self {
            revision,
            metadata_offset,
            length,
            program,
            elf_offset,
            programs_offset,
            sections_offset,
            control_offset,
            control_length,
            executable_length,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_test::{Token, assert_tokens};

    #[test]
    fn a_file_that_does_not_begin_with_the_magic_is_no_signed_executable() {
        let signed_bytes = Blob::from(b"ELF\0and more".as_slice());

        let header = Header::try_from(&signed_bytes);

        let text = String::from("ELF\0");
        assert_eq!(header.unwrap_err(), Error::SignedExecutable { text });
    }

    #[test]
    fn a_file_shorter_than_a_header_is_an_error() {
        let signed_bytes = Blob::from(b"SCE\0".as_slice());

        let header = Header::try_from(&signed_bytes);

        let bytes: usize = 4;
        assert_eq!(header.unwrap_err(), Error::Length { bytes });
    }

    #[test]
    fn a_header_reads_the_key_revision_and_the_program_type() {
        let mut signed_bytes = vec![0; 0x100];
        signed_bytes[..4].copy_from_slice(b"SCE\0");
        signed_bytes[0x08..0x0A].copy_from_slice(&u16::to_be_bytes(1));
        signed_bytes[0x7C..0x80].copy_from_slice(&u32::to_be_bytes(8));

        let header = Header::try_from(&Blob::from(signed_bytes)).unwrap();

        assert_eq!(header.revision(), Integer::from([0, 1]));
        assert_eq!(header.program(), Integer::from([0, 0, 0, 8]));
    }

    #[test]
    fn a_header_reads_where_the_metadata_and_the_elf_begin() {
        let mut signed_bytes = vec![0; 0x100];
        signed_bytes[..4].copy_from_slice(b"SCE\0");
        signed_bytes[0x0C..0x10].copy_from_slice(&u32::to_be_bytes(0x4A0));
        signed_bytes[0x30..0x38].copy_from_slice(&u64::to_be_bytes(0x90));

        let header = Header::try_from(&Blob::from(signed_bytes)).unwrap();

        assert_eq!(header.metadata_offset(), 0x4A0);
        assert_eq!(header.elf_offset(), 0x90);
    }

    #[test]
    fn a_header_goes_out_and_comes_back_field_for_field() {
        let mut signed_bytes = vec![0; 0x100];
        signed_bytes[..4].copy_from_slice(b"SCE\0");
        signed_bytes[0x08..0x0A].copy_from_slice(&u16::to_be_bytes(1));
        signed_bytes[0x0C..0x10].copy_from_slice(&u32::to_be_bytes(0x4A0));
        signed_bytes[0x30..0x38].copy_from_slice(&u64::to_be_bytes(0x90));
        signed_bytes[0x7C..0x80].copy_from_slice(&u32::to_be_bytes(8));
        let header = Header::try_from(&Blob::from(signed_bytes)).unwrap();

        let tokens = [
            Token::Struct {
                name: "Header",
                len: 10,
            },
            Token::Str("revision"),
            Token::Str("0001"),
            Token::Str("metadata_offset"),
            Token::U64(0x4A0),
            Token::Str("length"),
            Token::U64(0),
            Token::Str("program"),
            Token::Str("00000008"),
            Token::Str("elf_offset"),
            Token::U64(0x90),
            Token::Str("programs_offset"),
            Token::U64(0),
            Token::Str("sections_offset"),
            Token::U64(0),
            Token::Str("control_offset"),
            Token::U64(0),
            Token::Str("control_length"),
            Token::U64(0),
            Token::Str("executable_length"),
            Token::U64(0),
            Token::StructEnd,
        ];

        assert_tokens(&header, &tokens);
    }
}
