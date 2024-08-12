use crate::file_writer::BBFiles;
use itertools::Itertools;
use powdr_ast::{
    analyzed::{AlgebraicExpression, Analyzed, IdentityKind},
    parsed::SelectedExpressions,
};
use powdr_number::FieldElement;

use handlebars::Handlebars;
use serde_json::{json, Value as Json};

use crate::utils::sanitize_name;

#[derive(Debug)]
/// Permutation
///
/// Contains the information required to produce a permutation relation
pub struct Permutation {
    /// The name of the lookup
    pub name: String,
    /// The inverse column name
    pub inverse: String,
    /// -> PermSide - the left side of the permutation
    pub left: PermutationSide,
    /// -> PermSide - the right side of the permutation
    pub right: PermutationSide,
}

#[derive(Debug)]
/// PermSide
///
/// One side of a two sided permutation relationship
pub struct PermutationSide {
    /// -> Option<String> - the selector for the permutation ( on / off toggle )
    selector: Option<String>,
    /// The columns involved in this side of the permutation
    cols: Vec<String>,
}

pub trait PermutationBuilder {
    /// Takes in an AST and works out what permutation relations are needed
    /// Note: returns the name of the inverse columns, such that they can be added to he prover in subsequent steps
    fn create_permutation_files<F: FieldElement>(&self, analyzed: &Analyzed<F>)
        -> Vec<Permutation>;
}

impl PermutationBuilder for BBFiles {
    fn create_permutation_files<F: FieldElement>(
        &self,
        analyzed: &Analyzed<F>,
    ) -> Vec<Permutation> {
        let permutations = analyzed
            .identities
            .iter()
            .filter(|identity| matches!(identity.kind, IdentityKind::Permutation))
            .map(|perm| {
                let name = perm
                    .attribute
                    .clone()
                    .expect("Permutation name must be provided using attribute syntax")
                    .to_lowercase();
                Permutation {
                    name: name.clone(),
                    inverse: format!("{}_inv", &name),
                    left: get_perm_side(&perm.left),
                    right: get_perm_side(&perm.right),
                }
            })
            .collect_vec();

        let mut handlebars = Handlebars::new();

        handlebars
            .register_template_string(
                "permutation.hpp",
                std::str::from_utf8(include_bytes!("../templates/permutation.hpp.hbs")).unwrap(),
            )
            .unwrap();

        for permutation in permutations.iter() {
            let data = create_permutation_settings_data(permutation);
            let perm_settings = handlebars.render("permutation.hpp", &data).unwrap();

            let file_name = format!("{}.hpp", permutation.name);
            self.write_file(Some(&self.relations), &file_name, &perm_settings);
        }

        permutations
    }
}

/// The attributes of a permutation contain the name of the inverse, we collect all of these to create the inverse column
pub fn get_inverses_from_permutations(permutations: &[Permutation]) -> Vec<String> {
    permutations
        .iter()
        .map(|perm| perm.inverse.clone())
        .collect()
}

fn create_permutation_settings_data(permutation: &Permutation) -> Json {
    let columns_per_set = permutation.left.cols.len();

    // This also will need to work for both sides of this !
    let lhs_selector = permutation
        .left
        .selector
        .clone()
        .expect("At least one selector must be provided");
    // If a rhs selector is not present, then we use the rhs selector -- TODO(md): maybe we want the default to be always on?
    let rhs_selector = permutation
        .right
        .selector
        .clone()
        .unwrap_or(lhs_selector.clone());

    let lhs_cols = permutation.left.cols.clone();
    let rhs_cols = permutation.right.cols.clone();

    // 0.                       The polynomial containing the inverse products -> taken from the attributes
    // 1.                       The polynomial enabling the relation (the selector)
    // 2.                       lhs selector
    // 3.                       rhs selector
    // 4.. + columns per set.   lhs cols
    // 4 + columns per set.. .  rhs cols
    let mut perm_entities: Vec<String> = [
        permutation.inverse.clone(),
        lhs_selector.clone(),
        lhs_selector.clone(),
        rhs_selector.clone(),
    ]
    .to_vec();

    perm_entities.extend(lhs_cols);
    perm_entities.extend(rhs_cols);

    json!({
        "perm_name": permutation.name,
        "columns_per_set": columns_per_set,
        "lhs_selector": lhs_selector,
        "rhs_selector": rhs_selector,
        "perm_entities": perm_entities,
    })
}

fn get_perm_side<F: FieldElement>(
    def: &SelectedExpressions<AlgebraicExpression<F>>,
) -> PermutationSide {
    let get_name = |expr: &AlgebraicExpression<F>| match expr {
        AlgebraicExpression::Reference(a_ref) => sanitize_name(&a_ref.name),
        _ => panic!("Expected reference"),
    };

    PermutationSide {
        selector: def.selector.as_ref().map(get_name),
        cols: def.expressions.iter().map(get_name).collect_vec(),
    }
}
