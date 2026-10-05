use rustc_hash::FxHashSet as HashSet;

use crate::{Type, TypeBindings, TypeVariableId};

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

    /// The frame of a call made from this one. The callee's own instantiation and impl bindings
    /// take precedence, which keeps each recursive call's generics its own.
    pub(crate) fn for_call(&self, own_bindings: &TypeBindings) -> Frame {
        let mut frame = self.clone();
        for (var_id, binding) in own_bindings {
            frame.bindings.insert(*var_id, binding.clone());
            frame.runtime_solves.remove(var_id);
        }
        frame
    }

    /// The frame of a call, made from this one, to a closure created in `created_in`, whose
    /// bindings take precedence.
    pub(crate) fn for_closure(&self, created_in: Frame) -> Frame {
        let mut frame = self.clone();
        for var_id in created_in.bindings.keys() {
            frame.runtime_solves.remove(var_id);
        }
        frame.bindings.extend(created_in.bindings);
        frame.runtime_solves.extend(created_in.runtime_solves);
        frame
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
    fn a_callees_generic_shadowing_a_solve_is_not_a_solve() {
        let mut caller = Frame::default();
        caller.solve(binding(0, Type::Bool));
        let callee = caller.for_call(&binding(0, Type::FieldElement));
        assert!(!callee.has_runtime_solves());
    }
}
