use itertools::Itertools;
use powdr_ast::analyzed::AlgebraicBinaryOperation;
use powdr_ast::analyzed::AlgebraicExpression;
use powdr_ast::analyzed::AlgebraicUnaryOperation;
use powdr_ast::analyzed::Identity;
use powdr_ast::analyzed::{
    AlgebraicBinaryOperator, AlgebraicExpression as Expression, AlgebraicUnaryOperator,
    IdentityKind,
};
use powdr_ast::parsed::SelectedExpressions;
use std::collections::HashMap;
use std::collections::HashSet;
use std::path::Path;

use powdr_number::{BigUint, DegreeType, FieldElement};

use crate::file_writer::BBFiles;
use crate::utils::{capitalize, map_with_newline, snake_case};

/// Returned back to the vm builder from the create_relations call
pub struct RelationOutput {
    /// A list of the names of the created relations
    pub relations: Vec<String>,
    /// A list of the names of all of the 'used' shifted polys
    pub shifted_polys: Vec<String>,
}

/// Each created bb Identity is passed around with its degree so as needs to be manually
/// provided for sumcheck
type BBIdentity = (DegreeType, String);

pub trait RelationBuilder {
    /// Create Relations
    ///
    /// Takes in the ast ( for relations ), groups each of them by file, and then
    /// calls 'create relation' for each
    ///
    /// Relation output is passed back to the caller as the prover requires both:
    /// - The shifted polys
    /// - The names of the relations files created
    fn create_relations<F: FieldElement>(
        &self,
        root_name: &str,
        identities: &[Identity<AlgebraicExpression<F>>],
    ) -> RelationOutput;

    /// Create Relation
    ///
    /// Name and root name are required to determine the file path, e.g. it will be in the bberg/relations/generated
    /// followed by /root_name/name
    /// - root name should be the name provided with the --name flag
    /// - name will be a pil namespace
    ///
    /// - Identities are the identities that will be used to create the relations, they are generated within create_relations
    /// - row_type contains all of the columns that the relations namespace touches.
    fn create_relation(
        &self,
        root_name: &str,
        name: &str,
        sub_relations: &[String],
        identities: &[BBIdentity],
        row_type: &str,
        labels_lookup: String,
    );

    /// Declare views
    ///
    /// Declare views is a macro that generates a reference for each of the columns
    /// This reference will be a span into a sumcheck related object, it must be declared for EACH sub-relation
    /// as the sumcheck object is sensitive to the degree of the relation.
    fn create_declare_views(&self, name: &str, all_cols_and_shifts: &[String]);
}

impl RelationBuilder for BBFiles {
    fn create_relations<F: FieldElement>(
        &self,
        file_name: &str,
        analyzed_identities: &[Identity<AlgebraicExpression<F>>],
    ) -> RelationOutput {
        // Group relations per file
        let grouped_relations: HashMap<String, Vec<Identity<AlgebraicExpression<F>>>> =
            group_relations_per_file(analyzed_identities);
        let mut relations = grouped_relations.keys().cloned().collect_vec();
        relations.sort();

        // Contains all of the rows in each relation, will be useful for creating composite builder types
        let mut all_rows: HashMap<String, String> = HashMap::new();
        let mut shifted_polys: Vec<String> = Vec::new();

        // ----------------------- Create the relation files -----------------------
        for (relation_name, analyzed_idents) in grouped_relations.iter() {
            let IdentitiesOutput {
                subrelations,
                identities,
                collected_cols,
                collected_shifts,
                expression_labels,
            } = create_identities(file_name, analyzed_idents);

            // TODO: This can probably be moved into the create_identities function
            let row_type = create_row_type(&capitalize(relation_name), &collected_cols);

            // Aggregate all shifted polys
            shifted_polys.extend(collected_shifts);
            // Aggregate all rows
            all_rows.insert(relation_name.to_owned(), row_type.clone());

            let labels_lookup = create_relation_labels(relation_name, expression_labels);
            self.create_relation(
                file_name,
                relation_name,
                &subrelations,
                &identities,
                &row_type,
                labels_lookup,
            );
        }

        shifted_polys.sort();
        relations.sort();

        RelationOutput {
            relations,
            shifted_polys,
        }
    }

    fn create_relation(
        &self,
        root_name: &str,
        name: &str,
        sub_relations: &[String],
        identities: &[BBIdentity],
        row_type: &str,
        labels_lookup: String,
    ) {
        let includes = relation_includes();
        let class_boilerplate = relation_class_boilerplate(name, sub_relations, identities);
        let export = get_export(name);

        let relations = format!(
            "{includes}
namespace bb::{root_name}_vm {{

{row_type};

{labels_lookup}

{class_boilerplate}

{export}

        }}"
        );

        self.write_file(
            &format!("{}/{}", &self.rel, snake_case(root_name)),
            &format!("{}.hpp", snake_case(name)),
            &relations,
        );
    }

    fn create_declare_views(&self, name: &str, all_cols_and_shifts: &[String]) {
        let view_transformation =
            |name: &String| format!("[[maybe_unused]] auto {name} = View(new_term.{name});  \\");
        let make_view_per_row = map_with_newline(all_cols_and_shifts, view_transformation);

        let declare_views = format!(
            "
    #define {name}_DECLARE_VIEWS(index) \\
        using Accumulator = typename std::tuple_element<index, ContainerOverSubrelations>::type; \\
        using View = typename Accumulator::View; \\
        {make_view_per_row}


    "
        );

        self.write_file(
            &format!("{}/{}", &self.rel, snake_case(name)),
            "declare_views.hpp",
            &declare_views,
        );
    }
}

/// Group relations per file
///
/// The compiler returns all relations in one large vector, however we want to distinguish
/// which files .pil files the relations belong to for later code gen
///
/// Say we have two files foo.pil and bar.pil
/// foo.pil contains the following relations:
///    - foo1
///    - foo2
/// bar.pil contains the following relations:
///    - bar1
///    - bar2
///
/// This function will return a hashmap with the following structure:
/// {
///  "foo": [foo1, foo2],
///  "bar": [bar1, bar2]
/// }
///
/// This allows us to generate a relation.hpp file containing ONLY the relations for that .pil file
fn group_relations_per_file<F: FieldElement>(
    identities: &[Identity<AlgebraicExpression<F>>],
) -> HashMap<String, Vec<Identity<AlgebraicExpression<F>>>> {
    identities.iter().cloned().into_group_map_by(|identity| {
        identity
            .source
            .file_name
            .as_ref()
            .and_then(|file_name| Path::new(file_name.as_ref()).file_stem())
            .map(|stem| stem.to_string_lossy().into_owned())
            .unwrap_or_default()
            .replace(".pil", "")
    })
}

fn relation_class_boilerplate(
    name: &str,
    sub_relations: &[String],
    identities: &[BBIdentity],
) -> String {
    // We add one to all degrees because we have an extra scaling factor
    let degrees = identities.iter().map(|(d, _)| d + 1).collect();
    let degree_boilerplate = get_degree_boilerplate(degrees);
    let relation_code = get_relation_code(sub_relations);
    format!(
        "template <typename FF_> class {name}Impl {{
    public:
        using FF = FF_;
        
        {degree_boilerplate}
        
        {relation_code}
}};",
    )
}

fn get_export(name: &str) -> String {
    format!(
        "template <typename FF> using {name} = Relation<{name}Impl<FF>>;",
        name = name
    )
}

fn get_relation_code(ids: &[String]) -> String {
    let mut relation_code = r#"
    template <typename ContainerOverSubrelations, typename AllEntities>
    void static accumulate(
        ContainerOverSubrelations& evals,
        const AllEntities& new_term,
        [[maybe_unused]] const RelationParameters<FF>&,
        [[maybe_unused]] const FF& scaling_factor
    ){

    "#
    .to_owned();
    for id in ids {
        relation_code.push_str(&format!("{}\n", id));
    }
    relation_code.push_str("}\n");
    relation_code
}

fn get_degree_boilerplate(degrees: Vec<DegreeType>) -> String {
    let num_degrees = degrees.len();

    let mut degree_boilerplate = format!(
        "static constexpr std::array<size_t, {num_degrees}> SUBRELATION_PARTIAL_LENGTHS{{\n"
    );
    for degree in &degrees {
        degree_boilerplate.push_str(&format!("   {},\n", degree));
    }
    degree_boilerplate.push_str("};");

    degree_boilerplate
}

// The include statements required for a new relation file
fn relation_includes() -> &'static str {
    r#"
#pragma once
#include "../../relation_parameters.hpp"
#include "../../relation_types.hpp"
#include "./declare_views.hpp"
"#
}

// Each vm will need to have a row which is a combination of all of the witness columns
pub(crate) fn create_row_type(name: &str, all_rows: &[String]) -> String {
    let row_transformation = |row: &_| format!("    FF {row} {{}};");
    let all_annotated = map_with_newline(all_rows, row_transformation);

    format!(
        "template <typename FF> struct {name}Row {{
        {}

        [[maybe_unused]] static std::vector<std::string> names();
        }}",
        all_annotated,
    )
}

fn create_identity<T: FieldElement>(
    expression: &SelectedExpressions<Expression<T>>,
    collected_cols: &mut HashSet<String>,
    collected_public_identities: &mut HashSet<String>,
) -> Option<BBIdentity> {
    // We want to read the types of operators and then create the appropiate code

    if let Some(expr) = &expression.selector {
        let x = craft_expression(expr, collected_cols, collected_public_identities);
        log::trace!("expression {:?}", x);
        Some(x)
    } else {
        None
    }
}

// TODO: replace the preamble with a macro so the code looks nicer
fn create_subrelation(index: usize, preamble: String, identity: &mut BBIdentity) -> String {
    // \\\
    let id = &identity.1;

    format!(
        "//Contribution {index}
    {{\n{preamble}
    
    auto tmp = {id};
    tmp *= scaling_factor;
    std::get<{index}>(evals) += tmp;
}}",
    )
}

fn craft_expression<T: FieldElement>(
    expr: &Expression<T>,
    // TODO: maybe make state?
    collected_cols: &mut HashSet<String>,
    collected_public_identities: &mut HashSet<String>,
) -> BBIdentity {
    let var_name = match expr {
        Expression::Number(n) => {
            let number: BigUint = n.to_arbitrary_integer();
            if number.bit_len() < 32 {
                return (1, format!("FF({})", number));
            }
            if number.bit_len() < 64 {
                return (1, format!("FF({}UL)", number));
            }
            if number.bit_len() < 256 {
                let bytes = number.to_be_bytes();
                let padding_len = 32 - bytes.len();

                let mut padded_bytes = vec![0; padding_len];
                padded_bytes.extend_from_slice(&bytes);

                let mut chunks: Vec<u64> = padded_bytes
                    .chunks(8)
                    .map(|chunk| u64::from_be_bytes(chunk.try_into().unwrap()))
                    .collect();

                chunks.resize(4, 0);
                return (
                    1,
                    format!(
                        "FF(uint256_t{{{}UL, {}UL, {}UL, {}UL}})",
                        chunks[3], chunks[2], chunks[1], chunks[0],
                    ),
                );
            }
            unimplemented!("{:?}", expr);
        }
        Expression::Reference(polyref) => {
            let mut poly_name = polyref.name.replace('.', "_").to_string();
            if polyref.next {
                // NOTE: Naive algorithm to collect all shifted polys
                poly_name = format!("{}_shift", poly_name);
            }
            collected_cols.insert(poly_name.clone());
            (1, poly_name)
        }
        Expression::BinaryOperation(AlgebraicBinaryOperation {
            left: lhe,
            op,
            right: rhe,
        }) => {
            let (ld, lhs) = craft_expression(lhe, collected_cols, collected_public_identities);
            let (rd, rhs) = craft_expression(rhe, collected_cols, collected_public_identities);

            let degree = std::cmp::max(ld, rd);
            match op {
                AlgebraicBinaryOperator::Add => match lhe.as_ref() {
                    // BBerg hack, we do not want a field on the lhs of an expression
                    Expression::Number(_) => (degree, format!("({} + {})", rhs, lhs)),
                    _ => (degree, format!("({} + {})", lhs, rhs)),
                },
                AlgebraicBinaryOperator::Sub => {
                    // BBerg hack here, to make sure we dont have a trivial (- FF(0))
                    if let Expression::Number(rhe) = rhe.as_ref() {
                        // If the binary operation is a sub and the rhs expression is 0, we can just
                        // return the lhs
                        if rhe.to_arbitrary_integer() == 0u64.into() {
                            return (degree, lhs);
                        }
                    }
                    // Otherwise continue with the match
                    match lhe.as_ref() {
                        // BBerg hack, we do not want a field on the lhs of an expression
                        Expression::Number(_) => (degree, format!("(-{} + {})", rhs, lhs)),
                        _ => (degree, format!("({} - {})", lhs, rhs)),
                    }
                }
                AlgebraicBinaryOperator::Mul => match lhe.as_ref() {
                    // BBerg hack, we do not want a field on the lhs of an expression
                    Expression::Number(_) => (ld + rd, format!("({} * {})", rhs, lhs)),
                    _ => (ld + rd, format!("({} * {})", lhs, rhs)),
                },
                _ => unimplemented!("{:?}", expr),
            }
        }
        Expression::UnaryOperation(AlgebraicUnaryOperation {
            op: operator,
            expr: expression,
        }) => match operator {
            AlgebraicUnaryOperator::Minus => {
                let (d, e) =
                    craft_expression(expression, collected_cols, collected_public_identities);
                (d, format!("-{}", e))
            }
        },
        // TODO: for now we do nothing with calls to public identities
        // These probably can be implemented as some form of copy, however im not sure how we are going to process these down the line
        Expression::PublicReference(name) => {
            // We collect them for now to warn the user what is going on
            collected_public_identities.insert(name.clone());
            (1, "FF(0)".to_string())
        }
        // Note: challenges are not being used in our current pil construction
        Expression::Challenge(_) => unimplemented!("{:?}", expr),
    };
    var_name
}

pub struct IdentitiesOutput {
    subrelations: Vec<String>,
    identities: Vec<BBIdentity>,
    collected_cols: Vec<String>,
    collected_shifts: Vec<String>,
    expression_labels: HashMap<usize, String>,
}

pub(crate) fn create_identities<F: FieldElement>(
    file_name: &str,
    identities: &[Identity<Expression<F>>],
) -> IdentitiesOutput {
    // We only want the expressions for now
    // When we have a poly type, we only need the left side of it
    let ids = identities
        .iter()
        .filter(|identity| identity.kind == IdentityKind::Polynomial)
        .collect::<Vec<_>>();

    let mut identities = Vec::new();
    let mut subrelations = Vec::new();
    let mut expression_labels: HashMap<usize, String> = HashMap::new(); // Each relation can be given a label, this label can be assigned here
    let mut collected_cols: HashSet<String> = HashSet::new();
    let mut collected_public_identities: HashSet<String> = HashSet::new();

    // Collect labels for each identity
    // TODO: shite
    for (i, id) in ids.iter().enumerate() {
        if let Some(label) = &id.attribute {
            expression_labels.insert(i, label.clone());
        }
    }

    let expressions = ids.iter().map(|id| id.left.clone()).collect::<Vec<_>>();
    for (i, expression) in expressions.iter().enumerate() {
        let relation_boilerplate = format!(
            "{file_name}_DECLARE_VIEWS({i});
        ",
        );

        // TODO: collected pattern is shit
        let mut identity = create_identity(
            expression,
            &mut collected_cols,
            &mut collected_public_identities,
        )
        .unwrap();
        let subrelation = create_subrelation(i, relation_boilerplate, &mut identity);

        identities.push(identity);

        subrelations.push(subrelation);
    }

    // Print a warning to the user about usage of public identities
    if !collected_public_identities.is_empty() {
        log::warn!(
            "Public Identities are not supported yet in codegen, however some were collected"
        );
        log::warn!("Public Identities: {:?}", collected_public_identities);
    }

    let mut collected_cols: Vec<String> = collected_cols.drain().collect();
    let mut collected_shifts: Vec<String> = collected_cols
        .clone()
        .iter()
        .filter_map(|col| {
            if col.ends_with("shift") {
                Some(col.clone())
            } else {
                None
            }
        })
        .collect();

    collected_cols.sort();
    collected_shifts.sort();

    IdentitiesOutput {
        subrelations,
        identities,
        collected_cols,
        collected_shifts,
        expression_labels,
    }
}

/// Relation labels
///
/// To view relation labels we create a sparse switch that contains all of the collected labels
/// Whenever there is a failure, we can lookup into this mapping
///
/// Note: this mapping will never be that big, so we are quite naive in implementation
/// It should be able to be called from else where with relation_name::get_relation_label
fn create_relation_labels(relation_name: &str, labels: HashMap<usize, String>) -> String {
    // Sort labels by the index
    let label_transformation = |(index, label)| {
        format!(
            "case {index}:
            return \"{label}\";
        "
        )
    };

    // Sort the labels by their index
    let mut sorted_labels: Vec<(usize, String)> = labels.into_iter().collect();
    sorted_labels.sort_by(|a, b| a.0.cmp(&b.0));

    let switch_statement: String = sorted_labels
        .into_iter()
        .map(label_transformation)
        .collect::<Vec<String>>()
        .join("\n");

    format!(
        "
    inline std::string get_relation_label_{relation_name}(int index) {{
        switch (index) {{
            {switch_statement}
        }}
        return std::to_string(index);
    }}
    "
    )
}
