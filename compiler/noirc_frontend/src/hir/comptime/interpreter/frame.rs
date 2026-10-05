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
    /// Whether `bindings` holds a type solved while interpreting, rather than only generics.
    /// Until one does, every value's types are already as resolved as this frame can make them.
    has_runtime_solves: bool,
}

impl Frame {
    pub(crate) fn bindings(&self) -> &TypeBindings {
        &self.bindings
    }

    pub(crate) fn has_runtime_solves(&self) -> bool {
        self.has_runtime_solves
    }

    pub(crate) fn substitute(&self, typ: &Type) -> Type {
        typ.substitute(&self.bindings)
    }

    /// The frame of a call made from this one. The callee's own instantiation and impl bindings
    /// take precedence, which keeps each recursive call's generics its own.
    pub(crate) fn for_call(&self, own_bindings: &TypeBindings) -> Frame {
        let mut frame = self.clone();
        frame.bindings.extend(own_bindings.iter().map(|(id, binding)| (*id, binding.clone())));
        frame
    }

    /// The frame of a call, made from this one, to a closure created in `created_in`, whose
    /// bindings take precedence.
    pub(crate) fn for_closure(&self, created_in: Frame) -> Frame {
        let mut frame = self.clone();
        frame.bindings.extend(created_in.bindings);
        frame.has_runtime_solves |= created_in.has_runtime_solves;
        frame
    }

    /// Records types solved while interpreting.
    pub(crate) fn solve(&mut self, bindings: TypeBindings) {
        self.has_runtime_solves |= !bindings.is_empty();
        self.bindings.extend(bindings);
    }

    /// Takes back solved types so that they can be solved again, possibly differently.
    pub(crate) fn forget(&mut self, var_ids: &[TypeVariableId]) {
        for var_id in var_ids {
            self.bindings.remove(var_id);
        }
    }

    /// Copies into this frame each type variable `callee` solved that occurs in one of `visible`,
    /// and returns them. A binding `callee` holds that this frame does not hold identically was
    /// solved by `callee`; its own generics (`callee_generics`) are never handed back.
    ///
    /// A callee can only solve its own body's variables or ones that reached it through the types
    /// its caller can see, so `visible` is the types its generics were instantiated with, or a
    /// closure's own type (which includes its captures). A variable only the callee's body
    /// mentions stays behind, so a recursive call that solves it differently cannot overwrite its
    /// caller's solution.
    pub(crate) fn take_solves(
        &mut self,
        callee: &Frame,
        visible: &[&Type],
        callee_generics: &TypeBindings,
    ) -> Vec<TypeVariableId> {
        let mut taken = Vec::new();
        for (var_id, (var, kind, typ)) in &callee.bindings {
            if callee_generics.contains_key(var_id)
                || self.bindings.get(var_id).is_some_and(|(_, _, own)| own == typ)
                || !visible.iter().any(|visible| visible.occurs(*var_id))
            {
                continue;
            }
            let typ = callee.substitute(typ);
            self.bindings.insert(*var_id, (var.clone(), kind.clone(), typ));
            self.has_runtime_solves = true;
            taken.push(*var_id);
        }
        taken
    }
}
