use crate::{
    file_writer::BBFiles,
    utils::{create_get_const_entities, create_get_nonconst_entities, snake_case},
};
use itertools::Itertools;
use powdr_ast::{
    analyzed::{AlgebraicExpression, Analyzed, Identity, IdentityKind},
    parsed::SelectedExpressions,
};
use powdr_number::FieldElement;

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
        let lookups: Vec<&Identity<AlgebraicExpression<F>>> = analyzed
            .identities
            .iter()
            .filter(|identity| matches!(identity.kind, IdentityKind::Plookup))
            .collect();
        let new_lookups = lookups
            .iter()
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

        create_lookups(self, project_name, &new_lookups);
        new_lookups
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

/// Write the lookup settings files to disk
fn create_lookups(bb_files: &BBFiles, project_name: &str, lookups: &Vec<Lookup>) {
    for lookup in lookups {
        let lookup_settings = create_lookup_settings_file(lookup);

        let folder = format!("{}/{}", bb_files.rel, &snake_case(project_name));
        let file_name = format!(
            "{}{}",
            lookup.attribute.clone().unwrap_or("NONAME".to_owned()),
            ".hpp".to_owned()
        );
        bb_files.write_file(&folder, &file_name, &lookup_settings);
    }
}

/// All relation types eventually get wrapped in the relation type
/// This function creates the export for the relation type so that it can be added to the flavor
fn create_relation_exporter(lookup_name: &str) -> String {
    let settings_name = format!("{}_lookup_settings", lookup_name);
    let lookup_export = format!("template <typename FF_> using {lookup_name}_relation = GenericLookupRelation<{settings_name}, FF_>;");
    let relation_export = format!(
        "template <typename FF_> using {lookup_name} = GenericLookup<{settings_name}, FF_>;"
    );

    format!(
        "
    {lookup_export} 
    {relation_export} 
    "
    )
}

fn lookup_settings_includes() -> &'static str {
    r#"
    #pragma once

    #include "barretenberg/relations/generic_lookup/generic_lookup_relation.hpp"

    #include <cstddef>
    #include <tuple> 
    "#
}

fn create_lookup_settings_file(lookup: &Lookup) -> String {
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
    let read_term_types = "{0}";
    let write_term_types = "{0}";

    let lookup_settings_includes = lookup_settings_includes();
    let inverse_polynomial_is_computed_at_row =
        create_inverse_computed_at(&lhs_selector, &rhs_selector);
    let compute_inverse_exists = create_compute_inverse_exist(&lhs_selector, &rhs_selector);
    let const_entities = create_get_const_entities(&lookup_entities);
    let nonconst_entities = create_get_nonconst_entities(&lookup_entities);
    let relation_exporter = create_relation_exporter(&lookup_name);

    format!(
        "
        {lookup_settings_includes}

        namespace bb {{

        /**
         * @brief This class contains an example of how to set LookupSettings classes used by the
         * GenericLookupRelationImpl class to specify a scaled lookup
         *
         * @details To create your own lookup:
         * 1) Create a copy of this class and rename it
         * 2) Update all the values with the ones needed for your lookup
         * 3) Update \"DECLARE_LOOKUP_IMPLEMENTATIONS_FOR_ALL_SETTINGS\" and \"DEFINE_LOOKUP_IMPLEMENTATIONS_FOR_ALL_SETTINGS\" to
         * include the new settings
         * 4) Add the relation with the chosen settings to Relations in the flavor (for example,\"`
         *   using Relations = std::tuple<GenericLookupRelation<ExampleXorLookupSettings,
         * FF>>;)`
         *
         */
        class {lookup_name}_lookup_settings {{
          public:
            /**
             * @brief The number of read terms (how many lookups we perform) in each row
             *
             */
            static constexpr size_t READ_TERMS = {read_terms};
            /**
             * @brief The number of write terms (how many additions to the lookup table we make) in each row
             *
             */
            static constexpr size_t WRITE_TERMS = {write_terms};
        
            /**
             * @brief The type of READ_TERM used for each read index (basic and scaled)
             *
             */
            static constexpr size_t READ_TERM_TYPES[READ_TERMS] = {read_term_types};
        
            /**
             * @brief They type of WRITE_TERM used for each write index
             *
             */
            static constexpr size_t WRITE_TERM_TYPES[WRITE_TERMS] = {write_term_types};

            /**
             * @brief How many values represent a single lookup object. This value is used by the automatic read term
             * implementation in the relation in case the lookup is a basic or scaled tuple and in the write term if it's a
             * basic tuple
             *
             */
            static constexpr size_t LOOKUP_TUPLE_SIZE = {lookup_tuple_size};
        
            /**
             * @brief The polynomial degree of the relation telling us if the inverse polynomial value needs to be computed
             *
             */
            static constexpr size_t INVERSE_EXISTS_POLYNOMIAL_DEGREE = {inverse_degree};
        
            /**
             * @brief The degree of the read term if implemented arbitrarily. This value is not used by basic and scaled read
             * terms, but will cause compilation error if not defined
             *
             */
            static constexpr size_t READ_TERM_DEGREE = {read_term_degree};
        
            /**
             * @brief The degree of the write term if implemented arbitrarily. This value is not used by the basic write
             * term, but will cause compilation error if not defined
             *
             */
        
            static constexpr size_t WRITE_TERM_DEGREE = {write_term_degree};
        
            /**
             * @brief If this method returns true on a row of values, then the inverse polynomial exists at this index.
             * Otherwise the value needs to be set to zero.
             *
             * @details If this is true then the lookup takes place in this row
             *
             */
            {inverse_polynomial_is_computed_at_row}
        
            /**
             * @brief Subprocedure for computing the value deciding if the inverse polynomial value needs to be checked in this
             * row
             *
             * @tparam Accumulator Type specified by the lookup relation
             * @tparam AllEntities Values/Univariates of all entities row
             * @param in Value/Univariate of all entities at row/edge
             * @return Accumulator
             */
            {compute_inverse_exists}
        
            /**
             * @brief Get all the entities for the lookup when need to update them
             *
             * @details The generic structure of this tuple is described in ./generic_lookup_relation.hpp . The following is
             description for the current case:
             The entities are returned as a tuple of references in the following order (this is for ):
             * - The entity/polynomial used to store the product of the inverse values
             * - The entity/polynomial that specifies how many times the lookup table entry at this row has been looked up
             * - READ_TERMS entities/polynomials that enable individual lookup operations
             * - The entity/polynomial that enables adding an entry to the lookup table in this row
             * - LOOKUP_TUPLE_SIZE entities/polynomials representing the basic tuple being looked up as the first read term
             * - LOOKUP_TUPLE_SIZE entities/polynomials representing the previous accumulators in the second read term
             (scaled tuple)
             * - LOOKUP_TUPLE_SIZE entities/polynomials representing the shifts in the second read term (scaled tuple)
             * - LOOKUP_TUPLE_SIZE entities/polynomials representing the current accumulators in the second read term
             (scaled tuple)
             * - LOOKUP_TUPLE_SIZE entities/polynomials representing basic tuples added to the table
             *
             * @return All the entities needed for the lookup
             */
            {const_entities}

            /**
             * @brief Get all the entities for the lookup when we only need to read them
             * @details Same as in get_const_entities, but nonconst
             *
             * @return All the entities needed for the lookup
             */
            {nonconst_entities}
        }};

        {relation_exporter}
    }}
        "
    )
}

fn create_inverse_computed_at(lhs_selector: &String, rhs_selector: &String) -> String {
    let lhs_computed_selector = format!("in.{lhs_selector}");
    let rhs_computed_selector = format!("in.{rhs_selector}");
    format!("
    template <typename AllEntities> static inline auto inverse_polynomial_is_computed_at_row(const AllEntities& in) {{
        return ({lhs_computed_selector } == 1 || {rhs_computed_selector} == 1);
    }}")
}

fn create_compute_inverse_exist(lhs_selector: &String, rhs_selector: &String) -> String {
    let lhs_computed_selector = format!("in.{lhs_selector}");
    let rhs_computed_selector = format!("in.{rhs_selector}");
    format!("
    template <typename Accumulator, typename AllEntities> static inline auto compute_inverse_exists(const AllEntities& in) {{
        using View = typename Accumulator::View;
        const auto is_operation = View({lhs_computed_selector});
        const auto is_table_entry = View({rhs_computed_selector});
        return (is_operation + is_table_entry - is_operation * is_table_entry);
    }}")
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
