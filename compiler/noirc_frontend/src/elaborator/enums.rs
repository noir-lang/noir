use iter_extended::vecmap;
use noirc_errors::Location;

use crate::{
    ast::{EnumVariant, FunctionKind, NoirEnumeration, UnresolvedType, Visibility},
    hir_def::{
        expr::{HirEnumConstructorExpression, HirExpression, HirIdent},
        function::{FuncMeta, FunctionBody, HirFunction, Parameters},
        stmt::HirPattern,
    },
    node_interner::{DefinitionKind, FuncId, FunctionModifiers, TypeId},
    token::Attributes,
    DataType, Shared, Type,
};

use super::Elaborator;

impl Elaborator<'_> {
    #[allow(clippy::too_many_arguments)]
    pub(super) fn define_enum_variant_function(
        &mut self,
        enum_: &NoirEnumeration,
        type_id: TypeId,
        variant: &EnumVariant,
        variant_arg_types: Vec<Type>,
        variant_index: usize,
        datatype: &Shared<DataType>,
        self_type: &Type,
        self_type_unresolved: UnresolvedType,
    ) {
        let name_string = variant.name.to_string();
        let datatype_ref = datatype.borrow();
        let location = Location::new(variant.name.span(), self.file);

        let id = self.interner.push_empty_fn();

        let modifiers = FunctionModifiers {
            name: name_string.clone(),
            visibility: enum_.visibility,
            attributes: Attributes { function: None, secondary: Vec::new() },
            is_unconstrained: false,
            generic_count: datatype_ref.generics.len(),
            is_comptime: false,
            name_location: location,
        };
        let definition_id =
            self.interner.push_function_definition(id, modifiers, type_id.module_id(), location);

        let hir_name = HirIdent::non_trait_method(definition_id, location);
        let parameters = self.make_enum_variant_parameters(variant_arg_types, location);
        self.push_enum_variant_function_body(id, datatype, variant_index, &parameters, location);

        let function_type =
            datatype_ref.variant_function_type_with_forall(variant_index, datatype.clone());
        self.interner.push_definition_type(definition_id, function_type.clone());

        let meta = FuncMeta {
            name: hir_name,
            kind: FunctionKind::Normal,
            parameters,
            parameter_idents: Vec::new(),
            return_type: crate::ast::FunctionReturnType::Ty(self_type_unresolved),
            return_visibility: Visibility::Private,
            typ: function_type,
            direct_generics: datatype_ref.generics.clone(),
            all_generics: datatype_ref.generics.clone(),
            location,
            has_body: false,
            trait_constraints: Vec::new(),
            type_id: Some(type_id),
            trait_id: None,
            trait_impl: None,
            enum_variant_index: Some(variant_index),
            is_entry_point: false,
            has_inline_attribute: false,
            function_body: FunctionBody::Resolved,
            source_crate: self.crate_id,
            source_module: type_id.local_module_id(),
            source_file: self.file,
            self_type: None,
        };

        self.interner.push_fn_meta(meta, id);
        self.interner.add_method(self_type, name_string, id, None);

        let name = variant.name.clone();
        Self::get_module_mut(self.def_maps, type_id.module_id())
            .declare_function(name, enum_.visibility, id)
            .ok();
    }

    // Given:
    // ```
    // enum FooEnum { Foo(u32, u8), ... }
    //
    // fn Foo(a: u32, b: u8) -> FooEnum {}
    // ```
    // Create (pseudocode):
    // ```
    // fn Foo(a: u32, b: u8) -> FooEnum {
    //     // This can't actually be written directly in Noir
    //     FooEnum {
    //         tag: Foo_tag,
    //         Foo: (a, b),
    //         // fields from other variants are zeroed in monomorphization
    //     }
    // }
    // ```
    fn push_enum_variant_function_body(
        &mut self,
        id: FuncId,
        self_type: &Shared<DataType>,
        variant_index: usize,
        parameters: &Parameters,
        location: Location,
    ) {
        // Each parameter of the enum variant function is used as a parameter of the enum
        // constructor expression
        let arguments = vecmap(&parameters.0, |(pattern, typ, _)| match pattern {
            HirPattern::Identifier(ident) => {
                let id = self.interner.push_expr(HirExpression::Ident(ident.clone(), None));
                self.interner.push_expr_type(id, typ.clone());
                self.interner.push_expr_location(id, location.span, location.file);
                id
            }
            _ => unreachable!(),
        });

        let enum_generics = self_type.borrow().generic_types();
        let construct_variant = HirExpression::EnumConstructor(HirEnumConstructorExpression {
            r#type: self_type.clone(),
            enum_generics: enum_generics.clone(),
            arguments,
            variant_index,
        });
        let body = self.interner.push_expr(construct_variant);
        self.interner.update_fn(id, HirFunction::unchecked_from_expr(body));

        let typ = Type::DataType(self_type.clone(), enum_generics);
        self.interner.push_expr_type(body, typ);
        self.interner.push_expr_location(body, location.span, location.file);
    }

    fn make_enum_variant_parameters(
        &mut self,
        parameter_types: Vec<Type>,
        location: Location,
    ) -> Parameters {
        Parameters(vecmap(parameter_types.into_iter().enumerate(), |(i, parameter_type)| {
            let name = format!("${i}");
            let parameter = DefinitionKind::Local(None);
            let id = self.interner.push_definition(name, false, false, parameter, location);
            let pattern = HirPattern::Identifier(HirIdent::non_trait_method(id, location));
            (pattern, parameter_type, Visibility::Private)
        }))
    }
}
