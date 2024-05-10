use std::collections::BTreeMap;

use crate::{node_interner::TraitId, hir::def_collector::dc_crate::UnresolvedTrait};

use super::Elaborator;


impl<'context> Elaborator<'context> {
    pub fn collect_trait(&mut self, traits: BTreeMap<TraitId, UnresolvedTrait>) {
        for (trait_id, unresolved_trait) in &traits {
            self.interner.push_empty_trait(*trait_id, unresolved_trait);
        }
        let mut all_errors = Vec::new();

        for (trait_id, unresolved_trait) in traits {
            let generics = vecmap(&unresolved_trait.trait_def.generics, |_| {
                TypeVariable::unbound(self.interner.next_type_variable_id())
            });

            // Resolve order
            // 1. Trait Types ( Trait constants can have a trait type, therefore types before constants)
            let _ = self.resolve_trait_types(&unresolved_trait);
            // 2. Trait Constants ( Trait's methods can use trait types & constants, therefore they should be after)
            let _ = self.resolve_trait_constants(&unresolved_trait);
            // 3. Trait Methods
            let (methods, errors) =
                self.resolve_trait_methods(trait_id, &unresolved_trait, &generics);

            all_errors.extend(errors);

            self.interner.update_trait(trait_id, |trait_def| {
                trait_def.set_methods(methods);
                trait_def.generics = generics;
            });

            // This check needs to be after the trait's methods are set since
            // the interner may set `interner.ordering_type` based on the result type
            // of the Cmp trait, if this is it.
            if self.crate_id.is_stdlib() {
                self.interner.try_add_operator_trait(trait_id);
            }
        }
        all_errors
    }
}
