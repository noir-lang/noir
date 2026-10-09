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
pub mod oracles;
pub mod source;
pub mod strategy;

use acir::{
    FieldElement,
    circuit::Program,
    native_types::{Witness, WitnessMap},
};
use bn254_blackbox_solver::Bn254BlackBoxSolver;
use nargo::foreign_calls::{
    DefaultForeignCallBuilder, ForeignCallExecutor, layers, transcript::LoggingForeignCallExecutor,
};
use std::{
    collections::{BTreeMap, HashSet},
    path::PathBuf,
};

use crate::{
    hints::{HintSite, hint_sites, program_with_override},
    oracles::{LoggedCall, Replay, parse_transcript},
    strategy::candidates,
};

/// Where the honest run's oracle calls are answered from.
#[derive(Clone, Debug, Default)]
pub struct OracleConfig {
    /// JSON-RPC host to ask, as `nargo execute --oracle-resolver` takes.
    pub resolver_url: Option<String>,
    pub root_path: Option<PathBuf>,
    pub package_name: Option<String>,
}

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

/// Whether the search ran out of budget before it ran out of candidates.
///
/// A truncated run has ruled nothing out. Reporting it as a clean one is the single most harmful
/// mistake a caller can make with this tool, so the distinction is a value rather than a log line.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RunStatus {
    Complete,
    Truncated,
}

impl RunStatus {
    pub fn label(self) -> &'static str {
        match self {
            RunStatus::Complete => "complete",
            RunStatus::Truncated => "truncated",
        }
    }
}

/// What the search actually did at one hint call site.
///
/// A site with no candidates was never put under any pressure, so silence about it means nothing.
#[derive(Clone, Debug)]
pub struct SiteCoverage {
    pub label: String,
    pub hint: String,
    pub outputs: usize,
    pub candidates_tried: usize,
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
    /// How many oracle calls this witness makes with different arguments than the honest run. The
    /// proof is the same either way, so this is what the outside world would see differently.
    pub oracle_divergences: usize,
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
    /// Oracle calls the honest run made, all of which the search answered from its recording.
    pub oracle_calls: usize,
    /// Whether the candidate budget ran out before the candidates did.
    pub status: RunStatus,
    /// What was tried at each hint call site.
    pub coverage: Vec<SiteCoverage>,
}

impl Report {
    pub fn compiler_bugs(&self) -> impl Iterator<Item = &Finding> {
        self.findings.iter().filter(|finding| finding.severity() == Severity::CompilerBug)
    }
}

fn solve<E: ForeignCallExecutor<FieldElement>>(
    program: &Program<FieldElement>,
    initial_witness: WitnessMap<FieldElement>,
    foreign_calls: &mut E,
) -> Option<WitnessMap<FieldElement>> {
    let mut stack =
        nargo::ops::execute_program(program, initial_witness, &Bn254BlackBoxSolver, foreign_calls)
            .ok()?;
    stack.pop().map(|item| item.witness)
}

/// Execute honestly, recording every oracle call so the search can answer the rest offline.
fn honest_run(
    program: &Program<FieldElement>,
    initial_witness: WitnessMap<FieldElement>,
    oracles: &OracleConfig,
) -> Result<(WitnessMap<FieldElement>, Vec<LoggedCall>), String> {
    let host = DefaultForeignCallBuilder {
        output: std::io::sink(),
        enable_mocks: true,
        resolver_url: oracles.resolver_url.clone(),
        root_path: oracles.root_path.clone(),
        package_name: oracles.package_name.clone(),
    }
    .build_with_base(layers::Unhandled);

    let mut recorder = LoggingForeignCallExecutor::new(host, Vec::new());
    let witness = solve(program, initial_witness, &mut recorder).ok_or_else(|| {
        if oracles.resolver_url.is_some() {
            "honest execution failed; the oracle host may have rejected a call".to_string()
        } else {
            "honest execution failed; if this program calls oracles, pass --oracle-resolver"
                .to_string()
        }
    })?;

    Ok((witness, parse_transcript(&recorder.output)?))
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
    oracles: &OracleConfig,
    only_opcodes: Option<&HashSet<usize>>,
) -> Result<Report, String> {
    let (honest, transcript) = honest_run(program, initial_witness.clone(), oracles)?;

    let mut sites = hint_sites(program, 0, &honest);
    if let Some(only) = only_opcodes {
        sites.retain(|site| only.contains(&site.opcode_index));
        if sites.is_empty() {
            return Err("no hint call site matches the requested target".to_string());
        }
    }
    let known: BTreeMap<Witness, FieldElement> =
        honest.clone().into_iter().collect::<BTreeMap<_, _>>();
    let (candidates, truncated) = candidates(&program.functions[0], &sites, &known, limit);
    let candidates_tried = candidates.len();
    let mut tried_per_site = vec![0usize; sites.len()];

    let return_witnesses = &program.functions[0].return_values.0;
    let mut findings: Vec<Finding> = Vec::new();

    for candidate in candidates {
        let site = &sites[candidate.site_index];
        tried_per_site[candidate.site_index] += 1;
        let modified = program_with_override(program, site, &candidate.values);
        let mut replay = Replay::new(transcript.clone());
        let Some(witness) = solve(&modified, initial_witness.clone(), &mut replay) else {
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
            oracle_divergences: replay.divergences().len(),
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
    let coverage = sites
        .iter()
        .zip(tried_per_site)
        .map(|(site, candidates_tried)| SiteCoverage {
            label: site.label(),
            hint: site.kind.name().to_string(),
            outputs: site.outputs.len(),
            candidates_tried,
        })
        .collect();

    Ok(Report {
        findings,
        sites: sites.len(),
        candidates_tried,
        has_return_values: !return_witnesses.is_empty(),
        oracle_calls: transcript.len(),
        status: if truncated { RunStatus::Truncated } else { RunStatus::Complete },
        coverage,
    })
}
