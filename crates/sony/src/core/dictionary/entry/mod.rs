use crate::core::identifier::Identifier;
use crate::core::name;
use crate::core::name::Name;
use ddd::{DomainLayer, Layered, ValueObject};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(into = "String", try_from = "String")]
pub(super) struct Entry {
    name: Name,
    identifier: Identifier,
}

impl Layered for Entry {
    type Layer = DomainLayer;
}

impl ValueObject for Entry {}

impl Entry {
    pub(super) fn name(&self) -> &Name {
        &self.name
    }

    pub(super) fn identifier(&self) -> Identifier {
        self.identifier
    }
}

impl From<Name> for Entry {
    fn from(name: Name) -> Self {
        let identifier = Identifier::from(&name);
        Self { name, identifier }
    }
}

impl TryFrom<&str> for Entry {
    type Error = name::Error;

    fn try_from(text: &str) -> Result<Self, Self::Error> {
        let name = Name::try_from(text)?;
        Ok(Self::from(name))
    }
}

impl TryFrom<String> for Entry {
    type Error = name::Error;

    fn try_from(text: String) -> Result<Self, Self::Error> {
        Self::try_from(text.as_str())
    }
}

impl From<Entry> for String {
    fn from(entry: Entry) -> Self {
        entry.name.text().to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::name;
    use serde_test::{Token, assert_de_tokens_error, assert_tokens};

    #[test]
    fn an_entry_keeps_its_own_name() {
        let text = "cellHttpUtilParseUri";

        let entry = Entry::try_from(text).unwrap();

        let entry_text = entry.name().to_string();
        assert_eq!(entry_text, "cellHttpUtilParseUri");
    }

    #[test]
    fn an_entry_keeps_the_identifier_its_name_yields() {
        let text = "cellHttpUtilParseUri";

        let entry = Entry::try_from(text).unwrap();

        let entry_text = entry.identifier().to_string();
        assert_eq!(entry_text, "32FAAF58");
    }

    #[test]
    fn a_writing_that_is_no_name_yields_no_entry() {
        let text = "cell Http";

        let entry = Entry::try_from(text);

        assert!(entry.is_err());
    }

    #[test]
    fn one_name_yields_the_same_entry_every_time() {
        let text = "sceNpLookupInit";

        let entry = Entry::try_from(text).unwrap();

        let same_entry = Entry::try_from(text).unwrap();
        assert_eq!(entry, same_entry);
    }

    #[test]
    fn an_entry_goes_out_and_comes_back_as_its_own_name() {
        let entry = Entry::try_from("cellHttpUtilParseUri").unwrap();

        let tokens = [Token::Str("cellHttpUtilParseUri")];

        assert_tokens(&entry, &tokens);
    }

    #[test]
    fn a_writing_that_is_no_name_yields_no_entry_on_the_way_back_either() {
        let text = String::from("cell Http");
        let tokens = [Token::Str("cell Http")];

        let error_text = name::Error::Name { text }.to_string();

        assert_de_tokens_error::<Entry>(&tokens, &error_text);
    }
}
