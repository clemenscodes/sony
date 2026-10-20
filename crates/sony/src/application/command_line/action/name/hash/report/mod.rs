use ddd::{Layered, PresentationLayer};
use sony::identifier::Identifier;
use sony::name::Name;
use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub(crate) struct Report {
    name: Name,
    identifier: Identifier,
}

impl Layered for Report {
    type Layer = PresentationLayer;
}

impl From<(Name, Identifier)> for Report {
    fn from((name, identifier): (Name, Identifier)) -> Self {
        Self { name, identifier }
    }
}

impl fmt::Display for Report {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = &self.name;
        let identifier = &self.identifier;
        write!(formatter, "[SONY] {name} yields {identifier}.")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_report_names_the_name_and_the_identifier_it_yields() {
        let name = Name::try_from("cellHttpUtilParseUri").unwrap();
        let identifier = Identifier::from(&name);

        let report = Report::from((name, identifier));

        let report_text = report.to_string();
        assert_eq!(report_text, "[SONY] cellHttpUtilParseUri yields 32FAAF58.");
    }
}
