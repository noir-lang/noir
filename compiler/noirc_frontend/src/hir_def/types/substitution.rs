use crate::{Kind, Type, TypeBindings, TypeVariable, TypeVariableId};

/// Bindings built up in layers, each written in terms of the layers before it: the bindings of
/// an impl's generics, then those of a method of that impl, whose types can mention the impl's
/// generics. Each layer is translated through the ones before it when it is added, so every
/// replacement is final and [`Substitution::apply`] is a single [`Type::substitute`].
///
/// Translating first is what keeps a layer's types in its caller's terms: in the bindings for
/// `UInt::<B, 32>::width()` called inside `impl<N, B> UInt<N, B>`, `N` is bound to the caller's
/// `B`, which the layers before already bind, so `N` never becomes the `32` the callee's `B` is
/// bound to.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Substitution {
    bindings: TypeBindings,
}

/// The entries a [`Substitution::push_layer`] replaced, for [`Substitution::pop_layer`].
#[must_use]
pub struct PushedLayer(Vec<(TypeVariableId, Option<Binding>)>);

type Binding = (TypeVariable, Kind, Type);

impl Substitution {
    pub fn apply(&self, typ: &Type) -> Type {
        typ.substitute(&self.bindings)
    }

    /// The bindings, each one final.
    pub fn bindings(&self) -> &TypeBindings {
        &self.bindings
    }

    pub fn is_empty(&self) -> bool {
        self.bindings.is_empty()
    }

    /// Adds `layer`, whose types are written in terms of this substitution, and returns what it
    /// replaced.
    ///
    /// The layer's types are first translated through this substitution. What they still mention
    /// of the variables the layer binds can then only be those variables themselves, as when
    /// unification solves `?a := [?b]` and `?b := u8`, so the layer is then applied to itself.
    /// The types already here are written in terms of outer layers, which don't mention the
    /// variables `layer` binds, so they stay as they are.
    pub fn push_layer(&mut self, layer: TypeBindings) -> PushedLayer {
        let layer = layer
            .into_iter()
            .map(|(id, (var, kind, typ))| (id, (var, kind, self.apply(&typ))))
            .collect();
        let layer = resolve_layer(layer);
        PushedLayer(
            layer
                .into_iter()
                .map(|(id, binding)| (id, self.bindings.insert(id, binding)))
                .collect(),
        )
    }

    /// Takes back a layer added by [`Self::push_layer`], restoring what it replaced. Layers are
    /// taken back in the reverse order they were added.
    pub fn pop_layer(&mut self, pushed: PushedLayer) {
        for (id, previous) in pushed.0.into_iter().rev() {
            match previous {
                Some(binding) => self.bindings.insert(id, binding),
                None => self.bindings.remove(&id),
            };
        }
    }

    /// Adds solutions for type variables that the types already here may contain, which a layer
    /// can't do: the solutions are added as a layer and then applied to the types already here.
    pub fn solve(&mut self, solved: TypeBindings) {
        let PushedLayer(added) = self.push_layer(solved);
        let solved: TypeBindings =
            added.into_iter().map(|(id, _)| (id, self.bindings[&id].clone())).collect();
        for (id, (_, _, typ)) in &mut self.bindings {
            if !solved.contains_key(id) {
                *typ = typ.substitute(&solved);
            }
        }
    }
}

/// Applies a layer to itself until none of its types mentions a variable it binds.
fn resolve_layer(mut layer: TypeBindings) -> TypeBindings {
    // No variable is bound in terms of itself, so each round makes progress and a chain is at
    // most as long as the layer.
    for _ in 0..layer.len() {
        let next: TypeBindings = layer
            .iter()
            .map(|(id, (var, kind, typ))| {
                (*id, (var.clone(), kind.clone(), typ.substitute(&layer)))
            })
            .collect();
        if next == layer {
            break;
        }
        layer = next;
    }
    layer
}
