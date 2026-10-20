pub mod error;

mod klicensee;

pub use error::Error;
pub use klicensee::Klicensee;

use bytes::blob::Blob;
use ddd::{DomainLayer, Layered, ValueObject};

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Copy)]
pub struct License {
    bytes: [u8; License::LENGTH],
}

impl Layered for License {
    type Layer = DomainLayer;
}

impl ValueObject for License {}

impl License {
    pub const LENGTH: usize = 16;

    pub fn bytes(&self) -> &[u8; Self::LENGTH] {
        &self.bytes
    }
}

impl TryFrom<&Blob> for License {
    type Error = Error;

    fn try_from(license: &Blob) -> Result<Self, Self::Error> {
        if license.length() > Self::LENGTH {
            let bytes = license.length();
            return Err(Error::Length { bytes });
        }
        let bytes = license.try_bytes::<{ Self::LENGTH }>(0);
        let Ok(bytes) = bytes else {
            let bytes = license.length();
            return Err(Error::Length { bytes });
        };
        let bytes = <[u8; Self::LENGTH]>::from(bytes);
        Ok(Self { bytes })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_license_is_the_sixteen_bytes_of_the_file_it_was_read_from() {
        let license = Blob::from([0xCC; License::LENGTH]);

        let license = License::try_from(&license).unwrap();

        assert_eq!(license.bytes(), &[0xCC; License::LENGTH]);
    }

    #[test]
    fn a_file_longer_than_a_license_is_no_license() {
        let license = Blob::zeroes(20);

        let license = License::try_from(&license);

        let bytes: usize = 20;
        assert_eq!(license.unwrap_err(), Error::Length { bytes });
    }

    #[test]
    fn a_file_of_no_bytes_is_no_license() {
        let license = Blob::default();

        let license = License::try_from(&license);

        let bytes: usize = 0;
        assert_eq!(license.unwrap_err(), Error::Length { bytes });
    }
}
