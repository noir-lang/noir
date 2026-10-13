use crate::{Substitution, Type, TypeBindings, TypeVariableId};

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
    /// The bindings of generics, one layer per call or closure this frame was extended into, each
    /// written in terms of the layers before it.
    layers: Vec<TypeBindings>,
    /// The type variables solved while interpreting, rather than bound as generics, written in
    /// terms of the whole frame. While there are none, every value's types are already as
    /// resolved as this frame can make them.
    solves: TypeBindings,
    /// `layers` and `solves` combined, which is what types are read through.
    substitution: Substitution,
}

impl Frame {
    pub(crate) fn bindings(&self) -> &TypeBindings {
        self.substitution.bindings()
    }

    pub(crate) fn has_runtime_solves(&self) -> bool {
        !self.solves.is_empty()
    }

    pub(crate) fn substitute(&self, typ: &Type) -> Type {
        self.substitution.apply(typ)
    }

    fn rebuild(&mut self) {
        let mut substitution = Substitution::default();
        for layer in &self.layers {
            // Layers are never taken back from a frame: a call or closure extends a copy.
            let _ = substitution.push_layer(layer.clone());
        }
        substitution.solve(self.solves.clone());
        self.substitution = substitution;
    }

    /// Adds `layer`, generics bound in terms of this frame, overriding solves of the same
    /// variables.
    fn push_layer(&mut self, layer: TypeBindings) {
        for var_id in layer.keys() {
            self.solves.remove(var_id);
        }
        self.layers.push(layer);
        self.rebuild();
    }

    /// This frame (the caller's) extended into the frame of a call made from it. The callee's own
    /// instantiation and impl bindings take precedence, which keeps each recursive call's
    /// generics its own.
    pub(crate) fn for_call(mut self, own_bindings: &TypeBindings) -> Frame {
        self.push_layer(own_bindings.clone());
        self
    }

    /// This frame (the caller's) extended into the frame of a call to a closure created in
    /// `created_in`, whose bindings take precedence.
    pub(crate) fn for_closure(mut self, created_in: Frame) -> Frame {
        let mut generics = created_in.substitution.bindings().clone();
        generics.retain(|var_id, _| !created_in.solves.contains_key(var_id));
        for var_id in created_in.solves.keys() {
            self.solves.remove(var_id);
        }
        self.solves.extend(created_in.solves);
        self.push_layer(generics);
        self
    }

    /// Records types solved while interpreting.
    pub(crate) fn solve(&mut self, bindings: TypeBindings) {
        self.solves.extend(bindings);
        self.rebuild();
    }

    /// Takes back solved types so that they can be solved again, possibly differently.
    pub(crate) fn forget(&mut self, var_ids: &[TypeVariableId]) {
        let before = self.solves.len();
        for var_id in var_ids {
            self.solves.remove(var_id);
        }
        if self.solves.len() != before {
            self.rebuild();
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
        for (var_id, (var, kind, typ)) in &callee.solves {
            if self.solves.get(var_id).is_some_and(|(_, _, own)| own == typ)
                || !visible.iter().any(|visible| visible.occurs(*var_id))
            {
                continue;
            }
            let typ = callee.substitute(typ);
            self.solves.insert(*var_id, (var.clone(), kind.clone(), typ));
            taken.push(*var_id);
        }
        if !taken.is_empty() {
            self.rebuild();
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
