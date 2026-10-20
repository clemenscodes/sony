use bytes::Integer;
use ddd::{DomainLayer, Layered, ValueObject};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Copy, Serialize, Deserialize)]
pub(in crate::core::executable::signed::executable) struct Keyset {
    revision: Integer<{ Keyset::REVISION_LENGTH }>,
    program: Integer<4>,
    key: [u8; Keyset::KEY],
    vector: [u8; Keyset::VECTOR],
}

impl Layered for Keyset {
    type Layer = DomainLayer;
}

impl ValueObject for Keyset {}

impl Keyset {
    pub(in crate::core::executable::signed::executable) const KEY: usize = 32;
    pub(in crate::core::executable::signed::executable) const VECTOR: usize = 16;
    pub(in crate::core::executable::signed::executable) const REVISION_LENGTH: usize = 2;

    pub(in crate::core::executable::signed::executable) fn key(&self) -> &[u8; Self::KEY] {
        &self.key
    }

    pub(in crate::core::executable::signed::executable) fn vector(&self) -> &[u8; Self::VECTOR] {
        &self.vector
    }

    pub(in crate::core::executable::signed::executable) fn fits(
        &self,
        revision: Integer<{ Self::REVISION_LENGTH }>,
        program: Integer<4>,
    ) -> bool {
        self.revision == revision && self.program == program
    }
}

impl
    From<(
        Integer<{ Keyset::REVISION_LENGTH }>,
        Integer<4>,
        [u8; Keyset::KEY],
        [u8; Keyset::VECTOR],
    )> for Keyset
{
    fn from(
        (revision, program, key, vector): (
            Integer<{ Keyset::REVISION_LENGTH }>,
            Integer<4>,
            [u8; Keyset::KEY],
            [u8; Keyset::VECTOR],
        ),
    ) -> Self {
        Self {
            revision,
            program,
            key,
            vector,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_test::{Token, assert_tokens};

    #[test]
    fn a_keyset_fits_the_revision_and_the_program_type_it_is_for() {
        let keyset = Keyset::from((
            Integer::from([0, 1]),
            Integer::from([0, 0, 0, 8]),
            [0; 32],
            [0; 16],
        ));

        let fits = keyset.fits(Integer::from([0, 1]), Integer::from([0, 0, 0, 8]));

        assert!(fits);
    }

    #[test]
    fn a_keyset_of_another_revision_fits_nothing_here() {
        let keyset = Keyset::from((
            Integer::from([0, 1]),
            Integer::from([0, 0, 0, 8]),
            [0; 32],
            [0; 16],
        ));

        let fits = keyset.fits(Integer::from([0, 2]), Integer::from([0, 0, 0, 8]));

        assert!(!fits);
    }

    #[test]
    fn a_keyset_of_another_program_type_fits_nothing_here() {
        let keyset = Keyset::from((
            Integer::from([0, 1]),
            Integer::from([0, 0, 0, 8]),
            [0; 32],
            [0; 16],
        ));

        let fits = keyset.fits(Integer::from([0, 1]), Integer::from([0, 0, 0, 4]));

        assert!(!fits);
    }

    #[test]
    fn a_keyset_goes_out_and_comes_back_field_for_field() {
        let key = [7; Keyset::KEY];
        let vector = [9; Keyset::VECTOR];
        let revision = Integer::from([0, 1]);
        let keyset = Keyset::from((revision, Integer::from([0, 0, 0, 8]), key, vector));

        let mut tokens = vec![
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
        tokens.extend([Token::TupleEnd, Token::StructEnd]);

        assert_tokens(&keyset, &tokens);
    }
}
