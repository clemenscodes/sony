pub(super) mod error;
mod keyset;

pub(super) use error::Error;
pub(super) use keyset::Keyset;

use crate::core::executable::signed::executable::header::Header;
use bytes::Integer;
use ddd::{DomainLayer, Layered, ValueObject};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Default, Serialize, Deserialize)]
#[serde(transparent)]
pub(super) struct Vault {
    keysets: Vec<Keyset>,
}

impl Layered for Vault {
    type Layer = DomainLayer;
}

impl ValueObject for Vault {}

impl Vault {
    const NPDRM: [u8; 4] = [0, 0, 0, 8];
    const APPLICATION: [u8; 4] = [0, 0, 0, 4];
    const KEY_REVISION: [u8; Keyset::REVISION_LENGTH] = [0, 1];

    pub(super) fn try_keyset(&self, header: &Header) -> Result<Keyset, Error> {
        let revision = header.revision();
        let program = header.program();
        let keyset = self
            .keysets
            .iter()
            .find(|keyset| keyset.fits(revision, program));
        let Some(keyset) = keyset else {
            return Err(Error::Keyset { revision, program });
        };
        Ok(*keyset)
    }
}

impl From<()> for Vault {
    fn from(_: ()) -> Self {
        let key = [
            0xF9, 0xED, 0xD0, 0x30, 0x1F, 0x77, 0x0F, 0xAB, 0xBA, 0x88, 0x63, 0xD9, 0x89, 0x7F,
            0x0F, 0xEA, 0x65, 0x51, 0xB0, 0x94, 0x31, 0xF6, 0x13, 0x12, 0x65, 0x4E, 0x28, 0xF4,
            0x35, 0x33, 0xEA, 0x6B,
        ];
        let vector = [
            0xA5, 0x51, 0xCC, 0xB4, 0xA4, 0x2C, 0x37, 0xA7, 0x34, 0xA2, 0xB4, 0xF9, 0x65, 0x7D,
            0x55, 0x40,
        ];
        let revision = Integer::from(Self::KEY_REVISION);
        let npdrm = Keyset::from((revision, Integer::from(Self::NPDRM), key, vector));
        let key = [
            0x79, 0x48, 0x18, 0x39, 0xC4, 0x06, 0xA6, 0x32, 0xBD, 0xB4, 0xAC, 0x09, 0x3D, 0x73,
            0xD9, 0x9A, 0xE1, 0x58, 0x7F, 0x24, 0xCE, 0x7E, 0x69, 0x19, 0x2C, 0x1C, 0xD0, 0x01,
            0x02, 0x74, 0xA8, 0xAB,
        ];
        let vector = [
            0x6F, 0x0F, 0x25, 0xE1, 0xC8, 0xC4, 0xB7, 0xAE, 0x70, 0xDF, 0x96, 0x8B, 0x04, 0x52,
            0x1D, 0xDA,
        ];
        let application = Keyset::from((revision, Integer::from(Self::APPLICATION), key, vector));
        let keysets = vec![npdrm, application];
        Self { keysets }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::blob::Blob;
    use serde_test::{Token, assert_tokens};

    #[test]
    fn the_vault_has_the_keyset_of_the_first_revision() {
        let vault = Vault::from(());
        let mut signed_bytes = vec![0; 0x100];
        signed_bytes[..4].copy_from_slice(b"SCE\0");
        signed_bytes[0x08..0x0A].copy_from_slice(&u16::to_be_bytes(1));
        signed_bytes[0x7C..0x80].copy_from_slice(&u32::to_be_bytes(8));
        let header = Header::try_from(&Blob::from(signed_bytes)).unwrap();

        let keyset = vault.try_keyset(&header);

        assert!(keyset.is_ok());
    }

    #[test]
    fn the_vault_has_a_keyset_for_a_program_that_is_no_npdrm() {
        let vault = Vault::from(());
        let mut signed_bytes = vec![0; 0x100];
        signed_bytes[..4].copy_from_slice(b"SCE\0");
        signed_bytes[0x08..0x0A].copy_from_slice(&u16::to_be_bytes(1));
        signed_bytes[0x7C..0x80].copy_from_slice(&u32::to_be_bytes(4));
        let header = Header::try_from(&Blob::from(signed_bytes)).unwrap();

        let keyset = vault.try_keyset(&header);

        assert!(keyset.is_ok());
    }

    #[test]
    fn a_revision_the_vault_has_no_keyset_for_is_an_error() {
        let vault = Vault::from(());
        let mut signed_bytes = vec![0; 0x100];
        signed_bytes[..4].copy_from_slice(b"SCE\0");
        signed_bytes[0x08..0x0A].copy_from_slice(&u16::to_be_bytes(7));
        signed_bytes[0x7C..0x80].copy_from_slice(&u32::to_be_bytes(8));
        let header = Header::try_from(&Blob::from(signed_bytes)).unwrap();

        let keyset = vault.try_keyset(&header);

        let error = Error::Keyset {
            revision: Integer::from([0, 7]),
            program: Integer::from([0, 0, 0, 8]),
        };
        assert_eq!(keyset.unwrap_err(), error);
    }

    #[test]
    fn a_vault_goes_out_and_comes_back_as_the_keysets_it_keeps() {
        let key = [7; Keyset::KEY];
        let vector = [9; Keyset::VECTOR];
        let revision = Integer::from(Vault::KEY_REVISION);
        let keyset = Keyset::from((revision, Integer::from(Vault::NPDRM), key, vector));
        let keysets = vec![keyset];
        let vault = Vault { keysets };

        let mut tokens = vec![
            Token::Seq { len: Some(1) },
            Token::Struct {
                name: "Keyset",
                len: 4,
            },
            Token::Str("revision"),
            Token::Str("0001"),
            Token::Str("program"),
            Token::Str("00000008"),
            Token::Str("key"),
            Token::Tuple { len: Keyset::KEY },
        ];
        tokens.extend(key.iter().copied().map(Token::U8));
        tokens.extend([
            Token::TupleEnd,
            Token::Str("vector"),
            Token::Tuple {
                len: Keyset::VECTOR,
            },
        ]);
        tokens.extend(vector.iter().copied().map(Token::U8));
        tokens.extend([Token::TupleEnd, Token::StructEnd, Token::SeqEnd]);

        assert_tokens(&vault, &tokens);
    }

    #[test]
    fn a_vault_of_no_keysets_goes_out_and_comes_back_as_no_keysets() {
        let vault = Vault::default();

        let tokens = [Token::Seq { len: Some(0) }, Token::SeqEnd];

        assert_tokens(&vault, &tokens);
    }
}
