use rustc_hash::FxHashSet as HashSet;

use crate::{Kind, Type, TypeBinding, TypeBindings, TypeVariable, TypeVariableId};

/// The types the function being interpreted sees: the bindings of its generics (a call's
/// instantiation and impl bindings, or the ones a closure was created under) plus the types
/// solved while it runs, such as a macro call's result type. Every type the interpreter reads
/// from the HIR of that function goes through [`Frame::substitute`].
///
/// A value keeps the types it was built with, so a value built before one of its types was solved
/// only resolves against a frame that holds the solution. A frame therefore starts from its
/// caller's bindings, and on return hands back what it solved that its caller can see. See
/// `design/comptime.md`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct Frame {
    bindings: TypeBindings,
    /// The type variables in `bindings` that were solved while interpreting, rather than bound as
    /// generics. While there are none, every value's types are already as resolved as this frame
    /// can make them.
    runtime_solves: HashSet<TypeVariableId>,
}

impl Frame {
    pub(crate) fn bindings(&self) -> &TypeBindings {
        &self.bindings
    }

    pub(crate) fn has_runtime_solves(&self) -> bool {
        !self.runtime_solves.is_empty()
    }

    pub(crate) fn substitute(&self, typ: &Type) -> Type {
        typ.substitute(&self.bindings)
    }

    /// This frame (the caller's) extended into the frame of a call made from it. The callee's own
    /// instantiation and impl bindings take precedence, which keeps each recursive call's
    /// generics its own.
    pub(crate) fn for_call(mut self, own_bindings: &TypeBindings) -> Frame {
        for (var_id, binding) in own_bindings {
            self.bindings.insert(*var_id, binding.clone());
            self.runtime_solves.remove(var_id);
        }
        self
    }

    /// This frame (the caller's) extended into the frame of a call to a closure created in
    /// `created_in`, whose bindings take precedence.
    ///
    /// A type solved in `created_in` can have been bound on its variable since the closure was
    /// created (see `Interpreter::commit_final_solves`). The binding is then what every type
    /// sees, and a frame never also holds a replacement for a bound variable, so those solves are
    /// left out.
    pub(crate) fn for_closure(mut self, created_in: Frame) -> Frame {
        for (var_id, binding) in created_in.bindings {
            let is_runtime_solve = created_in.runtime_solves.contains(&var_id);
            if is_runtime_solve && matches!(binding.0.binding(), TypeBinding::Bound(_)) {
                continue;
            }
            self.runtime_solves.remove(&var_id);
            if is_runtime_solve {
                self.runtime_solves.insert(var_id);
            }
            self.bindings.insert(var_id, binding);
        }
        self
    }

    /// Records types solved while interpreting.
    pub(crate) fn solve(&mut self, bindings: TypeBindings) {
        self.runtime_solves.extend(bindings.keys().copied());
        self.bindings.extend(bindings);
    }

    /// Takes back solved types so that they can be solved again, possibly differently.
    pub(crate) fn forget(&mut self, var_ids: &[TypeVariableId]) {
        for var_id in var_ids {
            if self.runtime_solves.remove(var_id) {
                self.bindings.remove(var_id);
            }
        }
    }

    /// Removes the type variables solved while interpreting from this frame, returning each with
    /// its solution resolved against the frame.
    pub(crate) fn take_runtime_solutions(&mut self) -> Vec<(TypeVariable, Kind, Type)> {
        let solutions: Vec<_> = self
            .runtime_solves
            .iter()
            .map(|var_id| {
                let (var, kind, typ) = &self.bindings[var_id];
                (var.clone(), kind.clone(), self.substitute(typ))
            })
            .collect();
        for var_id in self.runtime_solves.drain() {
            self.bindings.remove(&var_id);
        }
        solutions
    }

    /// Copies into this frame each type variable `callee` solved that occurs in one of `visible`,
    /// and returns them. Solves `callee` inherited from this frame and still holds unchanged are
    /// skipped.
    ///
    /// A callee can only solve its own body's variables or ones that reached it through the types
    /// its caller can see, so `visible` is the types its generics were instantiated with, or a
    /// closure's own type (which includes its captures). A variable only the callee's body
    /// mentions stays behind, so a recursive call that solves it differently cannot overwrite its
    /// caller's solution.
    pub(crate) fn take_solves(&mut self, callee: &Frame, visible: &[&Type]) -> Vec<TypeVariableId> {
        let mut taken = Vec::new();
        for var_id in &callee.runtime_solves {
            let (var, kind, typ) = &callee.bindings[var_id];
            if self.bindings.get(var_id).is_some_and(|(_, _, own)| own == typ)
                || !visible.iter().any(|visible| visible.occurs(*var_id))
            {
                continue;
            }
            let typ = callee.substitute(typ);
            self.bindings.insert(*var_id, (var.clone(), kind.clone(), typ));
            self.runtime_solves.insert(*var_id);
            taken.push(*var_id);
        }
        taken
    }
}

#[cfg(test)]
mod tests {
    use crate::{Kind, Type, TypeBindings, TypeVariable, TypeVariableId};

    use super::Frame;

    fn binding(id: usize, typ: Type) -> TypeBindings {
        let id = TypeVariableId(id);
        let mut bindings = TypeBindings::default();
        bindings.insert(id, (TypeVariable::unbound(id, Kind::Normal), Kind::Normal, typ));
        bindings
    }

    #[test]
    fn forgetting_every_runtime_solve_leaves_none() {
        let mut frame = Frame::default();
        frame.solve(binding(0, Type::Bool));
        assert!(frame.has_runtime_solves());

        frame.forget(&[TypeVariableId(0)]);
        assert!(!frame.has_runtime_solves());
        assert!(frame.bindings().is_empty());
    }

    #[test]
    fn forgetting_a_generic_keeps_its_binding() {
        let mut frame = Frame::default().for_call(&binding(0, Type::Bool));
        frame.forget(&[TypeVariableId(0)]);
        assert!(!frame.has_runtime_solves());
        assert_eq!(frame.bindings().len(), 1);
    }

    #[test]
    fn a_closure_frame_leaves_out_solves_bound_since_it_was_created() {
        let solved = binding(0, Type::Bool);
        let mut created_in = Frame::default();
        created_in.solve(solved.clone());
        solved[&TypeVariableId(0)].0.bind(Type::Bool);

        let frame = Frame::default().for_closure(created_in);
        assert!(!frame.has_runtime_solves());
        assert!(frame.bindings().is_empty());
    }

    #[test]
    fn a_callees_generic_shadowing_a_solve_is_not_a_solve() {
        let mut caller = Frame::default();
        caller.solve(binding(0, Type::Bool));
        let callee = caller.for_call(&binding(0, Type::FieldElement));
        assert!(!callee.has_runtime_solves());
    }
}
