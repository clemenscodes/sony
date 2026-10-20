pub mod executable;

pub use executable::SignedExecutable;

use crate::core::executable::Executable;
use std::fmt::Display;

pub trait Signed {
    type Error: Display;

    fn try_decrypt(&self) -> Result<Executable, Self::Error>;
}
