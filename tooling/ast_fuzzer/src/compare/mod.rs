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

/// Optional return value with the tracked side effects.
#[derive(Clone, Debug, PartialEq)]
pub struct PassedOutput<T> {
    pub return_value: Option<T>,
    pub print_output: String,
}

/// Error returned from the circuit, with tracked side effects.
///
/// We want to inspect side effects even on failures, so we can treat different failures
/// as equivalent as long as the other side effects are equivalent.
#[derive(Clone, Debug, PartialEq)]
pub struct FailedOutput<E> {
    pub error: E,
    pub print_output: String,
}

/// Possible outcomes of the differential execution of two equivalent programs.
///
/// Use [CompareResult::return_value_or_err] to do the final comparison between
/// the execution result.
pub enum CompareResult<T, E> {
    BothFailed(FailedOutput<E>, FailedOutput<E>),
    LeftFailed(FailedOutput<E>, PassedOutput<T>),
    RightFailed(PassedOutput<T>, FailedOutput<E>),
    BothPassed(PassedOutput<T>, PassedOutput<T>),
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
                if !Comparable::equivalent(&e1.error, &e2.error) {
                    let e1 = &e1.error;
                    let e2 = &e2.error;
                    bail!(
                        "both programs failed in non-equivalent ways:\n{e1}\n!~\n{e2}\n\n{e1:?}\n{e2:?}"
                    );
                } else {
                    let p1 = &e1.print_output;
                    let p2 = &e2.print_output;
                    if p1 != p2 {
                        bail!(
                            "both programs failed, but disagree on printed output:\n---\n{p1}\n--- != ---\n{p2}\n---",
                        );
                    } else {
                        Ok(None)
                    }
                }
            }
            CompareResult::LeftFailed(e, _) => {
                let e = &e.error;
                bail!("first program failed: {e}\n\n{e:?}")
            }
            CompareResult::RightFailed(_, e) => {
                let e = &e.error;
                bail!("second program failed: {e}\n\n{e:?}")
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
                    let p1 = &o1.print_output;
                    let p2 = &o2.print_output;
                    if p1 != p2 {
                        bail!(
                            "both programs passed, but disagree on printed output:\n---\n{p1}\n--- != ---\n{p2}\n---",
                        )
                    }
                    Ok(r1.as_ref())
                }
            },
        }
    }
}

/// We can turn on the logging of artifacts by setting the `RUST_LOG=debug` env var.
///
/// This can help reproducing failures. The functions in this module can help cut back some repetition.
/// Log things as soon as they are available in case the next step fails.
mod logging {
    use noirc_abi::{Abi, InputMap};
    use noirc_evaluator::ssa::{interpreter::value::Value, ssa_gen::Ssa};
    use noirc_frontend::monomorphization::ast::Program;

    use crate::{DisplayAstAsNoir, compare::CompareOptions};

    fn format_msg(msg: &str) -> String {
        if msg.is_empty() { String::new() } else { format!(" ({msg})") }
    }

    pub(super) fn log_program(program: &Program, msg: &str) {
        log::debug!("AST{}:\n{}\n", format_msg(msg), DisplayAstAsNoir(program));
    }

    pub(super) fn log_ssa(ssa: &Ssa, msg: &str) {
        log::debug!("SSA{}:\n{}\n", format_msg(msg), ssa.print_without_locations());
    }

    pub(super) fn log_comptime(src: &str, msg: &str) {
        log::debug!("comptime source{}:\n{}\n", format_msg(msg), src);
    }

    pub(super) fn log_options(options: &CompareOptions, msg: &str) {
        log::debug!("Options{}:\n{:?}\n", format_msg(msg), options);
    }

    pub(super) fn log_abi_inputs(abi: &Abi, input_map: &InputMap) {
        log::debug!(
            "ABI inputs:\n{}\n",
            noirc_abi::input_parser::Format::Toml
                .serialize(input_map, abi)
                .unwrap_or_else(|e| format!("failed to serialize inputs: {e}"))
        );
    }

    pub(super) fn log_ssa_inputs(ssa_args: &[Value]) {
        log::debug!(
            "SSA inputs:\n{}\n",
            ssa_args
                .iter()
                .enumerate()
                .map(|(i, v)| format!("{i}: {v}"))
                .collect::<Vec<_>>()
                .join("\n")
        );
    }
}
