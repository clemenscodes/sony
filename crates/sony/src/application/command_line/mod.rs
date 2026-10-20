mod action;
mod outcome;
mod service;

use crate::application::command_line::action::Action;
use crate::application::command_line::action::IdentifierAction;
use crate::application::command_line::action::NameAction;
use crate::application::command_line::outcome::Outcome;
use crate::application::command_line::service::Sony;
use clap::Parser;
use ddd::{Layered, PresentationLayer};
use std::io::{self, Write};

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Parser)]
#[command(
    name = "sony",
    about = "What one platform holder decided, as far as it can be derived."
)]
pub(crate) enum CommandLine {
    #[command(about = "The name a function was given.")]
    Name {
        #[command(subcommand)]
        action: NameAction,
    },
    #[command(about = "The identifier a module table writes for a name.")]
    Identifier {
        #[command(subcommand)]
        action: IdentifierAction,
    },
}

impl Layered for CommandLine {
    type Layer = PresentationLayer;
}

impl CommandLine {
    pub(crate) fn try_act(
        &self,
        output: &mut impl Write,
        errors: &mut impl Write,
    ) -> io::Result<Outcome> {
        let sony = match Sony::try_from(()) {
            Ok(sony) => sony,
            Err(error) => {
                writeln!(errors, "{error}")?;
                return Ok(Outcome::Error);
            }
        };
        match self {
            Self::Name { action } => match action {
                NameAction::Hash(hash) => Self::try_write(hash, &sony, output, errors),
            },
            Self::Identifier { action } => match action {
                IdentifierAction::Resolve(resolve) => {
                    Self::try_write(resolve, &sony, output, errors)
                }
            },
        }
    }

    fn try_write(
        action: &impl Action,
        sony: &Sony,
        output: &mut impl Write,
        errors: &mut impl Write,
    ) -> io::Result<Outcome> {
        match action.try_act(sony) {
            Ok(report) => {
                writeln!(output, "{report}")?;
                Ok(Outcome::Done)
            }
            Err(error) => {
                writeln!(errors, "{error}")?;
                Ok(Outcome::Error)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_command_line_without_a_concept_is_no_command_line() {
        let words = ["sony"];

        let line = CommandLine::try_parse_from(words);

        assert!(line.is_err());
    }

    #[test]
    fn a_concept_without_an_action_is_no_command_line() {
        let words = ["sony", "name"];

        let line = CommandLine::try_parse_from(words);

        assert!(line.is_err());
    }

    #[test]
    fn a_command_line_of_words_yields_the_concept_it_names() {
        let words = ["sony", "name", "hash", "cellHttpUtilParseUri"];

        let line = CommandLine::try_parse_from(words).unwrap();

        let named_file = matches!(line, CommandLine::Name { .. });
        assert!(named_file);
    }

    #[test]
    fn the_identifier_of_a_name_goes_on_the_output_stream() {
        let words = ["sony", "name", "hash", "cellHttpUtilParseUri"];
        let line = CommandLine::try_parse_from(words).unwrap();
        let mut output = Vec::new();
        let mut errors = Vec::new();

        let outcome = line.try_act(&mut output, &mut errors).unwrap();

        let output_text = String::from_utf8(output).unwrap();
        assert_eq!(outcome, Outcome::Done);
        assert!(errors.is_empty());
        assert_eq!(
            output_text,
            "[SONY] cellHttpUtilParseUri yields 32FAAF58.\n"
        );
    }

    #[test]
    fn the_name_of_an_identifier_goes_on_the_output_stream() {
        let words = ["sony", "identifier", "resolve", "9647570B"];
        let line = CommandLine::try_parse_from(words).unwrap();
        let mut output = Vec::new();
        let mut errors = Vec::new();

        let outcome = line.try_act(&mut output, &mut errors).unwrap();

        let output_text = String::from_utf8(output).unwrap();
        assert_eq!(outcome, Outcome::Done);
        assert!(errors.is_empty());
        assert_eq!(
            output_text,
            "[SONY] 9647570B is the identifier of sendto.\n"
        );
    }

    #[test]
    fn an_identifier_no_name_yields_is_named_on_the_error_stream() {
        let words = ["sony", "identifier", "resolve", "DEADBEEF"];
        let line = CommandLine::try_parse_from(words).unwrap();
        let mut output = Vec::new();
        let mut errors = Vec::new();

        let outcome = line.try_act(&mut output, &mut errors).unwrap();

        let error_text = String::from_utf8(errors).unwrap();
        assert_eq!(outcome, Outcome::Error);
        assert!(output.is_empty());
        assert!(error_text.contains("DEADBEEF"));
    }

    #[test]
    fn a_writing_that_is_no_name_is_named_on_the_error_stream() {
        let words = ["sony", "name", "hash", "cell Http"];
        let line = CommandLine::try_parse_from(words).unwrap();
        let mut output = Vec::new();
        let mut errors = Vec::new();

        let outcome = line.try_act(&mut output, &mut errors).unwrap();

        let error_text = String::from_utf8(errors).unwrap();
        assert_eq!(outcome, Outcome::Error);
        assert!(output.is_empty());
        assert_eq!(error_text, "[SONY] cell Http is no name.\n");
    }
}
