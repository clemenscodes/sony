mod identifier;
mod name;

pub(super) use identifier::IdentifierAction;
pub(super) use name::NameAction;

use crate::application::command_line::service::Sony;
use ddd::{Layered, PresentationLayer};
use std::fmt::Display;

pub(super) trait Action: Layered<Layer = PresentationLayer> {
    type Report: Display;
    type Error: Display;

    fn try_act(&self, sony: &Sony) -> Result<Self::Report, Self::Error>;
}
