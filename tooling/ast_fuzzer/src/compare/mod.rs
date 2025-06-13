use std::{
    fmt::{Debug, Display},
    ops::Deref,
};

use arbitrary::{Arbitrary, Unstructured};
use color_eyre::eyre::{self, bail};
use noirc_evaluator::ssa::SsaEvaluatorOptions;
use noirc_frontend::{Shared, monomorphization::ast::Program};

mod compiled;
mod comptime;
mod interpreted;

pub use compiled::{
    CompareArtifact, CompareCompiled, CompareCompiledResult, CompareMorph, ComparePipelines,
};
pub use comptime::CompareComptime;
pub use interpreted::{
    CompareInterpreted, CompareInterpretedResult, ComparePass, input_value_to_ssa,
    input_values_to_ssa,
};

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

/// Help ignore results and errors we find equivalent between executions.
pub trait Comparable {
    fn equivalent(a: &Self, b: &Self) -> bool;
}

impl<T: Comparable> Comparable for Vec<T> {
    fn equivalent(a: &Self, b: &Self) -> bool {
        a.len() == b.len() && a.iter().zip(b).all(|(a, b)| Comparable::equivalent(a, b))
    }
}

impl<T: Comparable> Comparable for Option<T> {
    fn equivalent(a: &Self, b: &Self) -> bool {
        match (a, b) {
            (Some(a), Some(b)) => Comparable::equivalent(a, b),
            (None, None) => true,
            _ => false,
        }
    }
}

impl<T: Comparable> Comparable for Shared<T> {
    fn equivalent(a: &Self, b: &Self) -> bool {
        let a = a.borrow();
        let b = b.borrow();
        Comparable::equivalent(a.deref(), b.deref())
    }
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
    E: Comparable + Display + Debug,
    T: Comparable + Debug,
{
    /// Check that two programs agree on a return value.
    ///
    /// Returns an error if anything is different.
    pub fn return_value_or_err(&self) -> eyre::Result<Option<&T>> {
        match self {
            CompareResult::BothFailed(e1, e2) => {
                if Comparable::equivalent(e1, e2) {
                    // Both programs failed the same way.
                    Ok(None)
                } else {
                    bail!("both programs failed:\n{e1}\n!=\n{e2}\n\n{e1:?}\n{e2:?}")
                }
            }
            CompareResult::LeftFailed(e, _) => {
                bail!("first program failed: {e}\n{e:?}")
            }
            CompareResult::RightFailed(_, e) => {
                bail!("second program failed: {e}\n{e:?}")
            }
            CompareResult::BothPassed(o1, o2) => match (&o1.return_value, &o2.return_value) {
                (Some(r1), Some(r2)) if !Comparable::equivalent(r1, r2) => {
                    bail!("programs disagree on return value:\n{r1:?}\n!=\n{r2:?}",)
                }
                (Some(r1), None) => {
                    bail!("only the first program returned a value: {r1:?}",)
                }
                (None, Some(r2)) => {
                    bail!("only the second program returned a value: {r2:?}",)
                }
                (r1, _) => {
                    if o1.print_output != o2.print_output {
                        bail!(
                            "programs disagree on printed output:\n---\n{}\n--- != ---\n{}\n---",
                            o1.print_output,
                            o2.print_output
                        )
                    }
                    Ok(r1.as_ref())
                }
            },
        }
    }
}
