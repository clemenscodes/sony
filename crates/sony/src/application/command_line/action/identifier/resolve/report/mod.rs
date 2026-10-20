use ddd::{Layered, PresentationLayer};
use sony::identifier::Identifier;
use sony::name::Name;
use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub(crate) struct Report {
    identifier: Identifier,
    name: Name,
}

impl Layered for Report {
    type Layer = PresentationLayer;
}

impl From<(Identifier, Name)> for Report {
    fn from((identifier, name): (Identifier, Name)) -> Self {
        Self { identifier, name }
    }
}

impl fmt::Display for Report {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let identifier = &self.identifier;
        let name = &self.name;
        write!(
            formatter,
            "[SONY] {identifier} is the identifier of {name}."
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_report_names_the_identifier_and_the_name_it_means() {
        let identifier = Identifier::try_from("32FAAF58").unwrap();
        let name = Name::try_from("cellHttpUtilParseUri").unwrap();

        let report = Report::from((identifier, name));

        let report_text = report.to_string();
        assert_eq!(
            report_text,
            "[SONY] 32FAAF58 is the identifier of cellHttpUtilParseUri."
        );
    }
}
