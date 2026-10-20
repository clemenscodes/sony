pub(super) mod error;

pub(super) use error::Error;

use crate::core::content_identifier::ContentIdentifier;
use bytes::Bytes;
use bytes::Integer;
use ddd::{DomainLayer, Layered, ValueObject};

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Copy)]
pub(super) struct Header {
    content_identifier: ContentIdentifier,
    block_length: usize,
    length: usize,
}

impl Layered for Header {
    type Layer = DomainLayer;
}

impl ValueObject for Header {}

impl Header {
    pub(super) const LENGTH: usize = 0x100;

    const MAGIC: [u8; 4] = *b"NPD\0";
    const VERSION: [u8; 4] = [0x00, 0x00, 0x00, 0x02];
    const LICENSE: [u8; 4] = [0x00, 0x00, 0x00, 0x02];
    const FLAGS: [u8; 4] = [0x00, 0x00, 0x00, 0x0C];

    pub(super) fn content_identifier(&self) -> &ContentIdentifier {
        &self.content_identifier
    }

    pub(super) fn block_length(&self) -> usize {
        self.block_length
    }

    pub(super) fn length(&self) -> usize {
        self.length
    }
}

impl TryFrom<&[u8; Header::LENGTH]> for Header {
    type Error = Error;

    fn try_from(header_bytes: &[u8; Header::LENGTH]) -> Result<Self, Self::Error> {
        let mut field_bytes = [0; 4];
        field_bytes.copy_from_slice(&header_bytes[0x00..0x04]);
        if field_bytes != Self::MAGIC {
            let magic = Bytes::from(field_bytes);
            return Err(Error::Magic { magic });
        }
        field_bytes.copy_from_slice(&header_bytes[0x04..0x08]);
        if field_bytes != Self::VERSION {
            let version = Integer::from(field_bytes);
            return Err(Error::Version { version });
        }
        field_bytes.copy_from_slice(&header_bytes[0x08..0x0C]);
        if field_bytes != Self::LICENSE {
            let license = Integer::from(field_bytes);
            return Err(Error::License { license });
        }
        field_bytes.copy_from_slice(&header_bytes[0x80..0x84]);
        if field_bytes != Self::FLAGS {
            let flags = Integer::from(field_bytes);
            return Err(Error::Flags { flags });
        }
        let mut content_identifier = [0; ContentIdentifier::LENGTH];
        content_identifier.copy_from_slice(&header_bytes[0x10..0x40]);
        let content_identifier = ContentIdentifier::try_from(content_identifier)?;
        field_bytes.copy_from_slice(&header_bytes[0x84..0x88]);
        let block_length = Integer::from(field_bytes);
        let block_length = usize::from(block_length);
        let mut word_bytes = [0; 8];
        word_bytes.copy_from_slice(&header_bytes[0x88..0x90]);
        let length = Integer::from(word_bytes);
        let length = usize::from(length);
        Ok(Self {
            content_identifier,
            block_length,
            length,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_header_of_a_protected_file_of_the_three_packs_reads_field_for_field() {
        let mut edata = [0; Header::LENGTH];
        edata[0x00..0x04].copy_from_slice(b"NPD\0");
        edata[0x04..0x08].copy_from_slice(&u32::to_be_bytes(2));
        edata[0x08..0x0C].copy_from_slice(&u32::to_be_bytes(2));
        edata[0x10..0x34].copy_from_slice(b"EP0000-XBRK00000_00-XBREAKTESTLABEL0");
        edata[0x80..0x84].copy_from_slice(&u32::to_be_bytes(0x0C));
        edata[0x84..0x88].copy_from_slice(&u32::to_be_bytes(16384));

        let header = Header::try_from(&edata).unwrap();

        assert_eq!(
            header.content_identifier().to_string(),
            "EP0000-XBRK00000_00-XBREAKTESTLABEL0"
        );
        assert_eq!(header.block_length(), 16384);
        assert_eq!(header.length(), 0);
    }

    #[test]
    fn a_content_identifier_of_bytes_that_are_no_letters_is_an_error() {
        let mut edata = [0; Header::LENGTH];
        edata[0x00..0x04].copy_from_slice(b"NPD\0");
        edata[0x04..0x08].copy_from_slice(&u32::to_be_bytes(2));
        edata[0x08..0x0C].copy_from_slice(&u32::to_be_bytes(2));
        edata[0x10] = 0xFF;
        edata[0x80..0x84].copy_from_slice(&u32::to_be_bytes(0x0C));

        let header = Header::try_from(&edata);

        assert!(header.is_err());
    }

    #[test]
    fn a_file_that_does_not_begin_with_the_magic_of_a_protected_file_is_none() {
        let mut edata = [0; Header::LENGTH];
        edata[0x00..0x04].copy_from_slice(b"PSAR");

        let header = Header::try_from(&edata);

        let magic = bytes::Bytes::from([0x50, 0x53, 0x41, 0x52]);
        assert_eq!(header.unwrap_err(), Error::Magic { magic });
    }

    #[test]
    fn a_version_this_repository_has_never_read_is_an_error() {
        let mut edata = [0; Header::LENGTH];
        edata[0x00..0x04].copy_from_slice(b"NPD\0");
        edata[0x04..0x08].copy_from_slice(&u32::to_be_bytes(4));

        let header = Header::try_from(&edata);

        let version = Integer::from([0x00, 0x00, 0x00, 0x04]);
        assert_eq!(header.unwrap_err(), Error::Version { version });
    }

    #[test]
    fn a_protected_file_no_license_file_opens_is_an_error() {
        let mut edata = [0; Header::LENGTH];
        edata[0x00..0x04].copy_from_slice(b"NPD\0");
        edata[0x04..0x08].copy_from_slice(&u32::to_be_bytes(2));
        edata[0x08..0x0C].copy_from_slice(&u32::to_be_bytes(3));

        let header = Header::try_from(&edata);

        let license = Integer::from([0x00, 0x00, 0x00, 0x03]);
        assert_eq!(header.unwrap_err(), Error::License { license });
    }

    #[test]
    fn flags_this_repository_has_never_read_are_errors() {
        let mut edata = [0; Header::LENGTH];
        edata[0x00..0x04].copy_from_slice(b"NPD\0");
        edata[0x04..0x08].copy_from_slice(&u32::to_be_bytes(2));
        edata[0x08..0x0C].copy_from_slice(&u32::to_be_bytes(2));
        edata[0x80..0x84].copy_from_slice(&u32::to_be_bytes(1));

        let header = Header::try_from(&edata);

        let flags = Integer::from([0x00, 0x00, 0x00, 0x01]);
        assert_eq!(header.unwrap_err(), Error::Flags { flags });
    }
}
