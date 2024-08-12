use itertools::Itertools;
use powdr_ast::analyzed::AlgebraicBinaryOperation;
use powdr_ast::analyzed::AlgebraicUnaryOperation;
use powdr_ast::analyzed::Analyzed;
use powdr_ast::analyzed::Identity;
use powdr_ast::analyzed::{AlgebraicExpression, IdentityKind};
use powdr_ast::parsed::SelectedExpressions;
use std::collections::HashMap;
use std::collections::HashSet;
use std::path::Path;

use powdr_number::{DegreeType, FieldElement};

use handlebars::Handlebars;
use serde_json::json;

use crate::expression_evaluation::get_alias_expressions_in_order;
use crate::expression_evaluation::get_alias_polys_in_order;
use crate::expression_evaluation::recurse_expression;
use crate::file_writer::BBFiles;
use crate::utils::snake_case;

/// Each created bb Identity is passed around with its degree so as needs to be manually
/// provided for sumcheck
#[derive(Debug)]
pub struct BBIdentity {
    pub degree: DegreeType,
    pub identity: String,
    pub label: Option<String>,
}

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
        analyzed: &Analyzed<F>,
    ) -> Vec<String>;

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
        identities: &[BBIdentity],
        skippable_if: &Option<BBIdentity>,
        alias_polys_in_order: &Vec<(String, u64, String)>,
    );
}

impl RelationBuilder for BBFiles {
    fn create_relations<F: FieldElement>(
        &self,
        file_name: &str,
        analyzed: &Analyzed<F>,
    ) -> Vec<String> {
        // These identities' terminal objects are either fields, columns, or alias expressions.
        let mut analyzed_identities = analyzed.identities.clone();
        analyzed_identities.sort_by(|a, b| a.id.cmp(&b.id));

        let alias_polys_in_order = get_alias_polys_in_order(analyzed);
        let alias_expressions_in_order = get_alias_expressions_in_order(&alias_polys_in_order);
        let indexed_aliases = alias_polys_in_order
            .into_iter()
            .map(|(sym, expr)| (&sym.absolute_name, expr))
            .collect::<HashMap<_, _>>();

        // Group relations per file
        let grouped_relations: HashMap<String, Vec<Identity<AlgebraicExpression<F>>>> =
            group_relations_per_file(&analyzed_identities);
        let mut relations = grouped_relations.keys().cloned().collect_vec();
        relations.sort();

        // ----------------------- Create the relation files -----------------------
        for (relation_name, analyzed_idents) in grouped_relations.iter() {
            let IdentitiesOutput {
                identities,
                skippable_if,
                // These are the aliases used in the identities in this file.
                collected_aliases,
            } = create_identities(analyzed_idents, &indexed_aliases);

            let used_alias_defs_in_order = alias_expressions_in_order
                .iter()
                .filter(|(name, _, _)| collected_aliases.contains(name))
                .cloned()
                .collect_vec();

            self.create_relation(
                file_name,
                relation_name,
                &identities,
                &skippable_if,
                &used_alias_defs_in_order,
            );
        }

        relations.sort();

        relations
    }

    fn create_relation(
        &self,
        root_name: &str,
        name: &str,
        identities: &[BBIdentity],
        skippable_if: &Option<BBIdentity>,
        alias_defs_in_order: &Vec<(String, u64, String)>,
    ) {
        let mut handlebars = Handlebars::new();
        handlebars.register_escape_fn(|s| s.to_string()); // No escaping

        let degrees: Vec<_> = identities.iter().map(|id| id.degree + 1).collect();
        let sorted_labels = identities
            .iter()
            .enumerate()
            .filter(|(_, id)| id.label.is_some())
            .map(|(idx, id)| (idx, id.label.clone().unwrap()))
            // Useful for debugging
            // .map(|(idx, id)| (idx, id.label.as_ref().unwrap_or(&id.identity).clone()))
            .collect_vec();

        let data = &json!({
            "root_name": root_name,
            "name": name,
            "identities": identities.iter().map(|id| {
                json!({
                    "degree": id.degree,
                    "identity": id.identity,
                })
            }).collect_vec(),
            "alias_defs": alias_defs_in_order.iter().map(|(name, degree, expr)| {
                json!({
                    "name": name,
                    "degree": degree,
                    "expr": expr,
                })
            }).collect_vec(),
            "skippable_if": skippable_if.as_ref().map(|id| id.identity.clone()),
            "degrees": degrees,
            "labels": sorted_labels,
        });

        handlebars
            .register_template_string(
                "relation.hpp",
                std::str::from_utf8(include_bytes!("../templates/relation.hpp.hbs")).unwrap(),
            )
            .unwrap();

        let relation_hpp = handlebars.render("relation.hpp", data).unwrap();

        self.write_file(
            Some(&self.relations),
            &format!("{}.hpp", snake_case(name)),
            &relation_hpp,
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

fn create_identity<F: FieldElement>(
    expression: &SelectedExpressions<AlgebraicExpression<F>>,
    collected_aliases: &mut HashSet<String>,
    label: &Option<String>,
    indexed_aliases: &HashMap<&String, &AlgebraicExpression<F>>,
) -> Option<BBIdentity> {
    // We want to read the types of operators and then create the appropiate code
    if let Some(expr) = &expression.selector {
        let (degree, id, col_aliases) = recurse_expression(expr, indexed_aliases, false);
        collected_aliases.extend(col_aliases);
        log::trace!("expression {:?}, {:?}", degree, id);
        Some(BBIdentity {
            degree: degree,
            identity: id,
            label: label.clone(),
        })
    } else {
        None
    }
}

pub struct IdentitiesOutput {
    identities: Vec<BBIdentity>,
    skippable_if: Option<BBIdentity>,
    collected_aliases: HashSet<String>,
}

pub(crate) fn create_identities<F: FieldElement>(
    identities: &[Identity<AlgebraicExpression<F>>],
    indexed_aliases: &HashMap<&String, &AlgebraicExpression<F>>,
) -> IdentitiesOutput {
    // We only want the expressions for now
    // When we have a poly type, we only need the left side of it
    let ids = identities
        .iter()
        .filter(|identity| identity.kind == IdentityKind::Polynomial)
        .collect::<Vec<_>>();

    let mut identities = Vec::new();
    let mut skippable_if_identity = None;
    let mut collected_aliases: HashSet<String> = HashSet::new();

    for expression in ids.iter() {
        let identity = create_identity(
            &expression.left,
            &mut collected_aliases,
            &expression.attribute,
            indexed_aliases,
        )
        .unwrap();

        if identity.label.clone().is_some_and(|l| l == "skippable_if") {
            assert!(skippable_if_identity.is_none());
            skippable_if_identity = Some(identity);
        } else {
            identities.push(identity);
        }
    }

    IdentitiesOutput {
        identities,
        skippable_if: skippable_if_identity,
        collected_aliases,
    }
}

pub fn get_shifted_polys<F: FieldElement>(expressions: Vec<AlgebraicExpression<F>>) -> Vec<String> {
    let mut shifted_polys = HashSet::<String>::new();
    for expr in expressions {
        match expr {
            AlgebraicExpression::Reference(polyref) => {
                if polyref.next {
                    shifted_polys.insert(polyref.name.clone());
                }
            }
            AlgebraicExpression::BinaryOperation(AlgebraicBinaryOperation {
                left: lhe,
                right: rhe,
                ..
            }) => {
                shifted_polys.extend(get_shifted_polys(vec![*lhe]));
                shifted_polys.extend(get_shifted_polys(vec![*rhe]));
            }
            AlgebraicExpression::UnaryOperation(AlgebraicUnaryOperation { expr, .. }) => {
                shifted_polys.extend(get_shifted_polys(vec![*expr]));
            }
            _ => continue,
        }
    }
    shifted_polys.into_iter().collect()
}
