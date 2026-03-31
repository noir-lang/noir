use crate::{
    DataType, Shared, Type, elaborator::Elaborator, hir::def_map::fully_qualified_module_path,
};

impl Elaborator<'_> {
    /// Computes a list of types that appear in `actual` and `expected` that have the same base name but
    /// are actually different types. For example, given two types `foo::Bar` and `baz::Bar`, if a function
    /// is expected to take `foo::Bar` but is given `baz::Bar`, this function will return `[("foo::Bar", "baz::Bar")]`.
    /// The check is done recursively, so if the function is expected to take `Generic<foo::Bar>` but is given
    /// `Generic<baz::Bar>`, this function will also return `[("foo::Bar", "baz::Bar")]`.
    pub(super) fn compute_similar_types(
        &self,
        actual: &Type,
        expected: &Type,
    ) -> Vec<(String, String)> {
        let mut similar_types = Vec::new();
        self.compute_similar_types_helper(actual, expected, &mut similar_types);
        similar_types
    }

    fn compute_similar_types_helper(
        &self,
        actual: &Type,
        expected: &Type,
        similar_types: &mut Vec<(String, String)>,
    ) {
        let actual = actual.follow_bindings();
        let expected = expected.follow_bindings();

        let Some((actual, actual_generics)) = extract_data_type(actual) else {
            return;
        };
        let Some((expected, expected_generics)) = extract_data_type(expected) else {
            return;
        };

        if actual != expected && actual.borrow().name.as_str() == expected.borrow().name.as_str() {
            let actual_fully_qualified_name = data_type_fully_qualified_name(actual, self);
            let expected_fully_qualified_name = data_type_fully_qualified_name(expected, self);
            similar_types.push((actual_fully_qualified_name, expected_fully_qualified_name));
        }

        for (actual, expected) in actual_generics.iter().zip(expected_generics) {
            self.compute_similar_types_helper(actual, &expected, similar_types);
        }
    }
}

fn extract_data_type(typ: Type) -> Option<(Shared<DataType>, Vec<Type>)> {
    match typ {
        Type::DataType(data_type, generics) => Some((data_type, generics)),
        _ => None,
    }
}

fn data_type_fully_qualified_name(data_type: Shared<DataType>, elaborator: &Elaborator) -> String {
    let module_id = data_type.borrow().id.module_id();
    fully_qualified_module_path(
        elaborator.def_maps,
        elaborator.crate_graph,
        &elaborator.crate_id,
        module_id,
    )
}
