pub mod error;

mod header;

pub use error::Error;

use crate::core::content_identifier::ContentIdentifier;
use crate::core::edata::header::Header;
use crate::core::license::Klicensee;
use aes::Aes128;
use aes::cipher::BlockCipherDecrypt;
use bytes::blob::Blob;
use cmac::Cmac;
use cmac::KeyInit;
use cmac::digest::Mac;
use ddd::{DomainLayer, Layered, ValueObject};

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Copy)]
pub struct Edata {
    header: Header,
}

impl Layered for Edata {
    type Layer = DomainLayer;
}

impl ValueObject for Edata {}

impl Edata {
    const HASH: usize = 0xA0;
    const DIGEST: usize = 16;
    const KEY: [u8; Edata::DIGEST] = [
        0xBE, 0x95, 0x9C, 0xA8, 0x30, 0x8D, 0xEF, 0xA2, 0xE5, 0xE1, 0x80, 0xC6, 0x37, 0x12, 0xA9,
        0xAE,
    ];

    pub fn content_identifier(&self) -> &ContentIdentifier {
        self.header.content_identifier()
    }

    pub fn block_length(&self) -> usize {
        self.header.block_length()
    }

    pub fn length(&self) -> usize {
        self.header.length()
    }

    fn try_hash(edata_bytes: &[u8; Header::LENGTH], klicensee: &Klicensee) -> Result<(), Error> {
        let bytes = &edata_bytes[..Self::HASH];
        let hash = &edata_bytes[Self::HASH..Self::HASH + Self::DIGEST];
        let mut key = *klicensee.bytes();
        let cipher = Aes128::new(&Self::KEY.into());
        cipher.decrypt_block((&mut key).into());
        let mut mac = Cmac::<Aes128>::new(&key.into());
        mac.update(bytes);
        if mac.verify_slice(hash).is_err() {
            return Err(Error::Klicensee);
        }
        Ok(())
    }
}

impl TryFrom<(&Blob, &Klicensee)> for Edata {
    type Error = Error;

    fn try_from((edata, klicensee): (&Blob, &Klicensee)) -> Result<Self, Self::Error> {
        let edata_bytes = edata.try_bytes::<{ Header::LENGTH }>(0);
        let Ok(edata_bytes) = edata_bytes else {
            let bytes = edata.length();
            let header = header::Error::Length { bytes };
            return Err(Error::Header { header });
        };
        let edata_bytes = <[u8; Header::LENGTH]>::from(edata_bytes);
        let header = Header::try_from(&edata_bytes)?;
        Self::try_hash(&edata_bytes, klicensee)?;
        Ok(Self { header })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::license::License;

    #[test]
    fn a_klicensee_that_does_not_yield_the_hash_of_a_protected_file_does_not_open_it() {
        let mut edata = Blob::zeroes(Header::LENGTH);
        edata.try_write(0x00, *b"NPD\0").unwrap();
        edata.try_write(0x04, u32::to_be_bytes(2)).unwrap();
        edata.try_write(0x08, u32::to_be_bytes(2)).unwrap();
        edata.try_write(0x80, u32::to_be_bytes(0x0C)).unwrap();
        let license = Blob::from([0; License::LENGTH]);
        let license = License::try_from(&license).unwrap();
        let klicensee = Klicensee::from(&license);

        let edata = Edata::try_from((&edata, &klicensee));

        assert_eq!(edata.unwrap_err(), Error::Klicensee);
    }

    #[test]
    fn a_protected_file_its_klicensee_opens_states_its_content_and_its_lengths() {
        let mut edata = vec![0; Header::LENGTH];
        edata[0x00..0x04].copy_from_slice(b"NPD\0");
        edata[0x04..0x08].copy_from_slice(&u32::to_be_bytes(2));
        edata[0x08..0x0C].copy_from_slice(&u32::to_be_bytes(2));
        edata[0x10..0x1C].copy_from_slice(b"UP9000-TEST_");
        edata[0x80..0x84].copy_from_slice(&u32::to_be_bytes(0x0C));
        edata[0x84..0x88].copy_from_slice(&u32::to_be_bytes(0x4000));
        edata[0x88..0x90].copy_from_slice(&u64::to_be_bytes(0x1234));
        let license = Blob::from([0; License::LENGTH]);
        let license = License::try_from(&license).unwrap();
        let klicensee = Klicensee::from(&license);
        let mut key = *klicensee.bytes();
        let cipher = Aes128::new(&Edata::KEY.into());
        cipher.decrypt_block((&mut key).into());
        let mut mac = Cmac::<Aes128>::new_from_slice(&key).unwrap();
        mac.update(&edata[..Edata::HASH]);
        let digest = mac.finalize().into_bytes();
        edata[Edata::HASH..Edata::HASH + Edata::DIGEST].copy_from_slice(&digest);
        let edata = Blob::from(edata);

        let edata = Edata::try_from((&edata, &klicensee)).unwrap();

        assert_eq!(edata.block_length(), 0x4000);
        assert_eq!(edata.length(), 0x1234);
        assert_eq!(&edata.content_identifier().bytes()[..12], b"UP9000-TEST_");
    }

    #[test]
    fn a_file_of_the_length_of_a_header_that_is_no_protected_file_is_an_error() {
        let mut edata = vec![0; Header::LENGTH];
        edata[0x00..0x04].copy_from_slice(b"PSAR");
        let edata = Blob::from(edata);
        let license = Blob::from([0; License::LENGTH]);
        let license = License::try_from(&license).unwrap();
        let klicensee = Klicensee::from(&license);

        let edata = Edata::try_from((&edata, &klicensee));

        let magic = bytes::Bytes::from([0x50, 0x53, 0x41, 0x52]);
        let header = header::Error::Magic { magic };
        assert_eq!(edata.unwrap_err(), Error::Header { header });
    }

    #[test]
    fn a_file_that_is_no_protected_file_is_an_error_before_any_klicensee_is_tried() {
        let edata = Blob::from(*b"PSAR");
        let license = Blob::from([0; License::LENGTH]);
        let license = License::try_from(&license).unwrap();
        let klicensee = Klicensee::from(&license);

        let edata = Edata::try_from((&edata, &klicensee));

        let bytes: usize = 4;
        let header = header::Error::Length { bytes };
        assert_eq!(edata.unwrap_err(), Error::Header { header });
    }
}
