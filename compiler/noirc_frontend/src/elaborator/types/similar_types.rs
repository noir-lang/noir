use crate::{DataType, Type, elaborator::Elaborator, hir::def_map::fully_qualified_module_path};

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

        match (actual, expected) {
            (Type::DataType(data_type_1, generics_1), Type::DataType(data_type_2, generics_2)) => {
                let data_type_1 = data_type_1.borrow();
                let data_type_2 = data_type_2.borrow();
                if data_type_1.id != data_type_2.id
                    && data_type_1.name.as_str() == data_type_2.name.as_str()
                {
                    let actual_fully_qualified_name =
                        data_type_fully_qualified_name(&data_type_1, self);
                    let expected_fully_qualified_name =
                        data_type_fully_qualified_name(&data_type_2, self);
                    similar_types
                        .push((actual_fully_qualified_name, expected_fully_qualified_name));
                }

                for (actual, expected) in generics_1.iter().zip(generics_2) {
                    self.compute_similar_types_helper(actual, &expected, similar_types);
                }
            }
            (Type::Tuple(types_1), Type::Tuple(types_2)) => {
                for (actual, expected) in types_1.iter().zip(types_2) {
                    self.compute_similar_types_helper(actual, &expected, similar_types);
                }
            }
            (Type::Array(_, type_1), Type::Array(_, type_2))
            | (Type::Vector(type_1), Type::Vector(type_2))
            | (Type::FmtString(_, type_1), Type::FmtString(_, type_2))
            | (Type::Reference(type_1, _), Type::Reference(type_2, _)) => {
                self.compute_similar_types_helper(&type_1, &type_2, similar_types);
            }
            (Type::Function(arg_1, ret_1, env_1, _), Type::Function(arg_2, ret_2, env_2, _)) => {
                for (actual, expected) in arg_1.iter().zip(arg_2) {
                    self.compute_similar_types_helper(actual, &expected, similar_types);
                }
                self.compute_similar_types_helper(&ret_1, &ret_2, similar_types);
                self.compute_similar_types_helper(&env_1, &env_2, similar_types);
            }
            _ => {}
        }
    }
}

fn data_type_fully_qualified_name(data_type: &DataType, elaborator: &Elaborator) -> String {
    let module_id = data_type.id.module_id();
    fully_qualified_module_path(
        elaborator.def_maps,
        elaborator.crate_graph,
        &elaborator.crate_id,
        module_id,
    )
}
