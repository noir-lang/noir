use crate::circuit_builder::CircuitBuilder;
use crate::composer_builder::ComposerBuilder;
use crate::file_writer::BBFiles;
use crate::flavor_builder::FlavorBuilder;
use crate::lookup_builder::{
    get_counts_from_lookups, get_inverses_from_lookups, Lookup, LookupBuilder,
};
use crate::permutation_builder::{get_inverses_from_permutations, Permutation, PermutationBuilder};
use crate::prover_builder::ProverBuilder;
use crate::relation_builder::{get_shifted_polys, RelationBuilder};
use crate::utils::{flatten, sanitize_name, snake_case, sort_cols};
use crate::verifier_builder::VerifierBuilder;

use dialoguer::Confirm;
use itertools::Itertools;
use powdr_ast::analyzed::Analyzed;
use powdr_number::FieldElement;

/// All of the combinations of columns that are used in a bberg flavor file
struct ColumnGroups {
    /// fixed or constant columns in pil -> will be found in vk
    fixed: Vec<String>,
    /// witness or commit columns in pil -> will be found in proof
    witness: Vec<String>,
    /// witness or commit columns in pil, with out the inverse columns
    witnesses_without_inverses: Vec<String>,
    /// fixed + witness columns without lookup inverses
    all_cols_without_inverses: Vec<String>,
    /// fixed + witness columns with lookup inverses
    all_cols: Vec<String>,
    /// Columns that will not be shifted
    unshifted: Vec<String>,
    /// Columns that will be shifted
    to_be_shifted: Vec<String>,
    /// The shifts of the columns that will be shifted
    shifted: Vec<String>,
    /// fixed + witness + shifted
    all_cols_with_shifts: Vec<String>,
    /// Inverses from lookups and permuations
    inverses: Vec<String>,
    /// Public inputs (in source order)
    public_inputs: Vec<(usize, String)>,
}

/// Analyzed to cpp
///
/// Converts an analyzed pil AST into a set of cpp files that can be used to generate a proof
pub fn analyzed_to_cpp<F: FieldElement>(analyzed: &Analyzed<F>, vm_name: &str, delete_dir: bool) {
    let mut bb_files = BBFiles::default(&snake_case(&vm_name));

    // Remove the generated directory if it exists.
    // Pass `-y` as parameter if you want to skip the confirmation prompt.
    let confirmation = delete_dir
        || Confirm::new()
            .with_prompt(format!("Going to remove: {}. OK?", bb_files.base_dir))
            .default(true)
            .interact()
            .unwrap();
    if confirmation {
        println!("Removing generated directory: {}", bb_files.base_dir);
        bb_files.remove_generated_dir();
    }

    // ----------------------- Handle Standard Relation Identities -----------------------
    let relations = bb_files.create_relations(vm_name, analyzed);

    // ----------------------- Handle Lookup / Permutation Relation Identities -----------------------
    let permutations = bb_files.create_permutation_files(analyzed);
    let lookups = bb_files.create_lookup_files(analyzed);
    let lookup_and_permutations_names = sort_cols(&flatten(&[
        permutations.iter().map(|p| p.name.clone()).collect_vec(),
        lookups.iter().map(|l| l.name.clone()).collect_vec(),
    ]));

    // Collect all column names
    let ColumnGroups {
        fixed,
        witness,
        witnesses_without_inverses,
        all_cols,
        all_cols_without_inverses,
        unshifted: _unshifted,
        to_be_shifted,
        shifted,
        all_cols_with_shifts,
        inverses,
        public_inputs,
    } = get_all_col_names(analyzed, &permutations, &lookups);

    // ----------------------- Create the full row files -----------------------
    bb_files.create_full_row_hpp(vm_name, &all_cols);
    bb_files.create_full_row_cpp(vm_name, &all_cols);

    // ----------------------- Create the circuit builder files -----------------------
    bb_files.create_circuit_builder_hpp(vm_name);
    bb_files.create_circuit_builder_cpp(vm_name, &all_cols_without_inverses);

    // ----------------------- Create the flavor files -----------------------
    bb_files.create_flavor_hpp(
        vm_name,
        &relations,
        &lookup_and_permutations_names,
        &inverses,
        &fixed,
        &witness,
        &witnesses_without_inverses,
        &all_cols,
        &to_be_shifted,
        &shifted,
        &all_cols_with_shifts,
    );

    bb_files.create_flavor_cpp(
        vm_name,
        &relations,
        &inverses,
        &fixed,
        &witness,
        &witnesses_without_inverses,
        &all_cols,
        &to_be_shifted,
        &shifted,
        &all_cols_with_shifts,
    );

    bb_files.create_flavor_settings_hpp(vm_name);

    // ----------------------- Create the composer files -----------------------
    bb_files.create_composer_cpp(vm_name);
    bb_files.create_composer_hpp(vm_name);

    // ----------------------- Create the Verifier files -----------------------
    bb_files.create_verifier_cpp(vm_name, &public_inputs);
    bb_files.create_verifier_hpp(vm_name);

    // ----------------------- Create the Prover files -----------------------
    bb_files.create_prover_cpp(vm_name);
    bb_files.create_prover_hpp(vm_name);

    println!("Done with generation.");
}

fn get_all_col_names<F: FieldElement>(
    analyzed: &Analyzed<F>,
    permutations: &[Permutation],
    lookups: &[Lookup],
) -> ColumnGroups {
    let constant = sort_cols(
        &analyzed
            .constant_polys_in_source_order()
            .iter()
            .map(|(sym, _)| sym.absolute_name.clone())
            .map(|n| sanitize_name(&n))
            .collect_vec(),
    );
    let committed = sort_cols(
        &analyzed
            .committed_polys_in_source_order()
            .iter()
            .map(|(sym, _)| sym.absolute_name.clone())
            .map(|n| sanitize_name(&n))
            .collect_vec(),
    );
    let public = analyzed
        .public_polys_in_source_order()
        .iter()
        .map(|(sym, _)| sym.absolute_name.clone())
        .map(|n| sanitize_name(&n))
        .collect_vec();
    let to_be_shifted = sort_cols(
        &get_shifted_polys(
            analyzed
                .identities_with_inlined_intermediate_polynomials()
                .iter()
                .map(|i| i.left.selector.clone().unwrap())
                .collect_vec(),
        )
        .iter()
        .map(|n| sanitize_name(&n))
        .collect_vec(),
    );
    let shifted = to_be_shifted
        .iter()
        .map(|n| format!("{}_shift", n))
        .collect_vec();

    let inverses = flatten(&[
        get_inverses_from_permutations(permutations),
        get_inverses_from_lookups(lookups),
    ]);
    let lookup_counts = get_counts_from_lookups(lookups);

    let witnesses_without_inverses =
        flatten(&[public.clone(), committed.clone(), lookup_counts.clone()]);
    let witnesses_with_inverses = flatten(&[
        public.clone(),
        committed.clone(),
        inverses.clone(),
        lookup_counts,
    ]);

    // Group columns by properties
    let all_cols_without_inverses =
        flatten(&[constant.clone(), witnesses_without_inverses.clone()]);
    let all_cols = flatten(&[constant.clone(), witnesses_with_inverses.clone()]);
    let unshifted = flatten(&[constant.clone(), witnesses_with_inverses.clone()])
        .into_iter()
        .filter(|name| !shifted.contains(name))
        .collect_vec();
    let all_cols_with_shifts = flatten(&[
        constant.clone(),
        witnesses_with_inverses.clone(),
        shifted.clone(),
    ]);

    ColumnGroups {
        fixed: constant,
        witness: witnesses_with_inverses,
        all_cols_without_inverses: all_cols_without_inverses,
        witnesses_without_inverses: witnesses_without_inverses,
        all_cols: all_cols,
        unshifted: unshifted,
        to_be_shifted: to_be_shifted,
        shifted: shifted,
        all_cols_with_shifts: all_cols_with_shifts,
        inverses: inverses,
        public_inputs: public.iter().cloned().enumerate().collect_vec(),
    }
}
