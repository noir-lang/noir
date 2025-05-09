use acir::{FieldElement, native_types::WitnessStack};
use acvm::pwg::{OpcodeResolutionError, ResolvedAssertionPayload};
use arbitrary::{Arbitrary, Unstructured};
use color_eyre::eyre::{self, WrapErr, bail};
use nargo::{NargoError, errors::ExecutionError};
use noirc_abi::{Abi, input_parser::InputValue};
use noirc_evaluator::ssa::SsaEvaluatorOptions;
use noirc_frontend::monomorphization::ast::Program;

mod compiled;
mod comptime;
mod interpreted;

pub use compiled::{CompareArtifact, CompareCompiled, CompareMorph, ComparePipelines};
pub use comptime::CompareComptime;
pub use interpreted::{CompareInterpreted, ComparePass};

#[derive(Clone, Debug, PartialEq)]
pub struct ExecOutput {
    pub return_value: Option<InputValue>,
    pub print_output: String,
}

type ExecResult = (Result<WitnessStack<FieldElement>, NargoError<FieldElement>>, String);

/// Help iterate over the program(s) in the comparable artifact.
pub trait HasPrograms {
    fn programs(&self) -> Vec<&Program>;
}

/// Subset of [SsaEvaluatorOptions] that we want to vary.
///
/// It exists to reduce noise in the printed results, compared to showing the full `SsaEvaluatorOptions`.
#[derive(Debug, Clone, Default)]
pub struct CompareOptions {
    pub inliner_aggressiveness: i64,
}

impl Arbitrary<'_> for CompareOptions {
    fn arbitrary(u: &mut Unstructured<'_>) -> arbitrary::Result<Self> {
        Ok(Self { inliner_aggressiveness: *u.choose(&[i64::MIN, 0, i64::MAX])? })
    }
}

impl CompareOptions {
    /// Copy fields into an [SsaEvaluatorOptions] instance.
    pub fn onto(&self, mut options: SsaEvaluatorOptions) -> SsaEvaluatorOptions {
        options.inliner_aggressiveness = self.inliner_aggressiveness;
        options
    }
}

/// Possible outcomes of the differential execution of two equivalent programs.
///
/// Use [CompareResult::return_value_or_err] to do the final comparison between
/// the execution result.
pub enum CompareResult {
    BothFailed(NargoError<FieldElement>, NargoError<FieldElement>),
    LeftFailed(NargoError<FieldElement>, ExecOutput),
    RightFailed(ExecOutput, NargoError<FieldElement>),
    BothPassed(ExecOutput, ExecOutput),
}

impl CompareResult {
    fn new(
        abi: &Abi,
        (res1, print1): ExecResult,
        (res2, print2): ExecResult,
    ) -> eyre::Result<Self> {
        let decode = |ws: WitnessStack<FieldElement>| -> eyre::Result<Option<InputValue>> {
            let wm = &ws.peek().expect("there should be a main witness").witness;
            let (_, r) = abi.decode(wm).wrap_err("abi::decode")?;
            Ok(r)
        };

        match (res1, res2) {
            (Err(e1), Err(e2)) => Ok(CompareResult::BothFailed(e1, e2)),
            (Err(e1), Ok(ws2)) => Ok(CompareResult::LeftFailed(
                e1,
                ExecOutput { return_value: decode(ws2)?, print_output: print2 },
            )),
            (Ok(ws1), Err(e2)) => Ok(CompareResult::RightFailed(
                ExecOutput { return_value: decode(ws1)?, print_output: print1 },
                e2,
            )),
            (Ok(ws1), Ok(ws2)) => {
                let o1 = ExecOutput { return_value: decode(ws1)?, print_output: print1 };
                let o2 = ExecOutput { return_value: decode(ws2)?, print_output: print2 };
                Ok(CompareResult::BothPassed(o1, o2))
            }
        }
    }

    /// Check that the programs agree on a return value.
    ///
    /// Returns an error if anything is different.
    pub fn return_value_or_err(&self) -> eyre::Result<Option<&InputValue>> {
        match self {
            CompareResult::BothFailed(e1, e2) => {
                if Self::errors_match(e1, e2) {
                    // Both programs failed the same way.
                    Ok(None)
                } else {
                    bail!("both programs failed: {e1} vs {e2}\n{e1:?}\n{e2:?}")
                }
            }
            CompareResult::LeftFailed(e, _) => {
                bail!("first program failed: {e}\n{e:?}")
            }
            CompareResult::RightFailed(_, e) => {
                bail!("second program failed: {e}\n{e:?}")
            }
            CompareResult::BothPassed(o1, o2) => {
                if o1.return_value != o2.return_value {
                    bail!(
                        "programs disagree on return value:\n{:?}\n!=\n{:?}",
                        o1.return_value,
                        o2.return_value
                    )
                } else if o1.print_output != o2.print_output {
                    bail!(
                        "programs disagree on printed output:\n---\n{}\n\n---\n{}\n",
                        o1.print_output,
                        o2.print_output
                    )
                } else {
                    Ok(o1.return_value.as_ref())
                }
            }
        }
    }

    /// Check whether two errors can be considered equivalent.
    fn errors_match(e1: &NargoError<FieldElement>, e2: &NargoError<FieldElement>) -> bool {
        use ExecutionError::*;

        // For now consider non-execution errors as failures we need to investigate.
        let NargoError::ExecutionError(ee1) = e1 else {
            return false;
        };
        let NargoError::ExecutionError(ee2) = e2 else {
            return false;
        };

        match (ee1, ee2) {
            (AssertionFailed(p1, _, _), AssertionFailed(p2, _, _)) => p1 == p2,
            (SolvingError(s1, _), SolvingError(s2, _)) => format!("{s1}") == format!("{s2}"),
            (SolvingError(s, _), AssertionFailed(p, _, _))
            | (AssertionFailed(p, _, _), SolvingError(s, _)) => match (s, p) {
                (
                    OpcodeResolutionError::UnsatisfiedConstrain { .. },
                    ResolvedAssertionPayload::String(s),
                ) => s == "Attempted to divide by zero",
                _ => false,
            },
        }
    }
}
