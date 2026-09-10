//! Recording of type variable writes, so that a pass can check it left the ones it touched the
//! way it found them.
//!
//! A `TypeVariable` holds its binding in an `Rc<RefCell<_>>` shared with every `Type` that
//! mentions it, including the types held by the `NodeInterner`. Writing one is therefore a
//! mutation of the elaborated program that a shared reference to the interner does not account
//! for, and one left behind is visible to every later compilation against that interner.
//!
//! Every such write in the compiler goes through `TypeVariable::bind`, `try_bind`, `replace` or
//! `restore`, which are the only methods with access to the cell. Recording at those four points
//! means the log sees every write, whatever the writer intended to do about undoing it — which is
//! what makes it able to check a pass rather than restate what the pass already believes about
//! itself.
//!
//! Recording only happens while a check is open, so code that never opens one pays a thread-local
//! read per write and nothing else. Whether to open one is the caller's decision, which is why
//! there is no switch here.

use std::cell::RefCell;

use rustc_hash::FxHashMap as HashMap;

use super::types::{TypeBinding, TypeVariable, TypeVariableId};

/// What each cell held before the first write one check saw to it.
///
/// Keyed by variable, so a cell written a thousand times costs one entry — and the entry that
/// survives is the earliest, which is the one a check has to compare against.
type Frame = HashMap<TypeVariableId, (TypeVariable, TypeBinding)>;

thread_local! {
    /// One frame per open check, innermost last. Empty when nothing is being checked, which is
    /// how [`record`] knows to do nothing.
    static FRAMES: RefCell<Vec<Frame>> = const { RefCell::new(Vec::new()) };
}

/// Note that `var` is about to be written, remembering what it holds now.
///
/// Called from the write methods of [`TypeVariable`] themselves, from the branch that goes on to
/// write — a method that declines to write must not record an entry, or a check would report a
/// cell it never touched.
pub(super) fn record(var: &TypeVariable) {
    FRAMES.with_borrow_mut(|frames| {
        let Some(frame) = frames.last_mut() else {
            return;
        };
        frame.entry(var.id()).or_insert_with(|| (var.clone(), var.borrow().clone()));
    });
}

/// An open check over the type variable writes made since it was started. Pass it to [`finish`]
/// to see which of them were not undone.
///
/// Closing a check is [`Drop`], so it happens on the way out of a panic as well as on the way out
/// of a return. A thread that left one open would record every write it went on to make, for the
/// rest of the process, into a frame nothing will ever read.
///
/// Checks have to be closed innermost first, which they are: the only way to hold one is as a
/// local and the only way to close one is to consume it.
#[must_use = "a check that is not held to the end of the work it covers records nothing"]
pub struct WriteCheck {
    /// A check carries no data — it *is* its frame's lifetime, and the frame lives in [`FRAMES`].
    /// It is a distinct type so that closing one can be `Drop`.
    _frame: (),
}

/// Start recording type variable writes.
pub fn begin() -> WriteCheck {
    FRAMES.with_borrow_mut(|frames| frames.push(Frame::default()));
    WriteCheck { _frame: () }
}

impl Drop for WriteCheck {
    fn drop(&mut self) {
        FRAMES.with_borrow_mut(|frames| {
            let Some(frame) = frames.pop() else {
                return;
            };

            // An enclosing check's window covers these writes too. Where it already has an entry
            // for a cell, that entry wins: it holds what the cell held before the enclosing check
            // began, which is further back than anything this one saw.
            if let Some(enclosing) = frames.last_mut() {
                for (id, entry) in frame {
                    enclosing.entry(id).or_insert(entry);
                }
            }
        });
    }
}

/// Stop recording, and describe every cell written since [`begin`] whose contents now differ from
/// what it held before the first of those writes.
///
/// An empty result means the writes in this window cancelled out: whatever ran inside it left the
/// type variables it touched exactly as it found them.
pub fn finish(check: WriteCheck) -> Vec<String> {
    let mut unrestored = FRAMES.with_borrow(|frames| {
        let frame = frames.last().expect("the check being finished is still open");
        frame
            .values()
            .filter_map(|(var, previous)| {
                let now = var.borrow();
                (*now != *previous).then(|| {
                    format!(
                        "type variable {} was {} before it was written and is {} now",
                        var.id().0,
                        describe(previous),
                        describe(&now)
                    )
                })
            })
            .collect::<Vec<_>>()
    });

    drop(check);

    unrestored.sort();
    unrestored
}

fn describe(binding: &TypeBinding) -> String {
    match binding {
        TypeBinding::Bound(typ) => format!("bound to `{typ}`"),
        TypeBinding::Unbound(..) => "unbound".to_string(),
    }
}

/// How many checks are open on this thread. Zero everywhere outside a check, including after one
/// that was dropped rather than finished.
#[cfg(test)]
fn open_checks() -> usize {
    FRAMES.with_borrow(Vec::len)
}

/// How many cells the innermost open check has recorded a write to.
#[cfg(test)]
fn recorded_cells() -> usize {
    FRAMES.with_borrow(|frames| frames.last().map_or(0, HashMap::len))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hir_def::types::{BoundGenerics, BoundTypeVariables, Kind, Type, TypeVariableId};

    fn unbound_variable(id: usize) -> TypeVariable {
        TypeVariable::unbound(TypeVariableId(id), Kind::Normal)
    }

    /// A set holding a single binding, so a test can put a variable into and out of force at
    /// points a guard's scope would not reach.
    fn binding_of(var: &TypeVariable, typ: Type) -> BoundGenerics {
        let mut generics = BoundGenerics::default();
        generics.remember(var, &typ, &Kind::Normal);
        generics
    }

    #[test]
    fn reports_a_binding_that_was_never_put_back() {
        let check = begin();
        let var = unbound_variable(0);

        BoundTypeVariables::bind(&var, Type::FieldElement).commit();

        let unrestored = finish(check);
        assert_eq!(
            unrestored,
            vec!["type variable 0 was unbound before it was written and is bound to `Field` now"]
        );
    }

    #[test]
    fn reports_nothing_when_a_guard_restores_the_binding() {
        let check = begin();
        let var = unbound_variable(1);

        drop(BoundTypeVariables::bind(&var, Type::FieldElement));

        assert_eq!(finish(check), Vec::<String>::new());
        assert!(var.borrow().is_unbound());
    }

    /// A cell written several times is compared against what it held before the first of them,
    /// not against the value it happened to hold in between, and costs one entry however many
    /// times it is written.
    #[test]
    fn compares_against_the_contents_before_the_first_write() {
        let check = begin();
        let var = unbound_variable(2);
        let generics = binding_of(&var, Type::FieldElement);

        generics.apply();
        BoundTypeVariables::bind(&var, Type::Bool).commit();
        generics.remove();

        assert_eq!(recorded_cells(), 1, "three writes to one cell should cost one entry");
        assert_eq!(finish(check), Vec::<String>::new());
    }

    /// A variable an outer scope had bound is restored to that binding, not to unbound. This is
    /// what a guard buys over reverting to `Unbound`, so the check has to be able to see it.
    #[test]
    fn reports_a_nested_binding_restored_to_unbound_instead_of_its_outer_value() {
        let outer = begin();
        let var = unbound_variable(3);
        let _outer_binding = BoundTypeVariables::bind(&var, Type::FieldElement);

        let inner = begin();
        let generics = binding_of(&var, Type::Bool);
        generics.apply();
        generics.remove();

        assert_eq!(
            finish(inner),
            vec!["type variable 3 was bound to `Field` before it was written and is unbound now"]
        );
        drop(finish(outer));
    }

    /// Writes an inner check saw and undid are invisible to the check enclosing it.
    #[test]
    fn an_enclosing_check_does_not_see_writes_a_nested_one_undid() {
        let outer = begin();
        let var = unbound_variable(4);

        let inner = begin();
        drop(BoundTypeVariables::bind(&var, Type::FieldElement));
        assert_eq!(finish(inner), Vec::<String>::new());

        assert_eq!(finish(outer), Vec::<String>::new());
    }

    /// A write an inner check left behind is still there when the enclosing check looks.
    #[test]
    fn an_enclosing_check_sees_a_write_a_nested_one_left_behind() {
        let outer = begin();
        let var = unbound_variable(5);

        let inner = begin();
        BoundTypeVariables::bind(&var, Type::FieldElement).commit();
        assert_eq!(finish(inner).len(), 1);

        assert_eq!(
            finish(outer),
            vec!["type variable 5 was unbound before it was written and is bound to `Field` now"]
        );
    }

    /// Closing a check has to happen on the way out of a panic as well as on the way out of a
    /// return. A thread that leaves one open records every write it goes on to make for the rest
    /// of the process, into a frame nothing will ever read — and monomorphization panicking is
    /// not hypothetical, it is what `nargo test` catches and reports as a failing test.
    #[test]
    fn a_check_that_is_dropped_rather_than_finished_stops_recording() {
        assert_eq!(open_checks(), 0);

        let dropped = begin();
        assert_eq!(open_checks(), 1);
        drop(dropped);
        assert_eq!(open_checks(), 0, "dropping a check left it open");

        let var = unbound_variable(6);
        BoundTypeVariables::bind(&var, Type::FieldElement).commit();

        let check = begin();
        assert_eq!(recorded_cells(), 0, "a write made with no check open was recorded");
        drop(finish(check));
    }
}
