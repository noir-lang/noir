//! A search for second witnesses of a compiled Noir program.
//!
//! A circuit should pin down every intermediate value once its inputs are fixed. Where it does not,
//! a prover can satisfy every constraint with a different witness, which is what an underconstrained
//! circuit means in practice. This crate looks for such a witness by overriding the outputs of one
//! Brillig hint call and re-solving the program: if the solver accepts the result, the alternative
//! witness is the proof, and no reasoning about the source is needed to trust it.

pub mod derive;
pub mod directives;
pub mod hints;
pub mod source;
pub mod strategy;

use acir::{
    FieldElement,
    circuit::Program,
    native_types::{Witness, WitnessMap},
};
use bn254_blackbox_solver::Bn254BlackBoxSolver;
use nargo::foreign_calls::{DefaultForeignCallBuilder, layers};
use std::collections::BTreeMap;

use crate::{
    hints::{HintSite, hint_sites, program_with_override},
    strategy::candidates,
};

/// What a second witness means.
///
/// Two questions decide it. Does anything the verifier sees change — that is, a return value? And
/// if not, does the rest of the witness move with the mutation, or is the free value one that
/// nothing reads?
///
/// The second question has no mechanical answer for whether it is a bug. A circuit whose whole
/// statement is "I know a value with property P" has no return value to change, so a free witness
/// there is the break itself; a circuit that returns a result and happens to leave a scratch value
/// free is fine. Which one a program is depends on what it claims to prove, so those findings are
/// reported for a human rather than graded.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    /// A compiler-inserted hint's outputs are not pinned down, and a return value follows them.
    /// The circuit the compiler emitted is too weak.
    CompilerBug,
    /// The program returns a value an `unconstrained fn` produced without constraining it. The
    /// circuit matches the source; the source is what trusts an unchecked value.
    ProgramUnderconstrained,
    /// No return value moves, but the rest of the witness does: the proof does not pin down the
    /// values the program computed. Whether that is exploitable depends on what the circuit is
    /// meant to prove.
    WitnessNotUnique,
    /// Only the hint's own outputs move, and nothing else in the witness follows. The value is not
    /// read by anything — the inverse hint of an `x != 0` check when `x` is zero, or a call under
    /// a false predicate.
    Inert,
}

impl Severity {
    pub fn label(self) -> &'static str {
        match self {
            Severity::CompilerBug => "HIGH",
            Severity::ProgramUnderconstrained => "PROGRAM",
            Severity::WitnessNotUnique => "WITNESS",
            Severity::Inert => "INERT",
        }
    }
}

/// A witness that differs from the honest one and still satisfies every constraint.
#[derive(Clone, Debug)]
pub struct Finding {
    pub site: HintSite,
    pub strategy: String,
    /// Hint outputs that differ, as (witness, honest, second witness).
    pub changed_outputs: Vec<(Witness, FieldElement, FieldElement)>,
    /// Whether a return value changes, which is the only difference a verifier can see.
    pub changes_return: bool,
    /// How many witnesses other than this call's own outputs take a different value. A free value
    /// that nothing reads moves nothing; one the program computes with drags the rest along.
    pub blast_radius: usize,
    pub witness: WitnessMap<FieldElement>,
}

impl Finding {
    pub fn severity(&self) -> Severity {
        match (self.changes_return, self.site.kind.is_directive()) {
            (true, true) => Severity::CompilerBug,
            (true, false) => Severity::ProgramUnderconstrained,
            (false, _) if self.blast_radius > 0 => Severity::WitnessNotUnique,
            (false, _) => Severity::Inert,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Report {
    pub findings: Vec<Finding>,
    pub sites: usize,
    pub candidates_tried: usize,
    /// A circuit with no return values gives a verifier nothing to compare, so no finding in it can
    /// be graded by its effect on the output.
    pub has_return_values: bool,
}

impl Report {
    pub fn compiler_bugs(&self) -> impl Iterator<Item = &Finding> {
        self.findings.iter().filter(|finding| finding.severity() == Severity::CompilerBug)
    }
}

fn solve(
    program: &Program<FieldElement>,
    initial_witness: WitnessMap<FieldElement>,
) -> Option<WitnessMap<FieldElement>> {
    let mut foreign_call_executor = DefaultForeignCallBuilder {
        output: std::io::sink(),
        enable_mocks: false,
        resolver_url: None,
        root_path: None,
        package_name: None,
    }
    .build_with_base(layers::Unhandled);

    let mut stack = nargo::ops::execute_program(
        program,
        initial_witness,
        &Bn254BlackBoxSolver,
        &mut foreign_call_executor,
    )
    .ok()?;
    stack.pop().map(|item| item.witness)
}

/// Search `program` for a second witness of `initial_witness`.
///
/// `initial_witness` holds the inputs only; the honest run fills in the rest. Every candidate is
/// accepted or rejected by re-solving the whole program, so a reported finding is a witness the
/// solver itself accepted rather than something inferred from the constraints.
pub fn search(
    program: &Program<FieldElement>,
    initial_witness: WitnessMap<FieldElement>,
    limit: usize,
) -> Result<Report, String> {
    let honest = solve(program, initial_witness.clone())
        .ok_or_else(|| "honest execution failed".to_string())?;

    let sites = hint_sites(program, 0, &honest);
    let known: BTreeMap<Witness, FieldElement> =
        honest.clone().into_iter().collect::<BTreeMap<_, _>>();
    let candidates = candidates(&program.functions[0], &sites, &known, limit);
    let candidates_tried = candidates.len();

    let return_witnesses = &program.functions[0].return_values.0;
    let mut findings: Vec<Finding> = Vec::new();

    for candidate in candidates {
        let site = &sites[candidate.site_index];
        let modified = program_with_override(program, site, &candidate.values);
        let Some(witness) = solve(&modified, initial_witness.clone()) else {
            continue;
        };
        if witness == honest {
            continue;
        }

        let changed_outputs = site
            .outputs
            .iter()
            .enumerate()
            .filter_map(|(index, output)| {
                let value = *witness.get(output)?;
                (value != site.honest[index]).then_some((*output, site.honest[index], value))
            })
            .collect::<Vec<_>>();
        if changed_outputs.is_empty() {
            continue;
        }

        let changes_return = return_witnesses
            .iter()
            .any(|witness_index| witness.get(witness_index) != honest.get(witness_index));

        let blast_radius = witness
            .clone()
            .into_iter()
            .filter(|(index, value)| {
                !site.outputs.contains(index) && honest.get(index) != Some(value)
            })
            .count();

        let finding = Finding {
            site: site.clone(),
            strategy: candidate.strategy,
            changed_outputs,
            changes_return,
            blast_radius,
            witness,
        };

        // One report per site and grade: a site that is free at all is usually free in many ways,
        // and listing every alias buries the fact that there are distinct problems.
        let already_reported = findings.iter().any(|reported| {
            reported.site.opcode_index == finding.site.opcode_index
                && reported.severity() == finding.severity()
        });
        if !already_reported {
            findings.push(finding);
        }
    }

    findings.sort_by_key(|finding| (finding.severity(), finding.site.opcode_index));
    Ok(Report {
        findings,
        sites: sites.len(),
        candidates_tried,
        has_return_values: !return_witnesses.is_empty(),
    })
}
