//! A search for second witnesses of a compiled Noir program.
//!
//! A circuit should pin down every intermediate value once its inputs are fixed. Where it does not,
//! a prover can satisfy every constraint with a different witness, which is what an underconstrained
//! circuit means in practice. This crate looks for such a witness by overriding the outputs of one
//! Brillig hint call and re-solving the program: if the solver accepts the result, the alternative
//! witness is the proof, and no reasoning about the source is needed to trust it.

pub mod derive;
pub mod hints;
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

/// A witness that differs from the honest one and still satisfies every constraint.
#[derive(Clone, Debug)]
pub struct Finding {
    pub site: HintSite,
    pub strategy: String,
    /// Hint outputs that differ, as (witness, honest, second witness).
    pub changed_outputs: Vec<(Witness, FieldElement, FieldElement)>,
    /// Whether a return value changes, which is the difference between an exploitable circuit and
    /// a harmless one: an intermediate nobody reads may legitimately have several values.
    pub changes_return: bool,
    pub witness: WitnessMap<FieldElement>,
}

impl Finding {
    pub fn severity(&self) -> &'static str {
        if self.changes_return { "HIGH" } else { "LOW" }
    }
}

#[derive(Clone, Debug)]
pub struct Report {
    pub findings: Vec<Finding>,
    pub sites: usize,
    pub candidates_tried: usize,
}

impl Report {
    pub fn high(&self) -> impl Iterator<Item = &Finding> {
        self.findings.iter().filter(|finding| finding.changes_return)
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

        // One report per site and severity: a site that is free at all is usually free in many
        // ways, and listing every alias buries the fact that there are two distinct problems.
        let already_reported = findings.iter().any(|finding| {
            finding.site.opcode_index == site.opcode_index
                && finding.changes_return == changes_return
        });
        if already_reported {
            continue;
        }

        findings.push(Finding {
            site: site.clone(),
            strategy: candidate.strategy,
            changed_outputs,
            changes_return,
            witness,
        });
    }

    findings.sort_by_key(|finding| (!finding.changes_return, finding.site.opcode_index));
    Ok(Report { findings, sites: sites.len(), candidates_tried })
}
