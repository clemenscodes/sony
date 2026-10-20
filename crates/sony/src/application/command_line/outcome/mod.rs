use ddd::{Layered, PresentationLayer};
use std::process::ExitCode;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub(crate) enum Outcome {
    Done,
    Error,
}

impl Layered for Outcome {
    type Layer = PresentationLayer;
}

impl From<Outcome> for ExitCode {
    fn from(outcome: Outcome) -> Self {
        match outcome {
            Outcome::Done => Self::SUCCESS,
            Outcome::Error => Self::FAILURE,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_answer_that_was_given_ends_the_process_in_success() {
        let outcome = Outcome::Done;

        let code = ExitCode::from(outcome);

        let success = ExitCode::SUCCESS;
        assert_eq!(format!("{code:?}"), format!("{success:?}"));
    }

    #[test]
    fn an_error_ends_the_process_the_way_rust_ends_one_that_went_wrong() {
        let outcome = Outcome::Error;

        let code = ExitCode::from(outcome);

        let expectation = ExitCode::FAILURE;
        assert_eq!(format!("{code:?}"), format!("{expectation:?}"));
    }
}
