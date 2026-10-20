use ddd::{ApplicationLayer, ApplicationService, Layered};

use sony::dictionary;
use sony::dictionary::Dictionary;
use sony::identifier::Identifier;
use sony::name::Name;

pub(super) struct Sony {
    dictionary: Dictionary,
}

impl Layered for Sony {
    type Layer = ApplicationLayer;
}

impl ApplicationService for Sony {}

impl Sony {
    pub(super) fn try_resolve(&self, identifier: &Identifier) -> Result<Name, dictionary::Error> {
        let name = self.dictionary.try_resolve(identifier)?;
        Ok(name.clone())
    }
}

impl TryFrom<()> for Sony {
    type Error = dictionary::Error;

    fn try_from(_: ()) -> Result<Self, Self::Error> {
        let dictionary = Dictionary::try_from(())?;
        Ok(Self { dictionary })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sony::dictionary;

    #[test]
    fn an_identifier_the_disc_writes_is_resolved_to_its_name() {
        let sony = Sony::try_from(()).unwrap();
        let identifier = Identifier::try_from("32FAAF58").unwrap();

        let name = sony.try_resolve(&identifier).unwrap();

        let name_text = name.to_string();
        assert_eq!(name_text, "cellHttpUtilParseUri");
    }

    #[test]
    fn a_socket_call_is_resolved_from_the_identifier_the_disc_writes() {
        let sony = Sony::try_from(()).unwrap();
        let identifier = Identifier::try_from("9647570B").unwrap();

        let name = sony.try_resolve(&identifier).unwrap();

        let name_text = name.to_string();
        assert_eq!(name_text, "sendto");
    }

    #[test]
    fn an_identifier_no_name_yields_is_an_error() {
        let sony = Sony::try_from(()).unwrap();
        let identifier = Identifier::try_from("DEADBEEF").unwrap();

        let name = sony.try_resolve(&identifier);

        let identifier = String::from("DEADBEEF");
        let error = dictionary::Error::Unknown { identifier };
        assert_eq!(name.unwrap_err(), error);
    }

    #[test]
    fn a_resolved_name_yields_the_identifier_it_was_resolved_from() {
        let sony = Sony::try_from(()).unwrap();
        let identifier = Identifier::try_from("9647570B").unwrap();

        let name = sony.try_resolve(&identifier).unwrap();

        assert_eq!(Identifier::from(&name), identifier);
    }
}
