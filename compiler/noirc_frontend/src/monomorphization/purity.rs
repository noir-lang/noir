//! Monomorphization's check that it left the elaborated context as it found it.
//!
//! A `NodeInterner` is reusable across entry points: `nargo export` monomorphizes every exported
//! function of one context, and `tooling/nargo_cli/tests/stdlib-tests.rs` runs every stdlib test
//! against one. That holds only while monomorphization writes nothing to the interner it does not
//! put back, and there are two ways it can write.
//!
//! Type variable bindings are shared with the HIR through an `Rc<RefCell<_>>`, so writing one
//! needs no mutable reference to the interner at all; [`type_variable_writes`] records those at
//! the four methods that perform them. Instantiation bindings are stored on the interner per
//! expression and replaced wholesale, so a snapshot of that map catches those.
//!
//! Neither part reads the guards the monomorphiser uses to undo its own writes. That is the
//! point: a guard the monomorphiser forgot to hold is exactly what this has to be able to see.
//!
//! A check is opened around both places the monomorphiser runs: `monomorphize`, and the
//! elaborator's own use of a `Monomorphizer` to lower the value comptime evaluation produced for
//! a `main`.

use std::sync::OnceLock;

use rustc_hash::FxHashMap as HashMap;

use crate::hir_def::type_variable_writes::{self, WriteCheck};
use crate::node_interner::{ExprId, Growth, NodeInterner};

/// Whether monomorphization checks its own purity.
///
/// On in debug builds and off in release. `NOIR_CHECK_MONOMORPHIZATION_PURITY=1` turns it on in
/// any build, so a release CI job can run it; `=0` turns it off.
fn checking_enabled() -> bool {
    static ENABLED: OnceLock<bool> = OnceLock::new();
    *ENABLED.get_or_init(|| match std::env::var("NOIR_CHECK_MONOMORPHIZATION_PURITY") {
        Ok(value) => value != "0",
        Err(_) => cfg!(debug_assertions),
    })
}

/// A check open over the writes made to an interner since it was started.
///
/// Does nothing at all when checking is disabled, so the caller does not have to ask.
pub(crate) struct PurityCheck {
    state: Option<State>,
}

struct State {
    writes: WriteCheck,
    bindings: HashMap<ExprId, u64>,
    sizes: Vec<(&'static str, Growth, usize)>,
}

impl PurityCheck {
    pub(crate) fn begin(interner: &NodeInterner) -> Self {
        if !checking_enabled() {
            return Self { state: None };
        }
        Self {
            state: Some(State {
                writes: type_variable_writes::begin(),
                bindings: fingerprint(interner),
                sizes: interner.state_sizes(),
            }),
        }
    }

    /// Panic if the interner is not in the state it was in when the check was started.
    pub(crate) fn assert_context_unchanged(self, interner: &NodeInterner) {
        let differences = self.differences(interner);

        assert!(
            differences.is_empty(),
            "monomorphization left the elaborated context different to how it found it, so a \
             later compilation against this context can see what this one did:\n  {}",
            differences.join("\n  "),
        );
    }

    /// Describe everything about `interner` that differs from when the check was started.
    ///
    /// Empty when nothing does, and when checking is disabled.
    fn differences(self, interner: &NodeInterner) -> Vec<String> {
        let Some(state) = self.state else {
            return Vec::new();
        };

        let mut differences = type_variable_writes::finish(state.writes);
        differences.extend(describe_bindings_drift(&state.bindings, interner));
        differences.extend(describe_size_drift(&state.sizes, interner));
        differences
    }
}

/// A per-expression digest of the interner's stored instantiation bindings.
///
/// Each expression's bindings are combined with `^` so the digest does not depend on the order a
/// `HashMap` happens to iterate in. Digesting rather than cloning the map keeps the check cheap
/// enough to leave on for every compilation in a debug build; the entries that differ are read
/// back out of the interner to describe them, which only happens when something is wrong.
fn fingerprint(interner: &NodeInterner) -> HashMap<ExprId, u64> {
    interner
        .all_instantiation_bindings()
        .map(|(expr_id, bindings)| {
            let digest = bindings
                .iter()
                .map(|(id, (_, kind, typ))| hash_of(&(id, kind, typ)))
                .fold(0, |combined, entry| combined ^ entry);
            (expr_id, digest)
        })
        .collect()
}

fn hash_of(value: &impl std::hash::Hash) -> u64 {
    use std::hash::{BuildHasher, BuildHasherDefault};
    BuildHasherDefault::<rustc_hash::FxHasher>::default().hash_one(value)
}

/// Describe every expression that had instantiation bindings when the check began and no longer
/// has the same ones.
///
/// Expressions the check has not seen before are ignored. Monomorphization builds HIR for the
/// values comptime evaluation produced for globals, and each `Value::Function` it lowers pushes a
/// fresh expression and stores that function's bindings against it. Those are new nodes carrying
/// their own state, reachable only from themselves; a compilation that never sees them cannot be
/// affected by them, which is not true of a change to an expression that was already there.
fn describe_bindings_drift(before: &HashMap<ExprId, u64>, interner: &NodeInterner) -> Vec<String> {
    let after = fingerprint(interner);

    let mut drift: Vec<String> = before
        .iter()
        .filter(|(expr_id, digest)| after.get(expr_id) != Some(digest))
        .map(|(expr_id, _)| match interner.try_get_instantiation_bindings(*expr_id) {
            Some(bindings) => {
                let mut targets: Vec<String> = bindings
                    .iter()
                    .map(|(id, (_, _, typ))| format!("{} to `{typ}`", id.0))
                    .collect();
                targets.sort();
                format!("the instantiation bindings of {expr_id:?} now bind {}", targets.join(", "))
            }
            None => format!("the instantiation bindings of {expr_id:?} have been removed"),
        })
        .collect();

    drift.sort();
    drift
}

/// Describe every piece of interner state whose size changed in a way it is not allowed to.
///
/// Coarser than the two checks above and much broader: it covers every field of the interner
/// rather than the two channels monomorphization is known to write, so a pass that starts
/// inserting somewhere new is caught without anyone having had to think of that field in
/// advance. Nothing may shrink; only state keyed by an id the pass created may grow.
fn describe_size_drift(
    before: &[(&'static str, Growth, usize)],
    interner: &NodeInterner,
) -> Vec<String> {
    before
        .iter()
        .zip(interner.state_sizes())
        .filter_map(|((name, growth, was), (_, _, now))| match growth {
            Growth::Fixed if *was != now => {
                Some(format!("`{name}` went from {was} to {now}, and that is state a pass reading the interner may not change"))
            }
            Growth::AppendOnly if now < *was => {
                Some(format!("`{name}` went from {was} to {now}, losing entries that were already there"))
            }
            _ => None,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::TypeBindings;
    use crate::hir_def::expr::HirExpression;
    use crate::hir_def::types::Type;
    use crate::test_utils::get_program;

    /// A program with a generic call, so the interner holds instantiation bindings to tamper
    /// with, and the expression they belong to.
    fn context_with_instantiation_bindings() -> (crate::hir::Context<'static, 'static>, ExprId) {
        let src = r#"
            fn identity<T>(x: T) -> T { x }

            fn main() {
                let _ = identity(1);
            }
        "#;
        let (_, context, _) = get_program(src);
        let (expr_id, _) = context
            .def_interner
            .all_instantiation_bindings()
            .next()
            .expect("a generic call stores instantiation bindings");
        (context, expr_id)
    }

    #[test]
    fn reports_an_expression_whose_instantiation_bindings_changed() {
        let (mut context, expr_id) = context_with_instantiation_bindings();
        let interner = &mut context.def_interner;

        let check = PurityCheck::begin(interner);
        interner.store_instantiation_bindings(expr_id, TypeBindings::default());

        let differences = check.differences(interner);
        assert_eq!(differences.len(), 1, "{differences:?}");
        assert!(differences[0].contains("now bind"), "{differences:?}");
    }

    #[test]
    fn reports_an_expression_whose_instantiation_bindings_were_removed() {
        let (mut context, expr_id) = context_with_instantiation_bindings();
        let interner = &mut context.def_interner;

        let check = PurityCheck::begin(interner);
        interner.restore_instantiation_bindings(expr_id, None);

        // Caught twice over: by name, and as `instantiation_bindings` losing an entry.
        let differences = check.differences(interner);
        assert_eq!(differences.len(), 2, "{differences:?}");
        assert!(
            differences
                .contains(&format!("the instantiation bindings of {expr_id:?} have been removed")),
            "{differences:?}"
        );
        assert!(
            differences.iter().any(|difference| difference.contains("losing entries")),
            "{differences:?}"
        );
    }

    #[test]
    fn reports_nothing_when_an_expression_is_put_back() {
        let (mut context, expr_id) = context_with_instantiation_bindings();
        let interner = &mut context.def_interner;

        let check = PurityCheck::begin(interner);
        let saved = interner.try_get_instantiation_bindings(expr_id).cloned();
        interner.store_instantiation_bindings(expr_id, TypeBindings::default());
        interner.restore_instantiation_bindings(expr_id, saved);

        assert_eq!(check.differences(interner), Vec::<String>::new());
    }

    /// Bindings stored against an expression pushed while the check was open are new state, not a
    /// change to what was already there.
    #[test]
    fn ignores_an_expression_pushed_while_the_check_was_open() {
        let (mut context, expr_id) = context_with_instantiation_bindings();
        let interner = &mut context.def_interner;
        let location = interner.expr_location(&expr_id);

        let check = PurityCheck::begin(interner);
        let pushed = interner.push_expr_full(
            HirExpression::Literal(crate::hir_def::expr::HirLiteral::Bool(true)),
            location,
            Type::Bool,
        );
        interner.store_instantiation_bindings(pushed, TypeBindings::default());

        assert_eq!(check.differences(interner), Vec::<String>::new());
    }

    #[test]
    fn reports_a_type_variable_left_bound() {
        let (mut context, _) = context_with_instantiation_bindings();
        let interner = &mut context.def_interner;
        let var = interner.next_type_variable_with_kind(crate::Kind::Normal);
        let Type::TypeVariable(var) = var else { panic!("expected a type variable") };

        let check = PurityCheck::begin(interner);
        crate::hir_def::types::BoundTypeVariables::bind(&var, Type::FieldElement).commit();

        let differences = check.differences(interner);
        assert_eq!(differences.len(), 1, "{differences:?}");
        assert!(differences[0].contains("is bound to `Field` now"), "{differences:?}");
    }

    /// Growth in state that is not keyed by something the pass just created is reported, whatever
    /// the field is — nobody has to have thought of it in advance.
    #[test]
    fn reports_an_insertion_into_state_that_is_not_append_only() {
        let (mut context, expr_id) = context_with_instantiation_bindings();
        let interner = &mut context.def_interner;

        let check = PurityCheck::begin(interner);
        interner.exprs_with_errors.insert(expr_id);

        let differences = check.differences(interner);
        assert_eq!(differences.len(), 1, "{differences:?}");
        assert!(differences[0].contains("`exprs_with_errors`"), "{differences:?}");
        assert!(differences[0].contains("may not change"), "{differences:?}");
    }
}
