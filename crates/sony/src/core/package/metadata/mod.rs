pub mod error;

mod entry;

pub(super) use error::Error;

use crate::core::package::header::Header;
use crate::core::package::metadata::entry::Entry;
use bytes::blob::Blob;
use ddd::{DomainLayer, Layered, ValueObject};

#[derive(Clone, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub(super) struct Metadata {
    entries: Vec<Entry>,
}

impl Layered for Metadata {
    type Layer = DomainLayer;
}

impl ValueObject for Metadata {}

impl Metadata {
    pub(super) fn count(&self) -> usize {
        self.entries.len()
    }
}

impl TryFrom<(&Blob, &Header)> for Metadata {
    type Error = Error;

    fn try_from((package, header): (&Blob, &Header)) -> Result<Self, Self::Error> {
        let offset = header.metadata_offset();
        let ceiling = usize::saturating_add(offset, header.metadata_length());
        let count = header.metadata_count();
        let mut entries = Vec::with_capacity(count);
        let mut index = offset;
        for _ in 0..count {
            let length = usize::saturating_sub(package.length(), index);
            let chunk = package.chunk(index, length);
            let Some(chunk) = chunk else {
                let bytes = package.length();
                return Err(Error::Length { bytes });
            };
            let entry = Entry::try_from(&chunk)?;
            index = usize::saturating_add(index, entry.length());
            if index > ceiling {
                return Err(Error::Metadata {
                    offset: index,
                    ceiling,
                });
            }
            entries.push(entry);
        }
        Ok(Self { entries })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_metadata_of_a_package_reads_every_entry_its_header_counts() {
        let mut package = Blob::zeroes(0x98);
        package.try_write(0x00, [0x7F, 0x50, 0x4B, 0x47]).unwrap();
        package.try_write(0x04, [0x80, 0x00]).unwrap();
        package.try_write(0x06, [0x00, 0x01]).unwrap();
        package.try_write(0x08, u32::to_be_bytes(0x80)).unwrap();
        package.try_write(0x0C, u32::to_be_bytes(2)).unwrap();
        package.try_write(0x10, u32::to_be_bytes(0x18)).unwrap();
        package.try_write(0x80, u32::to_be_bytes(2)).unwrap();
        package.try_write(0x84, u32::to_be_bytes(4)).unwrap();
        package.try_write(0x88, u32::to_be_bytes(0x1B)).unwrap();
        package.try_write(0x8C, u32::to_be_bytes(3)).unwrap();
        package.try_write(0x90, u32::to_be_bytes(4)).unwrap();
        package.try_write(0x94, u32::to_be_bytes(0x80)).unwrap();
        let header = Header::try_from(&package).unwrap();

        let metadata = Metadata::try_from((&package, &header)).unwrap();

        assert_eq!(metadata.count(), 2);
    }

    #[test]
    fn entries_that_reach_past_the_metadata_are_errors() {
        let mut package = Blob::zeroes(0xA0);
        package.try_write(0x00, [0x7F, 0x50, 0x4B, 0x47]).unwrap();
        package.try_write(0x04, [0x80, 0x00]).unwrap();
        package.try_write(0x06, [0x00, 0x01]).unwrap();
        package.try_write(0x08, u32::to_be_bytes(0x80)).unwrap();
        package.try_write(0x0C, u32::to_be_bytes(1)).unwrap();
        package.try_write(0x10, u32::to_be_bytes(0x10)).unwrap();
        package.try_write(0x80, u32::to_be_bytes(2)).unwrap();
        package.try_write(0x84, u32::to_be_bytes(0x10)).unwrap();
        let header = Header::try_from(&package).unwrap();

        let metadata = Metadata::try_from((&package, &header));

        let offset: usize = 0x98;
        let ceiling: usize = 0x90;
        assert_eq!(metadata.unwrap_err(), Error::Metadata { offset, ceiling });
    }

    #[test]
    fn the_bytes_behind_the_last_entry_are_no_entry_and_are_left_alone() {
        let mut package = Blob::zeroes(0xC0);
        package.try_write(0x00, [0x7F, 0x50, 0x4B, 0x47]).unwrap();
        package.try_write(0x04, [0x80, 0x00]).unwrap();
        package.try_write(0x06, [0x00, 0x01]).unwrap();
        package.try_write(0x08, u32::to_be_bytes(0x80)).unwrap();
        package.try_write(0x0C, u32::to_be_bytes(1)).unwrap();
        package.try_write(0x10, u32::to_be_bytes(0x40)).unwrap();
        package.try_write(0x80, u32::to_be_bytes(2)).unwrap();
        package.try_write(0x84, u32::to_be_bytes(4)).unwrap();
        let header = Header::try_from(&package).unwrap();

        let metadata = Metadata::try_from((&package, &header)).unwrap();

        assert_eq!(metadata.count(), 1);
    }
}
