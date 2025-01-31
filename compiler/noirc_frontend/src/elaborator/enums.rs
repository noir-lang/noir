use acvm::AcirField;
use iter_extended::vecmap;
use noirc_errors::Location;

use crate::{
    ast::{EnumVariant, Expression, FunctionKind, NoirEnumeration, UnresolvedType, Visibility},
    hir::resolution::errors::ResolverError,
    hir_def::{
        expr::{
            HirArrayLiteral, HirEnumConstructorExpression, HirExpression, HirIdent, HirLiteral,
            HirMatchExpression,
        },
        function::{FuncMeta, FunctionBody, HirFunction, Parameters},
        stmt::HirPattern,
    },
    node_interner::{DefinitionKind, ExprId, FunctionModifiers, GlobalValue, TypeId},
    token::Attributes,
    DataType, Shared, Type,
};

use super::Elaborator;

impl Elaborator<'_> {
    /// Defines the value of an enum variant that we resolve an enum
    /// variant expression to. E.g. `Foo::Bar` in `Foo::Bar(baz)`.
    ///
    /// If the variant requires arguments we should define a function,
    /// otherwise we define a polymorphic global containing the tag value.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn define_enum_variant_constructor(
        &mut self,
        enum_: &NoirEnumeration,
        type_id: TypeId,
        variant: &EnumVariant,
        variant_arg_types: Option<Vec<Type>>,
        variant_index: usize,
        datatype: &Shared<DataType>,
        self_type: &Type,
        self_type_unresolved: UnresolvedType,
    ) {
        match variant_arg_types {
            Some(args) => self.define_enum_variant_function(
                enum_,
                type_id,
                variant,
                args,
                variant_index,
                datatype,
                self_type,
                self_type_unresolved,
            ),
            None => self.define_enum_variant_global(
                enum_,
                type_id,
                variant,
                variant_index,
                datatype,
                self_type,
            ),
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn define_enum_variant_global(
        &mut self,
        enum_: &NoirEnumeration,
        type_id: TypeId,
        variant: &EnumVariant,
        variant_index: usize,
        datatype: &Shared<DataType>,
        self_type: &Type,
    ) {
        let name = &variant.name;
        let location = Location::new(variant.name.span(), self.file);

        let global_id = self.interner.push_empty_global(
            name.clone(),
            type_id.local_module_id(),
            type_id.krate(),
            self.file,
            Vec::new(),
            false,
            false,
        );

        let mut typ = self_type.clone();
        if !datatype.borrow().generics.is_empty() {
            let typevars = vecmap(&datatype.borrow().generics, |generic| generic.type_var.clone());
            typ = Type::Forall(typevars, Box::new(typ));
        }

        let definition_id = self.interner.get_global(global_id).definition_id;
        self.interner.push_definition_type(definition_id, typ.clone());

        let no_parameters = Parameters(Vec::new());
        let global_body =
            self.make_enum_variant_constructor(datatype, variant_index, &no_parameters, location);
        let let_statement = crate::hir_def::stmt::HirStatement::Expression(global_body);

        let statement_id = self.interner.get_global(global_id).let_statement;
        self.interner.replace_statement(statement_id, let_statement);

        self.interner.get_global_mut(global_id).value = GlobalValue::Resolved(
            crate::hir::comptime::Value::Enum(variant_index, Vec::new(), typ),
        );

        Self::get_module_mut(self.def_maps, type_id.module_id())
            .declare_global(name.clone(), enum_.visibility, global_id)
            .ok();
    }

    #[allow(clippy::too_many_arguments)]
    fn define_enum_variant_function(
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

        let body =
            self.make_enum_variant_constructor(datatype, variant_index, &parameters, location);
        self.interner.update_fn(id, HirFunction::unchecked_from_expr(body));

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
    fn make_enum_variant_constructor(
        &mut self,
        self_type: &Shared<DataType>,
        variant_index: usize,
        parameters: &Parameters,
        location: Location,
    ) -> ExprId {
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

        let constructor = HirExpression::EnumConstructor(HirEnumConstructorExpression {
            r#type: self_type.clone(),
            arguments,
            variant_index,
        });

        let body = self.interner.push_expr(constructor);
        let enum_generics = self_type.borrow().generic_types();
        let typ = Type::DataType(self_type.clone(), enum_generics);
        self.interner.push_expr_type(body, typ);
        self.interner.push_expr_location(body, location.span, location.file);
        body
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

    /// This is an implementation of https://julesjacobs.com/notes/patternmatching/patternmatching.pdf
    pub(super) fn elaborate_match_rules(
        &mut self,
        value_to_match: ExprId,
        mut rules: MatchRules,
    ) -> Result<HirExpression, ResolverError> {
        self.push_tests_against_bare_variables(&mut rules);
        let case = self.select_case(&rules)?;
        self.generate_match_for_case(value_to_match, case, rules)
    }

    pub(super) fn push_tests_against_bare_variables(&mut self, _rules: &mut MatchRules) {}

    pub(super) fn select_case(&mut self, rules: &MatchRules) -> Result<usize, ResolverError> {
        // Start by always selecting the first case.
        // TODO: Use heuristic instead
        match rules.first() {
            Some((pattern, _)) => pattern.get_variant_index(self),
            None => Ok(0),
        }
    }

    /// Generate:
    /// case a {
    ///     C(a0, .., aN) -> <A>
    ///     _             -> <B>
    /// }
    /// where:
    ///   C == case == variant_index
    ///   a0, .., aN = arguments to constructor C
    ///   <A> = sub problem A
    ///   <B> = sub problem B
    pub(super) fn generate_match_for_case(
        &mut self,
        case: usize,
        rules: MatchRules,
    ) -> Result<(), ResolverError> {
        let mut sub_problem_a = Vec::new();
        let mut sub_problem_b = Vec::new();

        for (pattern, branch) in rules {
            let pattern_expr = self.interner.expression(&pattern);

            if pattern_expr.is_match_all() {
                sub_problem_a.push((pattern, branch.clone()));
                sub_problem_b.push((pattern, branch));
            } else if pattern.get_variant_index(self)? == case {
                sub_problem_a.push((pattern, branch));
            } else {
                sub_problem_b.push((pattern, branch));
            }
        }

        let rule_branch = self.elaborate_match_rules(value_to_match, sub_problem_a);
        let default_branch = self.elaborate_match_rules(value_to_match, sub_problem_b);

        Ok(HirExpression::Match(HirMatchExpression {
            value_to_match,
            rule_tag: case,
            rule_branch: default_branch,
        }))
    }
}

type MatchRules = Vec<(ExprId, Expression)>;

impl HirExpression {
    pub(crate) fn is_match_all(&self) -> bool {
        matches!(self, HirExpression::Ident(_, _))
    }
}

impl ExprId {
    pub(crate) fn get_variant_index(
        &self,
        elaborator: &mut Elaborator,
    ) -> Result<usize, ResolverError> {
        Ok(match elaborator.interner.expression(self) {
            HirExpression::Ident(_, _) => 0, // unreachable?
            HirExpression::Literal(literal) => match literal {
                HirLiteral::Array(_) => 0,

                // We're considering each slice length to be its own constuctor
                // to let users match on different lengths of the slice.
                HirLiteral::Slice(array) => match array {
                    HirArrayLiteral::Standard(elems) => elems.len(),
                    HirArrayLiteral::Repeated { length, .. } => {
                        let span = elaborator.interner.expr_span(self);
                        match length.evaluate_to_u32(span) {
                            Ok(length) => length as usize,
                            Err(error) => {
                                elaborator.push_err(error);
                                0
                            }
                        }
                    }
                },
                HirLiteral::Bool(value) => value as usize,
                HirLiteral::Integer(element, is_negative) => {
                    // We can't represent all Field values unfortunately so we'll have to error
                    // that we can't match on fields that are too big or small.
                    // Currently this is arbitrarily any field that doesn't fit into a u32.
                    if !is_negative {
                        if let Some(Ok(value)) = element.try_to_u64().map(TryInto::try_into) {
                            return Ok(value);
                        }
                    }
                    let span = elaborator.interner.expr_span(self);
                    return Err(ResolverError::IntegerPatternTooLarge { span });
                }
                HirLiteral::Unit => 0,

                // Str and FmtStr matching is a bit more difficult since we can't compare integers.
                // In the future we could do it with a full Eq call but we'll just error for now.
                HirLiteral::Str(_) | HirLiteral::FmtStr(..) => {
                    let span = elaborator.interner.expr_span(self);
                    return Err(ResolverError::CantMatchOnStrings { span });
                }
            },

            // These can resolve to enums, we'll need to check the metadata
            HirExpression::Call(expr) => return expr.func.get_variant_index(elaborator),

            // Enum constructors are usually represented by call expressions
            // or globals. We'll only get this variant if we're looking at the
            // value of a global enum or the body of an auto-generated enum
            // constructor function.
            HirExpression::EnumConstructor(expr) => expr.variant_index,

            // Struct & tuple types only have 1 constructor
            HirExpression::Tuple(_) => 0,
            HirExpression::Constructor(_) => 0,

            // None of these are valid pattern syntax
            HirExpression::Block(_)
            | HirExpression::Prefix(_)
            | HirExpression::Infix(_)
            | HirExpression::Index(_)
            | HirExpression::MemberAccess(_)
            | HirExpression::MethodCall(_)
            | HirExpression::Cast(_)
            | HirExpression::If(_)
            | HirExpression::Lambda(_)
            | HirExpression::Quote(_)
            | HirExpression::Unquote(_)
            | HirExpression::Comptime(_)
            | HirExpression::Unsafe(_) => {
                // TODO: error here
                0
            }
            HirExpression::Error => 0,
        })
    }
}
