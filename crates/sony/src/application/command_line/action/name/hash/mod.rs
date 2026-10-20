mod report;

use crate::application::command_line::action::Action;
use crate::application::command_line::action::name::hash::report::Report;
use crate::application::command_line::service::Sony;
use clap::Args;
use ddd::{Layered, PresentationLayer};
use sony::identifier::Identifier;
use sony::name;
use sony::name::Name;

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Args)]
#[command(about = "The identifier a name yields, which needs no dictionary.")]
pub(crate) struct Request {
    #[arg(value_name = "NAME", help = "The name a function was given.")]
    name: String,
}

impl Layered for Request {
    type Layer = PresentationLayer;
}

impl Action for Request {
    type Report = Report;
    type Error = name::Error;

    fn try_act(&self, _: &Sony) -> Result<Self::Report, Self::Error> {
        let name = Name::try_from(self.name.as_str())?;
        let identifier = Identifier::from(&name);
        let report = Report::from((name, identifier));
        Ok(report)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_name_is_reported_together_with_the_identifier_it_yields() {
        let sony = Sony::try_from(()).unwrap();
        let request = Request {
            name: String::from("cellHttpUtilParseUri"),
        };

        let report = request.try_act(&sony).unwrap();

        let report_text = report.to_string();
        assert_eq!(report_text, "[SONY] cellHttpUtilParseUri yields 32FAAF58.");
    }

    #[test]
    fn a_writing_that_is_no_name_is_an_error() {
        let sony = Sony::try_from(()).unwrap();
        let request = Request {
            name: String::from("cell Http"),
        };

        let report = request.try_act(&sony);

        assert!(report.is_err());
    }
}
