//! Answering the program's oracle calls.
//!
//! An oracle call suspends execution and asks a host process a question. The honest run needs a
//! real host, but the search re-executes the program once per candidate, and pointing thousands of
//! re-runs at that host is both slow and wrong: a host like Aztec's PXE has state, so a repeated
//! call can answer differently, and a candidate would then differ from the honest run for reasons
//! unrelated to the mutation being tested.
//!
//! So the host is asked once. The honest run's calls and answers are recorded, and every candidate
//! is answered from that recording.

use acir::{FieldElement, brillig::ForeignCallResult};
use acvm::pwg::ForeignCallWaitInfo;
use nargo::foreign_calls::{ForeignCallError, ForeignCallExecutor};
use serde::{Deserialize, Serialize};

/// One question and its answer, as `LoggingForeignCallExecutor` writes them.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LoggedCall {
    pub call: ForeignCallWaitInfo<FieldElement>,
    pub result: ForeignCallResult<FieldElement>,
}

/// Read the JSON-lines log a `LoggingForeignCallExecutor` produced.
pub fn parse_transcript(log: &[u8]) -> Result<Vec<LoggedCall>, String> {
    std::str::from_utf8(log)
        .map_err(|error| error.to_string())?
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| serde_json::from_str(line).map_err(|error| error.to_string()))
        .collect()
}

/// A candidate asking something the honest run did not ask.
///
/// The recorded answer is handed back anyway, because refusing would reject the candidate for the
/// wrong reason: a prover is free to ask a host whatever it likes, and what the search is testing
/// is whether the *constraints* still hold. The divergence is worth reporting on its own, though —
/// it means the same proof goes with different traffic to the outside world.
#[derive(Clone, Debug)]
pub struct Divergence {
    pub function: String,
}

/// Answers oracle calls from a recording of the honest run.
pub struct Replay {
    entries: Vec<LoggedCall>,
    position: usize,
    divergences: Vec<Divergence>,
}

impl Replay {
    pub fn new(entries: Vec<LoggedCall>) -> Self {
        Self { entries, position: 0, divergences: Vec::new() }
    }

    pub fn divergences(&self) -> &[Divergence] {
        &self.divergences
    }
}

impl ForeignCallExecutor<FieldElement> for Replay {
    fn execute(
        &mut self,
        foreign_call: &ForeignCallWaitInfo<FieldElement>,
    ) -> Result<ForeignCallResult<FieldElement>, ForeignCallError> {
        // Match by name rather than by position. A mutated witness can change how many times a
        // loop runs, so the calls are not guaranteed to line up one for one, and a strict
        // positional match would reject candidates that a prover could perfectly well produce.
        let offset = self.entries[self.position..]
            .iter()
            .position(|entry| entry.call.function == foreign_call.function)
            .ok_or_else(|| {
                ForeignCallError::TranscriptError(format!(
                    "'{}' was not called at this point during the honest run, so there is no \
                     recorded answer for it",
                    foreign_call.function
                ))
            })?;

        let index = self.position + offset;
        if self.entries[index].call.inputs != foreign_call.inputs {
            self.divergences.push(Divergence { function: foreign_call.function.clone() });
        }
        self.position = index + 1;

        Ok(self.entries[index].result.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use acvm::brillig_vm::brillig::ForeignCallParam;

    fn call(function: &str, input: u128) -> ForeignCallWaitInfo<FieldElement> {
        ForeignCallWaitInfo {
            function: function.to_string(),
            inputs: vec![ForeignCallParam::Single(FieldElement::from(input))],
        }
    }

    fn logged(function: &str, input: u128, answer: u128) -> LoggedCall {
        LoggedCall {
            call: call(function, input),
            result: ForeignCallResult { values: vec![ForeignCallParam::Single(answer.into())] },
        }
    }

    #[test]
    fn replays_the_recorded_answer() {
        let mut replay = Replay::new(vec![logged("getNote", 1, 42)]);

        let result = replay.execute(&call("getNote", 1)).expect("should answer from the log");

        assert_eq!(result.values, vec![ForeignCallParam::Single(FieldElement::from(42u128))]);
        assert!(replay.divergences().is_empty());
    }

    #[test]
    fn answers_but_reports_a_call_whose_arguments_moved() {
        let mut replay = Replay::new(vec![logged("getNote", 1, 42)]);

        let result = replay.execute(&call("getNote", 9)).expect("should still answer");

        assert_eq!(result.values, vec![ForeignCallParam::Single(FieldElement::from(42u128))]);
        assert_eq!(replay.divergences().len(), 1);
        assert_eq!(replay.divergences()[0].function, "getNote");
    }

    #[test]
    fn skips_ahead_to_the_matching_call() {
        // A mutation can change how often a loop runs, so calls need not line up one for one.
        let mut replay = Replay::new(vec![logged("print", 0, 0), logged("getNote", 1, 42)]);

        let result = replay.execute(&call("getNote", 1)).expect("should find the later entry");

        assert_eq!(result.values, vec![ForeignCallParam::Single(FieldElement::from(42u128))]);
    }

    #[test]
    fn refuses_a_call_the_honest_run_never_made() {
        let mut replay = Replay::new(vec![logged("getNote", 1, 42)]);

        assert!(replay.execute(&call("getSecret", 1)).is_err());
    }
}
