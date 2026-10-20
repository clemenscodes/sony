use crate::core::license::License;
use aes::Aes128;
use aes::cipher::{BlockCipherDecrypt, KeyInit};
use bytes::Bytes;
use ddd::{DomainLayer, Layered, ValueObject};
use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Copy)]
pub struct Klicensee {
    bytes: [u8; Klicensee::LENGTH],
}

impl Layered for Klicensee {
    type Layer = DomainLayer;
}

impl ValueObject for Klicensee {}

impl Klicensee {
    pub const LENGTH: usize = 16;

    const ROUNDS: usize = 5;

    const KEY: [u8; Klicensee::LENGTH] = [
        0x86, 0x9F, 0x77, 0x45, 0xC1, 0x3F, 0xD8, 0x90, 0xCC, 0xF2, 0x91, 0x88, 0xE3, 0xCC, 0x3E,
        0xDF,
    ];

    const ORDER: [u8; Klicensee::LENGTH] = [
        0x0C, 0x03, 0x06, 0x04, 0x01, 0x0B, 0x0F, 0x08, 0x02, 0x07, 0x00, 0x05, 0x0A, 0x0E, 0x0D,
        0x09,
    ];

    const MASK: [u8; Klicensee::LENGTH] = [
        0xA9, 0x3E, 0x1F, 0xD6, 0x7C, 0x55, 0xA3, 0x29, 0xB7, 0x5F, 0xDD, 0xA6, 0x2A, 0x95, 0xC7,
        0xA5,
    ];

    const SUBTRAHEND: [u8; Klicensee::LENGTH] = [
        0x67, 0xD4, 0x5D, 0xA3, 0x29, 0x6D, 0x00, 0x6A, 0x4E, 0x7C, 0x53, 0x7B, 0xF5, 0x53, 0x8C,
        0x74,
    ];

    pub fn bytes(&self) -> &[u8; Self::LENGTH] {
        &self.bytes
    }

    fn decrypt(bytes: Bytes<{ Klicensee::LENGTH }>) -> Bytes<{ Klicensee::LENGTH }> {
        let mut key = <[u8; Klicensee::LENGTH]>::from(bytes);
        let cipher = Aes128::new(&Self::KEY.into());
        let block = (&mut key).into();
        cipher.decrypt_block(block);
        Bytes::from(key)
    }

    fn mask(bytes: Bytes<{ Klicensee::LENGTH }>) -> Bytes<{ Klicensee::LENGTH }> {
        let mut key = <[u8; Klicensee::LENGTH]>::from(bytes);
        for order in Self::ORDER {
            let index = usize::from(order);
            let mask = Self::MASK[index];
            key[index] ^= mask;
        }
        Bytes::from(key)
    }

    fn mix(bytes: Bytes<{ Klicensee::LENGTH }>) -> Bytes<{ Klicensee::LENGTH }> {
        let mut key = <[u8; Klicensee::LENGTH]>::from(bytes);
        for order in (1..Self::LENGTH).rev() {
            let index = usize::from(Self::ORDER[order]);
            let mask_index = usize::from(Self::ORDER[order - 1]);
            key[index] ^= key[mask_index];
        }
        Bytes::from(key)
    }

    fn subtract(bytes: Bytes<{ Klicensee::LENGTH }>) -> Bytes<{ Klicensee::LENGTH }> {
        let mut key = <[u8; Klicensee::LENGTH]>::from(bytes);
        let mut borrow: u8 = 0;
        for order in Self::ORDER {
            let index = usize::from(order);
            let byte = key[index];
            let value = Self::difference(byte, borrow);
            let subtrahend = Self::SUBTRAHEND[index];
            if borrow != 1 || value != u8::MAX {
                borrow = u8::from(value < subtrahend);
            }
            key[index] = Self::difference(value, subtrahend);
        }
        Bytes::from(key)
    }

    fn difference(value: u8, subtrahend: u8) -> u8 {
        if value >= subtrahend {
            return value - subtrahend;
        }
        value + (u8::MAX - subtrahend) + 1
    }
}

impl From<&License> for Klicensee {
    fn from(license: &License) -> Self {
        let key = *license.bytes();
        let key = Bytes::from(key);
        let mut key = Self::decrypt(key);
        for _ in 0..Self::ROUNDS {
            key = Self::mask(key);
            key = Self::mix(key);
            key = Self::subtract(key);
        }
        let bytes = <[u8; Self::LENGTH]>::from(key);
        Self { bytes }
    }
}

impl fmt::Display for Klicensee {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut text = String::new();
        for byte in self.bytes {
            let letters = format!("{byte:02X}");
            text.push_str(&letters);
        }
        fmt::Display::fmt(&text, formatter)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::blob::Blob;

    #[test]
    fn a_key_of_zeroes_borrows_across_a_byte_that_is_already_the_highest() {
        let key = Bytes::from([0; Klicensee::LENGTH]);

        let difference = Klicensee::subtract(key);

        let bytes = [
            0x98, 0x2B, 0xA2, 0x5C, 0xD6, 0x92, 0xFF, 0x95, 0xB1, 0x83, 0xAC, 0x84, 0x0B, 0xAC,
            0x73, 0x8B,
        ];
        assert_eq!(difference, Bytes::from(bytes));
    }

    #[test]
    fn a_klicensee_is_sixteen_bytes_and_is_shown_in_upper_case() {
        let license = Blob::from([0; License::LENGTH]);
        let license = License::try_from(&license).unwrap();

        let klicensee = Klicensee::from(&license);

        let letters = klicensee.to_string();
        assert_eq!(letters.len(), 32);
        assert_eq!(letters, letters.to_uppercase());
    }

    #[test]
    fn two_licenses_that_differ_yield_two_klicensees_that_differ() {
        let blob = Blob::from([0; License::LENGTH]);
        let license = License::try_from(&blob).unwrap();
        let different_blob = Blob::from([1; License::LENGTH]);
        let different_license = License::try_from(&different_blob).unwrap();

        let klicensee = Klicensee::from(&license);

        let different_klicensee = Klicensee::from(&different_license);
        assert_ne!(klicensee, different_klicensee);
    }

    #[test]
    fn the_same_license_yields_the_same_klicensee_every_time() {
        let license = Blob::from([0xCC; License::LENGTH]);
        let license = License::try_from(&license).unwrap();

        let klicensee = Klicensee::from(&license);

        let same_klicensee = Klicensee::from(&license);
        assert_eq!(klicensee, same_klicensee);
    }
}
