pub(super) mod error;

use crate::core::executable::signed::executable::header::Header;
use aes::Aes128;
use aes::cipher::{BlockCipherDecrypt, KeyInit};
use bytes::Integer;
use bytes::blob::Blob;
use ddd::{DomainLayer, Layered, ValueObject};
use serde::{Deserialize, Serialize};

pub(super) use error::Error;

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Copy, Serialize, Deserialize)]
pub(super) enum License {
    Free,
    None,
}

impl Layered for License {
    type Layer = DomainLayer;
}

impl ValueObject for License {}

impl License {
    const CONTROL: u32 = 3;
    const MAGIC: &'static [u8] = b"NPD\0";
    const FREE_LICENSE: u32 = 3;
    const KLICENSEE: [u8; 16] = [
        0x72, 0xF9, 0x90, 0x78, 0x8F, 0x9C, 0xFF, 0x74, 0x57, 0x25, 0xF0, 0x8E, 0x4C, 0x12, 0x83,
        0x87,
    ];
    const KEY: [u8; 16] = [
        0xF2, 0xFB, 0xCA, 0x7A, 0x75, 0xB0, 0x4E, 0xDC, 0x13, 0x90, 0x63, 0x8C, 0xCD, 0xFD, 0xD1,
        0xEE,
    ];

    pub(super) fn key(&self) -> Option<[u8; 16]> {
        match self {
            Self::None => None,
            Self::Free => {
                let key = (&Self::KEY).into();
                let cipher = Aes128::new(key);
                let mut bytes = Self::KLICENSEE;
                let block = (&mut bytes).into();
                cipher.decrypt_block(block);
                Some(bytes)
            }
        }
    }
}

impl TryFrom<(&Header, &Blob)> for License {
    type Error = Error;

    fn try_from((header, signed_bytes): (&Header, &Blob)) -> Result<Self, Self::Error> {
        let mut offset = header.control_offset();
        let last_path = offset + header.control_length();
        while offset + 16 <= last_path {
            let kind = signed_bytes.try_bytes::<4>(offset);
            let length = signed_bytes.try_bytes::<4>(offset + 4);
            let (Ok(kind), Ok(length)) = (kind, length) else {
                let bytes = signed_bytes.length();
                return Err(Error::Length { bytes });
            };
            let kind = Integer::<4>::from(kind);
            let length = Integer::<4>::from(length);
            let kind = u32::from(kind);
            let length = usize::from(length);
            if length < 16 {
                let bytes = signed_bytes.length();
                return Err(Error::Length { bytes });
            }
            if kind != Self::CONTROL {
                offset += length;
                continue;
            }
            let magic = signed_bytes.chunk(offset + 16, Self::MAGIC.len());
            if magic != Some(Blob::from(Self::MAGIC)) {
                let bytes = signed_bytes.length();
                return Err(Error::Length { bytes });
            }
            let license = signed_bytes.try_bytes::<4>(offset + 24);
            let Ok(license) = license else {
                let bytes = signed_bytes.length();
                return Err(Error::Length { bytes });
            };
            let license = Integer::<4>::from(license);
            if u32::from(license) != Self::FREE_LICENSE {
                return Err(Error::License { license });
            }
            return Ok(Self::Free);
        }
        Ok(Self::None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::de::IntoDeserializer;
    use serde::de::value;

    #[test]
    fn a_free_license_yields_a_key_and_no_license_yields_none() {
        let free_license = License::Free;

        let key = free_license.key();

        assert!(key.is_some());
        assert_eq!(License::None.key(), None);
    }

    #[test]
    fn the_key_of_a_free_license_is_the_same_every_time() {
        let key = License::Free.key();

        let same_key = License::Free.key();

        assert_eq!(key, same_key);
    }

    #[test]
    fn a_signed_executable_without_control_information_has_no_license() {
        let header = Header::default();
        let signed_bytes = Blob::from(vec![0; 0x40]);

        let license = License::try_from((&header, &signed_bytes)).unwrap();

        assert_eq!(license, License::None);
    }

    #[test]
    fn control_information_that_states_a_free_license_yields_a_free_license() {
        let mut signed_bytes = vec![0; 0x100];
        signed_bytes[..4].copy_from_slice(b"SCE\0");
        signed_bytes[0x58..0x60].copy_from_slice(&u64::to_be_bytes(0x80));
        signed_bytes[0x60..0x68].copy_from_slice(&u64::to_be_bytes(0x40));
        signed_bytes[0x80..0x84].copy_from_slice(&u32::to_be_bytes(3));
        signed_bytes[0x84..0x88].copy_from_slice(&u32::to_be_bytes(0x40));
        signed_bytes[0x90..0x94].copy_from_slice(b"NPD\0");
        signed_bytes[0x98..0x9C].copy_from_slice(&u32::to_be_bytes(3));
        let signed_bytes = Blob::from(signed_bytes);
        let header = Header::try_from(&signed_bytes).unwrap();

        let license = License::try_from((&header, &signed_bytes)).unwrap();

        assert_eq!(license, License::Free);
    }

    #[test]
    fn control_information_of_another_kind_is_passed_over_and_leaves_no_license() {
        let mut signed_bytes = vec![0; 0x100];
        signed_bytes[..4].copy_from_slice(b"SCE\0");
        signed_bytes[0x58..0x60].copy_from_slice(&u64::to_be_bytes(0x80));
        signed_bytes[0x60..0x68].copy_from_slice(&u64::to_be_bytes(0x40));
        signed_bytes[0x80..0x84].copy_from_slice(&u32::to_be_bytes(1));
        signed_bytes[0x84..0x88].copy_from_slice(&u32::to_be_bytes(0x40));
        let signed_bytes = Blob::from(signed_bytes);
        let header = Header::try_from(&signed_bytes).unwrap();

        let license = License::try_from((&header, &signed_bytes)).unwrap();

        assert_eq!(license, License::None);
    }

    #[test]
    fn control_information_shorter_than_its_own_head_is_an_error() {
        let mut signed_bytes = vec![0; 0x100];
        signed_bytes[..4].copy_from_slice(b"SCE\0");
        signed_bytes[0x58..0x60].copy_from_slice(&u64::to_be_bytes(0x80));
        signed_bytes[0x60..0x68].copy_from_slice(&u64::to_be_bytes(0x40));
        signed_bytes[0x80..0x84].copy_from_slice(&u32::to_be_bytes(3));
        signed_bytes[0x84..0x88].copy_from_slice(&u32::to_be_bytes(8));
        let signed_bytes = Blob::from(signed_bytes);
        let header = Header::try_from(&signed_bytes).unwrap();

        let license = License::try_from((&header, &signed_bytes));

        let bytes: usize = 0x100;
        assert_eq!(license.unwrap_err(), Error::Length { bytes });
    }

    #[test]
    fn control_information_that_does_not_begin_with_npd_is_an_error() {
        let mut signed_bytes = vec![0; 0x100];
        signed_bytes[..4].copy_from_slice(b"SCE\0");
        signed_bytes[0x58..0x60].copy_from_slice(&u64::to_be_bytes(0x80));
        signed_bytes[0x60..0x68].copy_from_slice(&u64::to_be_bytes(0x40));
        signed_bytes[0x80..0x84].copy_from_slice(&u32::to_be_bytes(3));
        signed_bytes[0x84..0x88].copy_from_slice(&u32::to_be_bytes(0x40));
        let signed_bytes = Blob::from(signed_bytes);
        let header = Header::try_from(&signed_bytes).unwrap();

        let license = License::try_from((&header, &signed_bytes));

        let bytes: usize = 0x100;
        assert_eq!(license.unwrap_err(), Error::Length { bytes });
    }

    #[test]
    fn a_license_this_repository_has_no_key_for_is_named_as_the_number_it_states() {
        let mut signed_bytes = vec![0; 0x100];
        signed_bytes[..4].copy_from_slice(b"SCE\0");
        signed_bytes[0x58..0x60].copy_from_slice(&u64::to_be_bytes(0x80));
        signed_bytes[0x60..0x68].copy_from_slice(&u64::to_be_bytes(0x40));
        signed_bytes[0x80..0x84].copy_from_slice(&u32::to_be_bytes(3));
        signed_bytes[0x84..0x88].copy_from_slice(&u32::to_be_bytes(0x40));
        signed_bytes[0x90..0x94].copy_from_slice(b"NPD\0");
        signed_bytes[0x98..0x9C].copy_from_slice(&u32::to_be_bytes(2));
        let signed_bytes = Blob::from(signed_bytes);
        let header = Header::try_from(&signed_bytes).unwrap();

        let response = License::try_from((&header, &signed_bytes));

        let license = Integer::from([0, 0, 0, 2]);
        assert_eq!(response.unwrap_err(), Error::License { license });
    }

    #[test]
    fn control_information_that_ends_before_its_license_is_an_error() {
        let mut signed_bytes = vec![0; 0x100];
        signed_bytes[..4].copy_from_slice(b"SCE\0");
        signed_bytes[0x58..0x60].copy_from_slice(&u64::to_be_bytes(0xE8));
        signed_bytes[0x60..0x68].copy_from_slice(&u64::to_be_bytes(0x40));
        signed_bytes[0xE8..0xEC].copy_from_slice(&u32::to_be_bytes(3));
        signed_bytes[0xEC..0xF0].copy_from_slice(&u32::to_be_bytes(0x40));
        signed_bytes[0xF8..0xFC].copy_from_slice(b"NPD\0");
        let signed_bytes = Blob::from(signed_bytes);
        let header = Header::try_from(&signed_bytes).unwrap();

        let license = License::try_from((&header, &signed_bytes));

        let bytes: usize = 0x100;
        assert_eq!(license.unwrap_err(), Error::Length { bytes });
    }

    #[test]
    fn control_information_that_begins_past_the_file_is_an_error() {
        let mut signed_bytes = vec![0; 0x100];
        signed_bytes[..4].copy_from_slice(b"SCE\0");
        signed_bytes[0x58..0x60].copy_from_slice(&u64::to_be_bytes(0xFF0));
        signed_bytes[0x60..0x68].copy_from_slice(&u64::to_be_bytes(0x40));
        let signed_bytes = Blob::from(signed_bytes);
        let header = Header::try_from(&signed_bytes).unwrap();

        let license = License::try_from((&header, &signed_bytes));

        let bytes: usize = 0x100;
        assert_eq!(license.unwrap_err(), Error::Length { bytes });
    }

    #[test]
    fn a_license_comes_back_from_its_own_name() {
        let deserializer = IntoDeserializer::<value::Error>::into_deserializer("Free");

        let license = License::deserialize(deserializer).unwrap();

        assert_eq!(license, License::Free);
    }
}
