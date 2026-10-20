mod error;

mod report;

use crate::application::command_line::action::Action;
use crate::application::command_line::action::identifier::resolve::error::Error;
use crate::application::command_line::action::identifier::resolve::report::Report;
use crate::application::command_line::service::Sony;
use clap::Args;
use ddd::{Layered, PresentationLayer};
use sony::identifier::Identifier;

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Args)]
#[command(about = "The name an identifier means, where a name for it is known.")]
pub(crate) struct Request {
    #[arg(
        value_name = "IDENTIFIER",
        help = "The identifier a module table writes, eight upper case hexadecimal digits."
    )]
    identifier: String,
}

impl Layered for Request {
    type Layer = PresentationLayer;
}

impl Action for Request {
    type Report = Report;
    type Error = Error;

    fn try_act(&self, sony: &Sony) -> Result<Self::Report, Self::Error> {
        let identifier = Identifier::try_from(self.identifier.as_str())?;
        let name = sony.try_resolve(&identifier)?;
        let report = Report::from((identifier, name));
        Ok(report)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_identifier_is_reported_together_with_the_name_it_means() {
        let sony = Sony::try_from(()).unwrap();
        let request = Request {
            identifier: String::from("32FAAF58"),
        };

        let report = request.try_act(&sony).unwrap();

        let report_text = report.to_string();
        assert_eq!(
            report_text,
            "[SONY] 32FAAF58 is the identifier of cellHttpUtilParseUri."
        );
    }

    #[test]
    fn an_identifier_no_name_yields_is_an_error() {
        let sony = Sony::try_from(()).unwrap();
        let request = Request {
            identifier: String::from("DEADBEEF"),
        };

        let report = request.try_act(&sony);

        assert!(report.is_err());
    }

    #[test]
    fn a_writing_that_is_no_identifier_is_an_error() {
        let sony = Sony::try_from(()).unwrap();
        let request = Request {
            identifier: String::from("32faaf58"),
        };

        let report = request.try_act(&sony);

        assert!(report.is_err());
    }
}
