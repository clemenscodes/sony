mod resolve;

use crate::application::command_line::action::identifier::resolve::Request;
use clap::Subcommand;

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Subcommand)]
pub(crate) enum IdentifierAction {
    Resolve(Request),
}
