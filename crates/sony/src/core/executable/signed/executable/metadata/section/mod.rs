use bytes::Integer;
use ddd::{DomainLayer, Layered, ValueObject};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Copy, Serialize, Deserialize)]
pub(in crate::core::executable::signed::executable) struct Section {
    offset: usize,
    length: usize,
    kind: Integer<4>,
    program_index: usize,
    key: Integer<4>,
    key_index: usize,
    vector_index: usize,
    packing: Integer<4>,
}

impl Layered for Section {
    type Layer = DomainLayer;
}

impl ValueObject for Section {}

impl Section {
    pub(in crate::core::executable::signed::executable) const LENGTH: usize = 0x30;

    const FIELD_LENGTH: usize = 8;
    const PROGRAM: u32 = 2;
    const KEY: u32 = 3;
    const PACKING: u32 = 2;

    pub(in crate::core::executable::signed::executable) fn offset(&self) -> usize {
        self.offset
    }

    pub(in crate::core::executable::signed::executable) fn length(&self) -> usize {
        self.length
    }

    pub(in crate::core::executable::signed::executable) fn program_index(&self) -> usize {
        self.program_index
    }

    pub(in crate::core::executable::signed::executable) fn key_index(&self) -> usize {
        self.key_index
    }

    pub(in crate::core::executable::signed::executable) fn vector_index(&self) -> usize {
        self.vector_index
    }

    pub(in crate::core::executable::signed::executable) fn is_program(&self) -> bool {
        u32::from(self.kind) == Self::PROGRAM
    }

    pub(in crate::core::executable::signed::executable) fn has_key(&self) -> bool {
        u32::from(self.key) == Self::KEY
    }

    pub(in crate::core::executable::signed::executable) fn has_packing(&self) -> bool {
        u32::from(self.packing) == Self::PACKING
    }
}

impl From<&[u8; Section::LENGTH]> for Section {
    fn from(bytes: &[u8; Section::LENGTH]) -> Self {
        let mut field_bytes = [0; Section::FIELD_LENGTH];
        field_bytes.copy_from_slice(&bytes[0x00..0x08]);
        let offset = usize::from(Integer::from(field_bytes));
        field_bytes.copy_from_slice(&bytes[0x08..0x10]);
        let length = usize::from(Integer::from(field_bytes));
        let mut field_bytes = [0; 4];
        field_bytes.copy_from_slice(&bytes[0x10..0x14]);
        let kind = Integer::from(field_bytes);
        field_bytes.copy_from_slice(&bytes[0x14..0x18]);
        let program_index = usize::from(Integer::from(field_bytes));
        field_bytes.copy_from_slice(&bytes[0x20..0x24]);
        let key = Integer::from(field_bytes);
        field_bytes.copy_from_slice(&bytes[0x24..0x28]);
        let key_index = usize::from(Integer::from(field_bytes));
        field_bytes.copy_from_slice(&bytes[0x28..0x2C]);
        let vector_index = usize::from(Integer::from(field_bytes));
        field_bytes.copy_from_slice(&bytes[0x2C..0x30]);
        let packing = Integer::from(field_bytes);
        Self {
            offset,
            length,
            kind,
            program_index,
            key,
            key_index,
            vector_index,
            packing,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_test::{Token, assert_tokens};

    #[test]
    fn a_section_reads_where_its_data_is_and_how_much_of_it_there_is() {
        let mut bytes = [0; Section::LENGTH];
        bytes[0x00..0x08].copy_from_slice(&u64::to_be_bytes(0xA80));
        bytes[0x08..0x10].copy_from_slice(&u64::to_be_bytes(0x1000));

        let section = Section::from(&bytes);

        assert_eq!(section.offset(), 0xA80);
        assert_eq!(section.length(), 0x1000);
    }

    #[test]
    fn a_section_of_type_two_is_program() {
        let mut bytes = [0; Section::LENGTH];
        bytes[0x10..0x14].copy_from_slice(&u32::to_be_bytes(2));

        let section = Section::from(&bytes);

        assert!(section.is_program());
    }

    #[test]
    fn a_section_with_a_key_names_the_key_and_the_vector_it_needs() {
        let mut bytes = [0; Section::LENGTH];
        bytes[0x20..0x24].copy_from_slice(&u32::to_be_bytes(3));
        bytes[0x24..0x28].copy_from_slice(&u32::to_be_bytes(4));
        bytes[0x28..0x2C].copy_from_slice(&u32::to_be_bytes(5));

        let section = Section::from(&bytes);

        assert!(section.has_key());
        assert_eq!(section.key_index(), 4);
        assert_eq!(section.vector_index(), 5);
    }

    #[test]
    fn a_section_without_packing_states_so() {
        let bytes = [0; Section::LENGTH];

        let section = Section::from(&bytes);

        assert!(!section.has_packing());
    }

    #[test]
    fn a_section_goes_out_and_comes_back_field_for_field() {
        let mut bytes = [0; Section::LENGTH];
        bytes[0x00..0x08].copy_from_slice(&u64::to_be_bytes(0xA80));
        bytes[0x08..0x10].copy_from_slice(&u64::to_be_bytes(0x1000));
        bytes[0x10..0x14].copy_from_slice(&u32::to_be_bytes(2));
        bytes[0x14..0x18].copy_from_slice(&u32::to_be_bytes(1));
        bytes[0x20..0x24].copy_from_slice(&u32::to_be_bytes(3));
        bytes[0x24..0x28].copy_from_slice(&u32::to_be_bytes(4));
        bytes[0x28..0x2C].copy_from_slice(&u32::to_be_bytes(5));
        bytes[0x2C..0x30].copy_from_slice(&u32::to_be_bytes(2));
        let section = Section::from(&bytes);

        let tokens = [
            Token::Struct {
                name: "Section",
                len: 8,
            },
            Token::Str("offset"),
            Token::U64(0xA80),
            Token::Str("length"),
            Token::U64(0x1000),
            Token::Str("kind"),
            Token::Str("00000002"),
            Token::Str("program_index"),
            Token::U64(1),
            Token::Str("key"),
            Token::Str("00000003"),
            Token::Str("key_index"),
            Token::U64(4),
            Token::Str("vector_index"),
            Token::U64(5),
            Token::Str("packing"),
            Token::Str("00000002"),
            Token::StructEnd,
        ];

        assert_tokens(&section, &tokens);
    }
}
