pub mod error;

pub(super) use error::Error;

use bytes::Integer;
use bytes::blob::Blob;
use ddd::{DomainLayer, Layered, ValueObject};

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub(super) struct Entry {
    bytes: Blob,
}

impl Layered for Entry {
    type Layer = DomainLayer;
}

impl ValueObject for Entry {}

impl Entry {
    pub(super) const LENGTH: usize = 8;

    pub(super) fn length(&self) -> usize {
        usize::saturating_add(Self::LENGTH, self.bytes.length())
    }
}

impl TryFrom<&Blob> for Entry {
    type Error = Error;

    fn try_from(metadata: &Blob) -> Result<Self, Self::Error> {
        let length = Integer::from(metadata.try_bytes::<4>(0x04)?);
        let length = usize::from(length);
        let bytes = metadata.chunk(Self::LENGTH, length);
        let Some(bytes) = bytes else {
            let bytes = metadata.length();
            return Err(Error::Length { bytes });
        };
        Ok(Self { bytes })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_entry_of_the_metadata_is_eight_bytes_and_the_bytes_it_names() {
        let mut metadata = Blob::zeroes(0x10);
        metadata.try_write(0x00, u32::to_be_bytes(2)).unwrap();
        metadata.try_write(0x04, u32::to_be_bytes(4)).unwrap();
        metadata.try_write(0x08, u32::to_be_bytes(0x1B)).unwrap();

        let entry = Entry::try_from(&metadata).unwrap();

        assert_eq!(entry.length(), 12);
    }

    #[test]
    fn an_entry_of_no_bytes_is_eight_bytes_long_all_the_same() {
        let metadata = Blob::zeroes(Entry::LENGTH);

        let entry = Entry::try_from(&metadata).unwrap();

        assert_eq!(entry.length(), Entry::LENGTH);
    }

    #[test]
    fn an_entry_that_names_more_bytes_than_follow_it_is_an_error() {
        let mut metadata = Blob::zeroes(0x10);
        metadata.try_write(0x04, u32::to_be_bytes(64)).unwrap();

        let entry = Entry::try_from(&metadata);

        let bytes: usize = 0x10;
        assert_eq!(entry.unwrap_err(), Error::Length { bytes });
    }
}
