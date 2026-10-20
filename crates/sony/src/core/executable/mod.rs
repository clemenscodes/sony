pub mod signed;

use bytes::blob::Blob;
use ddd::{DomainLayer, Layered, ValueObject};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Default, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Executable {
    bytes: Blob,
}

impl Layered for Executable {
    type Layer = DomainLayer;
}

impl ValueObject for Executable {}

impl Executable {
    pub fn bytes(&self) -> &Blob {
        &self.bytes
    }
}

impl From<Blob> for Executable {
    fn from(bytes: Blob) -> Self {
        Self { bytes }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_test::{Token, assert_tokens};

    #[test]
    fn an_executable_keeps_its_own_bytes() {
        let bytes = Blob::from(vec![0x7F, b'E', b'L', b'F']);

        let executable = Executable::from(bytes);

        assert_eq!(
            executable.bytes(),
            &Blob::from(vec![0x7F, b'E', b'L', b'F'])
        );
    }

    #[test]
    fn two_executables_of_the_same_bytes_are_the_same_executable() {
        let executable = Executable::from(Blob::from(vec![1, 2, 3]));

        let same_executable = Executable::from(Blob::from(vec![1, 2, 3]));

        assert_eq!(executable, same_executable);
    }

    #[test]
    fn an_executable_goes_out_and_comes_back_as_the_bytes_it_keeps() {
        let executable = Executable::from(Blob::from(vec![0x7F, b'E', b'L', b'F']));

        let tokens = [
            Token::Seq { len: Some(4) },
            Token::U8(0x7F),
            Token::U8(b'E'),
            Token::U8(b'L'),
            Token::U8(b'F'),
            Token::SeqEnd,
        ];

        assert_tokens(&executable, &tokens);
    }

    #[test]
    fn an_executable_of_no_bytes_goes_out_and_comes_back_as_no_bytes() {
        let executable = Executable::default();

        let tokens = [Token::Seq { len: Some(0) }, Token::SeqEnd];

        assert_tokens(&executable, &tokens);
    }
}
