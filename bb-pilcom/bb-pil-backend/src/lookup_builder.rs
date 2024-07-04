use crate::{file_writer::BBFiles, utils::snake_case};
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
/// Lookup
///
/// Contains the information required to produce a lookup relation
/// Lookup object and lookup side object are very similar in structure, however they are duplicated for
/// readability.
pub struct Lookup {
    ///  the name given to the inverse helper column
    pub attribute: Option<String>,
    /// The name of the counts polynomial that stores the number of times a lookup is read
    pub counts_poly: String,
    /// the left side of the lookup
    pub left: LookupSide,
    /// the right side of the lookup
    pub right: LookupSide,
}

#[derive(Debug)]
/// LookupSide
///
/// One side of a two sided lookup relationship
pub struct LookupSide {
    /// -> Option<String> - the selector for the lookup ( on / off toggle )
    selector: Option<String>,
    /// The columns involved in this side of the lookup
    cols: Vec<String>,
}

pub trait LookupBuilder {
    /// Takes in an AST and works out what lookup relations are needed
    /// Note: returns the name of the inverse columns, such that they can be added to the prover in subsequent steps
    fn create_lookup_files<F: FieldElement>(
        &self,
        name: &str,
        analyzed: &Analyzed<F>,
    ) -> Vec<Lookup>;
}

impl LookupBuilder for BBFiles {
    fn create_lookup_files<F: FieldElement>(
        &self,
        project_name: &str,
        analyzed: &Analyzed<F>,
    ) -> Vec<Lookup> {
        let lookups = analyzed
            .identities
            .iter()
            .filter(|identity| matches!(identity.kind, IdentityKind::Plookup))
            .map(|lookup| Lookup {
                attribute: lookup.attribute.clone().map(|att| att.to_lowercase()),
                counts_poly: format!(
                    "{}_counts",
                    lookup.attribute.clone().unwrap().to_lowercase()
                ),
                left: get_lookup_side(&lookup.left),
                right: get_lookup_side(&lookup.right),
            })
            .collect_vec();

        let mut handlebars = Handlebars::new();

        handlebars
            .register_template_string(
                "lookup.hpp",
                std::str::from_utf8(include_bytes!("../templates/lookup.hpp.hbs")).unwrap(),
            )
            .unwrap();

        for lookup in lookups.iter() {
            let data = create_lookup_settings_data(lookup);
            let lookup_settings = handlebars.render("lookup.hpp", &data).unwrap();

            let folder = format!("{}/{}", self.rel, &snake_case(project_name));
            let file_name = format!(
                "{}{}",
                lookup.attribute.clone().unwrap_or("NONAME".to_owned()),
                ".hpp".to_owned()
            );
            self.write_file(&folder, &file_name, &lookup_settings);
        }

        lookups
    }
}

/// The attributes of a lookup contain the name of the inverse, we collect all of these to create the inverse column
pub fn get_inverses_from_lookups(lookups: &[Lookup]) -> Vec<String> {
    lookups
        .iter()
        .map(|lookup| lookup.attribute.clone().unwrap())
        .collect()
}

pub fn get_counts_from_lookups(lookups: &[Lookup]) -> Vec<String> {
    lookups
        .iter()
        .map(|lookup| lookup.counts_poly.clone())
        .collect()
}

fn create_lookup_settings_data(lookup: &Lookup) -> Json {
    let columns_per_set = lookup.left.cols.len();
    let lookup_name = lookup
        .attribute
        .clone()
        .expect("Inverse column name must be provided within lookup attribute - #[<here>]");
    let counts_poly_name = lookup.counts_poly.to_owned();

    // NOTE: https://github.com/AztecProtocol/aztec-packages/issues/3879
    // Settings are not flexible enough to combine inverses
    let lhs_selector = lookup
        .left
        .selector
        .clone()
        .expect("Left hand side selector for lookup required");
    let rhs_selector = lookup
        .right
        .selector
        .clone()
        .expect("Right hand side selector for lookup required");
    let lhs_cols = lookup.left.cols.clone();
    let rhs_cols = lookup.right.cols.clone();

    assert!(
        lhs_cols.len() == rhs_cols.len(),
        "Lookup columns lhs must be the same length as rhs"
    );

    // 0.                       The polynomial containing the inverse products -> taken from the attributes
    // 1.                       The polynomial with the counts!
    // 2.                       lhs selector
    // 3.                       rhs selector
    // 4.. + columns per set.   lhs cols
    // 4 + columns per set.. .  rhs cols
    let mut lookup_entities: Vec<String> = [
        lookup_name.clone(),
        counts_poly_name.clone(),
        lhs_selector.clone(),
        rhs_selector.clone(),
    ]
    .to_vec();

    lookup_entities.extend(lhs_cols);
    lookup_entities.extend(rhs_cols);

    // NOTE: these are hardcoded as 1 for now until more optimizations are required
    let read_terms = 1;
    let write_terms = 1;
    let lookup_tuple_size = columns_per_set;

    // NOTE: hardcoded until optimizations required
    let inverse_degree = 4;
    let read_term_degree = 0;
    let write_term_degree = 0;
    let read_term_types = "{0}".to_owned();
    let write_term_types = "{0}".to_owned();

    json!({
        "lookup_name": lookup_name,
        "lhs_selector": lhs_selector,
        "rhs_selector": rhs_selector,
        "read_terms": read_terms,
        "write_terms": write_terms,
        "lookup_tuple_size": lookup_tuple_size,
        "inverse_degree": inverse_degree,
        "read_term_degree": read_term_degree,
        "write_term_degree": write_term_degree,
        "read_term_types": read_term_types,
        "write_term_types": write_term_types,
        "lookup_entities": lookup_entities,
    })
}

fn get_lookup_side<F: FieldElement>(
    def: &SelectedExpressions<AlgebraicExpression<F>>,
) -> LookupSide {
    let get_name = |expr: &AlgebraicExpression<F>| match expr {
        AlgebraicExpression::Reference(a_ref) => sanitize_name(&a_ref.name),
        _ => panic!("Expected reference"),
    };

    LookupSide {
        selector: def.selector.as_ref().map(get_name),
        cols: def.expressions.iter().map(get_name).collect_vec(),
    }
}
