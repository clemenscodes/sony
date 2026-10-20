pub(super) mod error;

mod platform;
mod release;

pub(super) use error::Error;

use crate::core::content_identifier::ContentIdentifier;
use crate::core::package::header::platform::Platform;
use crate::core::package::header::release::Release;
use bytes::Bytes;
use bytes::Integer;
use bytes::blob::Blob;
use ddd::{DomainLayer, Layered, ValueObject};

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Copy)]
pub(super) struct Header {
    release: Release,
    platform: Platform,
    metadata_offset: usize,
    metadata_count: usize,
    metadata_length: usize,
    file_count: usize,
    length: usize,
    contents_offset: usize,
    contents_length: usize,
    content_identifier: ContentIdentifier,
    digest: Bytes<{ Header::DIGEST }>,
    vector: Bytes<{ Header::VECTOR }>,
}

impl Layered for Header {
    type Layer = DomainLayer;
}

impl ValueObject for Header {}

impl Header {
    pub(super) const LENGTH: usize = 0x80;
    pub(super) const DIGEST: usize = 16;
    pub(super) const VECTOR: usize = 16;

    const MAGIC: [u8; 4] = [0x7F, 0x50, 0x4B, 0x47];

    pub(super) fn content_identifier(&self) -> &ContentIdentifier {
        &self.content_identifier
    }

    pub(super) fn metadata_offset(&self) -> usize {
        self.metadata_offset
    }

    pub(super) fn metadata_count(&self) -> usize {
        self.metadata_count
    }

    pub(super) fn metadata_length(&self) -> usize {
        self.metadata_length
    }

    pub(super) fn file_count(&self) -> usize {
        self.file_count
    }

    pub(super) fn contents_offset(&self) -> usize {
        self.contents_offset
    }

    pub(super) fn contents_length(&self) -> usize {
        self.contents_length
    }

    pub(super) fn vector(&self) -> Bytes<{ Self::VECTOR }> {
        self.vector
    }
}

impl From<(usize, &Blob)> for Header {
    fn from((files, contents): (usize, &Blob)) -> Self {
        let release = Release::default();
        let platform = Platform::default();
        let metadata_offset = Self::LENGTH;
        let metadata_count = 0;
        let metadata_length = 0;
        let file_count = files;
        let contents_offset = Self::LENGTH;
        let contents_length = contents.length();
        let length = usize::saturating_add(Self::LENGTH, contents_length);
        let content_identifier = ContentIdentifier::default();
        let digest = Bytes::default();
        let vector = Bytes::default();
        Self {
            release,
            platform,
            metadata_offset,
            metadata_count,
            metadata_length,
            file_count,
            length,
            contents_offset,
            contents_length,
            content_identifier,
            digest,
            vector,
        }
    }
}

impl From<&Header> for [u8; Header::LENGTH] {
    fn from(header: &Header) -> Self {
        let mut bytes = [0; Header::LENGTH];
        bytes[0x00..0x04].copy_from_slice(&Header::MAGIC);
        bytes[0x04..0x06].copy_from_slice(header.release.bytes());
        bytes[0x06..0x08].copy_from_slice(header.platform.bytes());
        let metadata_offset = header.metadata_offset as u32;
        let metadata_count = header.metadata_count as u32;
        let metadata_length = header.metadata_length as u32;
        let file_count = header.file_count as u32;
        bytes[0x08..0x0C].copy_from_slice(&metadata_offset.to_be_bytes());
        bytes[0x0C..0x10].copy_from_slice(&metadata_count.to_be_bytes());
        bytes[0x10..0x14].copy_from_slice(&metadata_length.to_be_bytes());
        bytes[0x14..0x18].copy_from_slice(&file_count.to_be_bytes());
        let length = header.length as u64;
        let contents_offset = header.contents_offset as u64;
        let contents_length = header.contents_length as u64;
        bytes[0x18..0x20].copy_from_slice(&length.to_be_bytes());
        bytes[0x20..0x28].copy_from_slice(&contents_offset.to_be_bytes());
        bytes[0x28..0x30].copy_from_slice(&contents_length.to_be_bytes());
        bytes[0x30..0x60].copy_from_slice(header.content_identifier.bytes());
        let digest = <[u8; Header::DIGEST]>::from(header.digest);
        let vector = <[u8; Header::VECTOR]>::from(header.vector);
        bytes[0x60..0x70].copy_from_slice(&digest);
        bytes[0x70..0x80].copy_from_slice(&vector);
        bytes
    }
}

impl TryFrom<&Blob> for Header {
    type Error = Error;

    fn try_from(package: &Blob) -> Result<Self, Self::Error> {
        let header_bytes = package.try_bytes::<{ Header::LENGTH }>(0);
        let Ok(header_bytes) = header_bytes else {
            let bytes = package.length();
            return Err(Error::Length { bytes });
        };
        let header_bytes = <[u8; Header::LENGTH]>::from(header_bytes);
        let mut magic_bytes = [0; 4];
        magic_bytes.copy_from_slice(&header_bytes[0x00..0x04]);
        if magic_bytes != Self::MAGIC {
            let magic = Bytes::from(magic_bytes);
            return Err(Error::Magic { magic });
        }
        let mut release_bytes = [0; Release::LENGTH];
        release_bytes.copy_from_slice(&header_bytes[0x04..0x06]);
        let release = Release::try_from(release_bytes)?;
        let mut platform_bytes = [0; Platform::LENGTH];
        platform_bytes.copy_from_slice(&header_bytes[0x06..0x08]);
        let platform = Platform::try_from(platform_bytes)?;
        let mut field_bytes = [0; 4];
        field_bytes.copy_from_slice(&header_bytes[0x08..0x0C]);
        let metadata_offset = Integer::from(field_bytes);
        field_bytes.copy_from_slice(&header_bytes[0x0C..0x10]);
        let metadata_count = Integer::from(field_bytes);
        field_bytes.copy_from_slice(&header_bytes[0x10..0x14]);
        let metadata_length = Integer::from(field_bytes);
        field_bytes.copy_from_slice(&header_bytes[0x14..0x18]);
        let file_count = Integer::from(field_bytes);
        let metadata_offset = usize::from(metadata_offset);
        let metadata_count = usize::from(metadata_count);
        let metadata_length = usize::from(metadata_length);
        let file_count = usize::from(file_count);
        let mut word_bytes = [0; 8];
        word_bytes.copy_from_slice(&header_bytes[0x18..0x20]);
        let length = Integer::from(word_bytes);
        word_bytes.copy_from_slice(&header_bytes[0x20..0x28]);
        let contents_offset = Integer::from(word_bytes);
        word_bytes.copy_from_slice(&header_bytes[0x28..0x30]);
        let contents_length = Integer::from(word_bytes);
        let length = usize::from(length);
        let contents_offset = usize::from(contents_offset);
        let contents_length = usize::from(contents_length);
        let mut content_identifier = [0; ContentIdentifier::LENGTH];
        content_identifier.copy_from_slice(&header_bytes[0x30..0x60]);
        let content_identifier = ContentIdentifier::try_from(content_identifier)?;
        let mut digest_bytes = [0; Self::DIGEST];
        digest_bytes.copy_from_slice(&header_bytes[0x60..0x70]);
        let digest = Bytes::from(digest_bytes);
        let mut vector_bytes = [0; Self::VECTOR];
        vector_bytes.copy_from_slice(&header_bytes[0x70..Self::LENGTH]);
        let vector = Bytes::from(vector_bytes);
        Ok(Self {
            release,
            platform,
            metadata_offset,
            metadata_count,
            metadata_length,
            file_count,
            length,
            contents_offset,
            contents_length,
            content_identifier,
            digest,
            vector,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_header_of_the_golden_weapons_pack_is_read_back_as_the_bytes_it_came_from() {
        let mut package = Blob::zeroes(Header::LENGTH);
        package.try_write(0x00, [0x7F, 0x50, 0x4B, 0x47]).unwrap();
        package.try_write(0x04, [0x80, 0x00]).unwrap();
        package.try_write(0x06, [0x00, 0x01]).unwrap();
        package.try_write(0x08, u32::to_be_bytes(0xC0)).unwrap();
        package.try_write(0x0C, u32::to_be_bytes(8)).unwrap();
        package.try_write(0x10, u32::to_be_bytes(0xC0)).unwrap();
        package.try_write(0x14, u32::to_be_bytes(12)).unwrap();
        package.try_write(0x18, u64::to_be_bytes(318864)).unwrap();
        package.try_write(0x20, u64::to_be_bytes(0x180)).unwrap();
        package.try_write(0x28, u64::to_be_bytes(318384)).unwrap();
        package
            .try_write(0x30, *b"EP0000-XBRK00000_00-XBREAKTESTLABEL0")
            .unwrap();

        let header = Header::try_from(&package).unwrap();

        let bytes = Blob::from(<[u8; Header::LENGTH]>::from(&header));
        assert_eq!(bytes, package);
    }

    #[test]
    fn the_content_identifier_of_a_header_is_the_one_its_license_names() {
        let mut package = Blob::zeroes(Header::LENGTH);
        package.try_write(0x00, [0x7F, 0x50, 0x4B, 0x47]).unwrap();
        package.try_write(0x04, [0x80, 0x00]).unwrap();
        package.try_write(0x06, [0x00, 0x01]).unwrap();
        package
            .try_write(0x30, *b"EP0000-XBRK00000_00-XBREAKTESTLABEL0")
            .unwrap();

        let header = Header::try_from(&package).unwrap();

        let content_identifier = header.content_identifier().to_string();
        assert_eq!(content_identifier, "EP0000-XBRK00000_00-XBREAKTESTLABEL0");
    }

    #[test]
    fn a_content_identifier_of_bytes_that_are_no_letters_is_an_error() {
        let mut bytes = Blob::zeroes(Header::LENGTH);
        bytes.try_write(0x00, [0x7F, 0x50, 0x4B, 0x47]).unwrap();
        bytes.try_write(0x04, [0x80, 0x00]).unwrap();
        bytes.try_write(0x06, [0x00, 0x01]).unwrap();
        bytes.try_write(0x30, [0xFF]).unwrap();

        let header = Header::try_from(&bytes);

        assert!(header.is_err());
    }

    #[test]
    fn a_file_that_does_not_begin_with_the_magic_of_a_package_is_no_package() {
        let mut package = Blob::zeroes(Header::LENGTH);
        package.try_write(0x00, *b"PSAR").unwrap();

        let header = Header::try_from(&package);

        let magic = Bytes::from([0x50, 0x53, 0x41, 0x52]);
        assert_eq!(header.unwrap_err(), Error::Magic { magic });
    }

    #[test]
    fn a_file_shorter_than_a_header_is_an_error() {
        let package = Blob::from([0x7F, 0x50, 0x4B, 0x47]);

        let header = Header::try_from(&package);

        let bytes: usize = 4;
        assert_eq!(header.unwrap_err(), Error::Length { bytes });
    }

    #[test]
    fn a_package_the_store_never_served_is_an_error() {
        let mut package = Blob::zeroes(Header::LENGTH);
        package.try_write(0x00, [0x7F, 0x50, 0x4B, 0x47]).unwrap();
        package.try_write(0x06, [0x00, 0x01]).unwrap();

        let header = Header::try_from(&package);

        let release = Integer::from([0x00, 0x00]);
        let release = release::Error::Release { release };
        assert_eq!(header.unwrap_err(), Error::Release { release });
    }

    #[test]
    fn a_package_of_a_handheld_is_an_error() {
        let mut package = Blob::zeroes(Header::LENGTH);
        package.try_write(0x00, [0x7F, 0x50, 0x4B, 0x47]).unwrap();
        package.try_write(0x04, [0x80, 0x00]).unwrap();
        package.try_write(0x06, [0x00, 0x02]).unwrap();

        let header = Header::try_from(&package);

        let platform = Integer::from([0x00, 0x02]);
        let platform = platform::Error::Platform { platform };
        assert_eq!(header.unwrap_err(), Error::Platform { platform });
    }
}
