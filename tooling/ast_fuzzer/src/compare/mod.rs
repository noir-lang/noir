use std::fmt::{Debug, Display};

use arbitrary::{Arbitrary, Unstructured};
use color_eyre::eyre::{self, bail};
use noirc_evaluator::ssa::SsaEvaluatorOptions;
use noirc_frontend::monomorphization::ast::Program;

mod compiled;
mod comptime;
mod interpreted;

pub use compiled::{
    CompareArtifact, CompareCompiled, CompareCompiledResult, CompareMorph, ComparePipelines,
};
pub use comptime::CompareComptime;
pub use interpreted::{CompareInterpreted, CompareInterpretedResult, ComparePass};

#[derive(Clone, Debug, PartialEq)]
pub struct ExecOutput<T> {
    pub return_value: Option<T>,
    pub print_output: String,
}

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

/// Help ignore errors we find equivalent between executions.
pub trait CompareError: Display + Debug {
    fn equivalent(e1: &Self, e2: &Self) -> bool;
}

/// Possible outcomes of the differential execution of two equivalent programs.
///
/// Use [CompareResult::return_value_or_err] to do the final comparison between
/// the execution result.
pub enum CompareResult<T, E> {
    BothFailed(E, E),
    LeftFailed(E, ExecOutput<T>),
    RightFailed(ExecOutput<T>, E),
    BothPassed(ExecOutput<T>, ExecOutput<T>),
}

impl<T, E> CompareResult<T, E>
where
    E: CompareError,
    T: PartialEq + Debug,
{
    /// Check that two programs agree on a return value.
    ///
    /// Returns an error if anything is different.
    pub fn return_value_or_err(&self) -> eyre::Result<Option<&T>> {
        match self {
            CompareResult::BothFailed(e1, e2) => {
                if CompareError::equivalent(e1, e2) {
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
}
