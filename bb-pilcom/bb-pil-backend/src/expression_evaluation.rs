use crate::utils::{format_field, sanitize_name};
use itertools::Itertools;

use std::collections::{HashMap, HashSet};

use powdr_ast::analyzed::{
    AlgebraicBinaryOperation, AlgebraicBinaryOperator, AlgebraicExpression,
    AlgebraicUnaryOperation, AlgebraicUnaryOperator, Analyzed, Symbol,
};
use powdr_number::FieldElement;

pub fn get_alias_expressions_in_order<F: FieldElement>(
    alias_polys_in_order: &Vec<(&Symbol, &AlgebraicExpression<F>)>,
) -> Vec<(String, u64, String)> {
    let aliases_hash = alias_polys_in_order
        .iter()
        .map(|(sym, expr)| (&sym.absolute_name, *expr))
        .collect::<HashMap<_, _>>();

    alias_polys_in_order
        .iter()
        .map(|(sym, _)| {
            let (degree, expr) =
                get_alias_expression_and_degree(sym.absolute_name.as_str(), &aliases_hash);
            (sanitize_name(&sym.absolute_name), degree, expr)
        })
        .collect_vec()
}

pub fn get_alias_polys_in_order<F: FieldElement>(
    analyzed: &Analyzed<F>,
) -> Vec<(&Symbol, &AlgebraicExpression<F>)> {
    analyzed
        .intermediate_polys_in_source_order()
        .iter()
        .map(|(s, exprs)| {
            (
                s,
                // Only support the first expression for now.
                // I don't even know what more than one means.
                exprs.first().unwrap(),
            )
        })
        .collect_vec()
}

pub fn get_alias_expression_and_degree<F: FieldElement>(
    alias_name: &str,
    aliases: &HashMap<&String, &AlgebraicExpression<F>>,
) -> (u64, String) {
    let (degree, expression, _) =
        recurse_expression(aliases.get(&alias_name.to_owned()).unwrap(), aliases, false);
    (degree, expression)
}

// Returns (degree, expression, transitive aliases)
// Will always return all the used aliases.
// Aliases are sanitized.
pub fn recurse_expression<F: FieldElement>(
    top_expr: &AlgebraicExpression<F>,
    aliases: &HashMap<&String, &AlgebraicExpression<F>>,
    inline_aliases: bool,
) -> (u64, String, HashSet<String>) {
    match top_expr {
        AlgebraicExpression::Number(n) => (0, format_field(n), HashSet::new()),
        AlgebraicExpression::Reference(polyref) => {
            if aliases.contains_key(&polyref.name) {
                let alias_expr = aliases.get(&polyref.name).unwrap();
                let (d, expr, mut rec_aliases) =
                    recurse_expression(alias_expr, aliases, inline_aliases);
                let sanitized_name = sanitize_name(&polyref.name);
                rec_aliases.insert(sanitized_name.clone());
                if inline_aliases {
                    (d, expr, rec_aliases)
                } else {
                    (d, sanitized_name, rec_aliases)
                }
            } else {
                let mut poly_name = sanitize_name(&polyref.name);
                if polyref.next {
                    poly_name = format!("{}_shift", poly_name);
                }
                (1, format!("new_term.{}", poly_name), HashSet::new())
            }
        }
        AlgebraicExpression::BinaryOperation(AlgebraicBinaryOperation {
            left: lhe,
            op,
            right: rhe,
        }) => {
            let (ld, lhs, lhs_aliases) = recurse_expression(lhe, aliases, inline_aliases);
            let (rd, rhs, rhs_aliases) = recurse_expression(rhe, aliases, inline_aliases);
            let joined_aliases = lhs_aliases.union(&rhs_aliases).cloned().collect();

            match op {
                AlgebraicBinaryOperator::Add => (
                    std::cmp::max(ld, rd),
                    format!("({} + {})", lhs, rhs),
                    joined_aliases,
                ),
                AlgebraicBinaryOperator::Sub =>
                // There seem to be many cases where the rhs is a 0, try to avoid it.
                {
                    if **rhe == AlgebraicExpression::Number(F::zero()) {
                        (ld, lhs, joined_aliases)
                    } else {
                        (
                            std::cmp::max(ld, rd),
                            format!("({} - {})", lhs, rhs),
                            joined_aliases,
                        )
                    }
                }
                AlgebraicBinaryOperator::Mul => {
                    (ld + rd, format!("({} * {})", lhs, rhs), joined_aliases)
                }
                _ => unimplemented!("{:?}", op),
            }
        }
        AlgebraicExpression::UnaryOperation(AlgebraicUnaryOperation {
            op: operator,
            expr: rec_expr,
        }) => match operator {
            AlgebraicUnaryOperator::Minus => {
                let (d, e, rec_aliases) = recurse_expression(rec_expr, aliases, inline_aliases);
                (d, format!("-{}", e), rec_aliases)
            }
        },
        // Not currently used
        AlgebraicExpression::PublicReference(_) => unimplemented!("{:?}", top_expr),
        // Challenges are not being used in our current pil construction
        AlgebraicExpression::Challenge(_) => unimplemented!("{:?}", top_expr),
    }
}
