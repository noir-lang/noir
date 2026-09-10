//! Checks on the shape of the program monomorphization produced.
//!
//! A malformed program is wrong in a way that reads as valid all the way into SSA generation,
//! because the identifiers involved are plain integers that index into a `Vec`: a function id that
//! is off by one does not fail to resolve, it resolves to a different function.
//!
//! Run in debug builds.

use super::ast::{Definition, Expression, Function, Program};
use super::visitor::visit_ident_mut;

/// Panic if `program` is not internally consistent.
///
/// Takes `&mut` only because the AST visitors do; nothing here changes the program.
pub(super) fn assert_program_is_well_formed(program: &mut Program) {
    let problems = problems(program);

    assert!(
        problems.is_empty(),
        "monomorphization produced a program that does not hold together:\n  {}",
        problems.join("\n  "),
    );
}

fn problems(program: &mut Program) -> Vec<String> {
    let mut problems = Vec::new();

    problems.extend(function_ids_match_their_positions(&program.functions));
    problems.extend(references_resolve(program));

    problems.sort();
    problems
}

/// A `FuncId` is an index into `Program::functions`, so a function's position in that vector has
/// to be its own id.
///
/// Nothing else holds that together. `Monomorphizer::into_program` collects a
/// `BTreeMap<FuncId, Function>` into a vector and drops the keys, so a gap in the ids — a
/// function whose id was allocated and never filled in — would shift every function above it down
/// one place while leaving the ids in call sites pointing at the old positions. That is silent:
/// every call still resolves, to the wrong function. `create_foreign_proxies` then appends more
/// functions afterwards and has to keep the same correspondence.
fn function_ids_match_their_positions(functions: &[Function]) -> Vec<String> {
    functions
        .iter()
        .enumerate()
        .filter(|(position, function)| function.id.0 as usize != *position)
        .map(|(position, function)| {
            format!(
                "function `{}` sits at position {position} but carries id {}, so calls to it \
                 resolve to whichever function is at position {}",
                function.name, function.id.0, function.id.0
            )
        })
        .collect()
}

/// Every function and global an identifier names has to exist.
///
/// These ids index a `Vec` and a `BTreeMap` that a later pass reads without checking, so one that
/// points past the end is a panic in SSA generation at best and a call to an unrelated function at
/// worst. Locals are not covered: they are scoped to a function body, so checking them means
/// tracking scopes rather than looking an id up.
fn references_resolve(program: &mut Program) -> Vec<String> {
    let function_count = program.functions.len();
    let defined_globals: Vec<_> = program.globals.keys().copied().collect();
    let mut problems = Vec::new();

    let check = |where_: &str, body: &mut Expression, problems: &mut Vec<String>| {
        visit_ident_mut(body, &mut |ident| match ident.definition {
            Definition::Function(id) if id.0 as usize >= function_count => {
                problems.push(format!(
                    "`{where_}` calls function {id}, but the program only has {function_count}"
                ));
            }
            Definition::Global(id) if !defined_globals.contains(&id) => {
                problems.push(format!(
                    "`{where_}` reads global {}, which the program does not define",
                    id.0
                ));
            }
            _ => (),
        });
    };

    for function in &mut program.functions {
        let name = function.name.clone();
        check(&name, &mut function.body, &mut problems);
    }

    for (id, (name, _, value)) in &mut program.globals {
        let where_ = format!("global {} ({name})", id.0);
        check(&where_, value, &mut problems);
    }

    problems
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::monomorphization::ast::FuncId;
    use crate::monomorphization::visitor::visit_ident_mut;
    use crate::test_utils::get_monomorphized;

    /// A program with a call in it, so there is a `Definition::Function` to point somewhere else
    /// and more than one function to put in the wrong order.
    fn program_with_a_call() -> Program {
        let src = r#"
            fn callee() -> Field { 1 }

            fn main() {
                let _ = callee();
            }
        "#;
        let program = get_monomorphized(src).expect("the program monomorphizes");
        assert!(program.functions.len() > 1, "expected a call to produce a second function");
        program
    }

    #[test]
    fn a_well_formed_program_has_no_problems() {
        let mut program = program_with_a_call();
        assert_eq!(problems(&mut program), Vec::<String>::new());
    }

    #[test]
    fn reports_a_function_that_is_not_where_its_id_says() {
        let mut program = program_with_a_call();
        program.functions.swap(0, 1);

        let problems = problems(&mut program);
        assert_eq!(problems.len(), 2, "{problems:?}");
        assert!(
            problems.iter().all(|problem| problem.contains("sits at position")),
            "{problems:?}"
        );
    }

    #[test]
    fn reports_a_call_to_a_function_the_program_does_not_have() {
        let mut program = program_with_a_call();
        let past_the_end = FuncId(program.functions.len() as u32);

        for function in &mut program.functions {
            visit_ident_mut(&mut function.body, &mut |ident| {
                if let Definition::Function(id) = &mut ident.definition {
                    *id = past_the_end;
                }
            });
        }

        let problems = problems(&mut program);
        assert!(!problems.is_empty(), "a dangling call was not reported");
        assert!(
            problems.iter().all(|problem| problem.contains("but the program only has")),
            "{problems:?}"
        );
    }
}
