mod hash;

use crate::application::command_line::action::name::hash::Request;
use clap::Subcommand;

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Subcommand)]
pub(crate) enum NameAction {
    Hash(Request),
}
