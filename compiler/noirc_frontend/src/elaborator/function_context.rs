//! Function-local context management for type variables and trait constraints.

use noirc_errors::Location;

use crate::{
    Kind, Type, TypeBindings,
    elaborator::lints::check_integer_literal_fits_its_type,
    hir::{
        comptime::Value,
        type_check::{NoMatchingImplFoundError, TypeCheckError},
    },
    hir_def::traits::TraitConstraint,
    node_interner::{
        DefinitionKind, ExprId, GlobalValue, ImplSearchErrorKind, TraitImplKind, TypeId,
    },
};
use crate::{TypeVariableId, node_interner::DefinitionId};

use super::Elaborator;

#[derive(Default)]
pub(super) struct FunctionContext {
    /// All type variables created in the current function.
    /// This map is used to default any integer type variables at the end of
    /// a function (before checking trait constraints) if a type wasn't already chosen.
    defaultable_type_variables: Vec<Type>,

    /// Type variables that must be bound at the end of the function.
    required_type_variables: Vec<RequiredTypeVariable>,

    /// Trait constraints are collected during type checking until they are
    /// verified at the end of a function. This is because constraints arise
    /// on each variable, but it is only until function calls when the types
    /// needed for the trait constraint may become known.
    /// The `select impl` bool indicates whether, after verifying the trait constraint,
    /// the resulting trait impl should be the one used for a call (sometimes trait
    /// constraints are verified but there's no call associated with them, like in the
    /// case of checking generic arguments)
    trait_constraints: Vec<LocalTraitConstraint>,

    /// All ExprId in a function that correspond to integer literals.
    /// At the end, if they don't fit in their type's min/max range, we'll produce an error.
    integer_literal_expr_ids: Vec<ExprId>,
}

/// A type variable that is required to be bound after type-checking a function.
#[derive(Debug)]
struct RequiredTypeVariable {
    type_variable_id: TypeVariableId,
    typ: Type,
    kind: BindableTypeVariableKind,
    location: Location,
}

/// A constraint local to the current [FunctionContext] to solve at the end of the context.
#[derive(Debug)]
struct LocalTraitConstraint {
    constraint: TraitConstraint,

    /// The expression this constraint originated from. Used for its location in error messages,
    /// and if `select_impl` is true, to be the expression the impl will replace during
    /// monomorphization.
    expr: ExprId,

    /// True if a reference to some function in this constraint's trait should replace the
    /// expression at `self.expr` during monomorphization.
    select_impl: bool,
}

/// The kind of required type variable.
#[derive(Debug, Copy, Clone)]
pub(super) enum BindableTypeVariableKind {
    /// The type variable corresponds to a struct generic, in a constructor.
    StructGeneric { struct_id: TypeId, index: usize },
    /// The type variable corresponds to the type of an array literal.
    ArrayLiteral { is_array: bool },
    /// The type variable corresponds to an identifier, whose definition ID is the given one.
    Ident(DefinitionId),
}

impl Elaborator<'_> {
    /// Push a type variable into the current FunctionContext to be defaulted if needed
    /// at the end of the earlier of either the current function or the current comptime scope.
    pub(super) fn push_defaultable_type_variable(&mut self, typ: Type) {
        self.get_function_context_mut().defaultable_type_variables.push(typ);
    }

    /// Push a type variable (its ID and type) as a required type variable: it must be
    /// bound after type-checking the current function.
    pub(super) fn push_required_type_variable(
        &mut self,
        type_variable_id: TypeVariableId,
        typ: Type,
        kind: BindableTypeVariableKind,
        location: Location,
    ) {
        let var = RequiredTypeVariable { type_variable_id, typ, kind, location };
        self.get_function_context_mut().required_type_variables.push(var);
    }

    /// Push a trait constraint into the current FunctionContext to be solved if needed
    /// at the end of the earlier of either the current function or the current comptime scope.
    pub(super) fn push_trait_constraint(
        &mut self,
        constraint: TraitConstraint,
        expr: ExprId,
        select_impl: bool,
    ) {
        self.get_function_context_mut().trait_constraints.push(LocalTraitConstraint {
            constraint,
            expr,
            select_impl,
        });
    }

    /// Push an `ExprId` that corresponds to an integer literal.
    /// At the end of the current function we'll check that they fit in their type's range.
    pub fn push_integer_literal_expr_id(&mut self, literal_expr_id: ExprId) {
        self.get_function_context_mut().integer_literal_expr_ids.push(literal_expr_id);
    }

    fn get_function_context_mut(&mut self) -> &mut FunctionContext {
        let context = self.function_context.last_mut();
        context.expect("The function_context stack should always be non-empty")
    }

    pub(super) fn push_function_context(&mut self) {
        self.function_context.push(FunctionContext::default());
    }

    /// Defaults all type variables used in this function context then solves
    /// all still-unsolved trait constraints in this context.
    pub(super) fn check_and_pop_function_context(&mut self) {
        let context = self.function_context.pop().expect("Imbalanced function_context pushes");
        self.check_defaultable_type_variables(context.defaultable_type_variables);
        self.check_integer_literal_fit_their_type(context.integer_literal_expr_ids);
        self.check_trait_constraints(context.trait_constraints);
        self.check_required_type_variables(context.required_type_variables);
    }

    fn check_defaultable_type_variables(&self, type_variables: Vec<Type>) {
        for typ in type_variables {
            if let Type::TypeVariable(variable) = typ.follow_bindings() {
                let msg = "TypeChecker should only track defaultable type vars";
                variable.bind(variable.kind().default_type().expect(msg));
            }
        }
    }

    fn check_integer_literal_fit_their_type(&mut self, expr_ids: Vec<ExprId>) {
        for expr_id in expr_ids {
            if let Some(error) = check_integer_literal_fits_its_type(self.interner, &expr_id) {
                self.push_err(error);
            }
        }
    }

    fn check_trait_constraints(&mut self, trait_constraints: Vec<LocalTraitConstraint>) {
        for local in trait_constraints {
            match local.constraint.find_impl(self.interner) {
                Ok((impl_kind, instantiation_bindings)) => {
                    if local.select_impl {
                        self.select_impl(local.expr, impl_kind, instantiation_bindings);
                    }
                }
                Err(error) => {
                    let location = self.interner.expr_location(&local.expr);
                    self.push_trait_constraint_error(&local.constraint.typ, error, location);
                }
            }
        }
    }

    fn select_impl(
        &mut self,
        function_ident_id: ExprId,
        impl_kind: TraitImplKind,
        instantiation_bindings: TypeBindings,
    ) {
        // Insert any additional instantiation bindings into this expression's
        // instantiation bindings. We should avoid doing this if `select_impl` is
        // not true since that means we're not solving for this expressions exact
        // impl anyway. If we ignore this, we may rarely overwrite existing type
        // bindings causing incorrect types. The `vector_regex` test is one example
        // of that happening without this being behind `select_impl`.
        let mut bindings = self.interner.get_instantiation_bindings(function_ident_id).clone();

        // These can clash in the `vector_regex` test which causes us to insert
        // incorrect type bindings if they override the previous bindings.
        for (id, binding) in instantiation_bindings {
            let existing = bindings.insert(id, binding.clone());

            if let Some((_, type_var, existing)) = existing {
                let existing = existing.follow_bindings();
                let new = binding.2.follow_bindings();

                // Exact equality on types is intentional here, we never want to
                // overwrite even type variables but should probably avoid a panic if
                // the types are exactly the same.
                if existing != new {
                    panic!(
                        "Overwriting an existing type binding with a different type!\n  {type_var:?} <- {existing:?}\n  {type_var:?} <- {new:?}"
                    );
                }
            }
        }

        self.interner.store_instantiation_bindings(function_ident_id, bindings);
        self.interner.select_impl_for_expression(function_ident_id, impl_kind);
    }

    pub(super) fn push_trait_constraint_error(
        &mut self,
        object_type: &Type,
        error: ImplSearchErrorKind,
        location: Location,
    ) {
        match error {
            ImplSearchErrorKind::TypeAnnotationsNeededOnObjectType => {
                self.push_err(TypeCheckError::TypeAnnotationsNeededForMethodCall { location });
            }
            ImplSearchErrorKind::Nested(constraints) => {
                if let Some(error) =
                    NoMatchingImplFoundError::new(self.interner, constraints, location)
                {
                    self.push_err(TypeCheckError::NoMatchingImplFound(error));
                }
            }
            ImplSearchErrorKind::MultipleMatching(candidates) => {
                let object_type = object_type.clone();
                let err =
                    TypeCheckError::MultipleMatchingImpls { object_type, location, candidates };
                self.push_err(err);
            }
        }
    }

    fn check_required_type_variables(&mut self, type_variables: Vec<RequiredTypeVariable>) {
        for var in type_variables {
            let id = var.type_variable_id;
            if let Type::TypeVariable(_) = var.typ.follow_bindings() {
                let location = var.location;
                match var.kind {
                    BindableTypeVariableKind::StructGeneric { struct_id, index } => {
                        let data_type = self.interner.get_type(struct_id);
                        let generic = &data_type.borrow().generics[index];
                        let generic_name = generic.name.to_string();
                        let item_kind = "struct";
                        let item_name = data_type.borrow().name.to_string();
                        let is_numeric = matches!(generic.type_var.kind(), Kind::Numeric(..));
                        self.push_err(TypeCheckError::TypeAnnotationNeededOnItem {
                            location,
                            generic_name,
                            item_kind,
                            item_name,
                            is_numeric,
                        });
                    }
                    BindableTypeVariableKind::ArrayLiteral { is_array } => {
                        self.push_err(TypeCheckError::TypeAnnotationNeededOnArrayLiteral {
                            location,
                            is_array,
                        });
                    }
                    BindableTypeVariableKind::Ident(definition_id) => {
                        let definition = self.interner.definition(definition_id);
                        let definition_kind = definition.kind.clone();
                        match definition_kind {
                            DefinitionKind::Function(func_id) => {
                                // Try to find the type variable in the function's generic arguments
                                let mut direct_generics =
                                    self.interner.function_meta(&func_id).direct_generics.iter();
                                let generic =
                                    direct_generics.find(|generic| generic.type_var.id() == id);
                                if let Some(generic) = generic {
                                    let item_name =
                                        self.interner.definition_name(definition_id).to_string();
                                    let is_numeric =
                                        matches!(generic.type_var.kind(), Kind::Numeric(..));
                                    self.push_err(TypeCheckError::TypeAnnotationNeededOnItem {
                                        location,
                                        generic_name: generic.name.to_string(),
                                        item_kind: "function",
                                        item_name,
                                        is_numeric,
                                    });
                                    continue;
                                }

                                // If we find one in `all_generics` it means it's a generic on the type
                                // the function is in.
                                let Some(Type::DataType(typ, ..)) =
                                    &self.interner.function_meta(&func_id).self_type
                                else {
                                    continue;
                                };
                                let typ = typ.borrow();
                                let item_name = typ.name.to_string();
                                let item_kind = if typ.is_struct() { "struct" } else { "enum" };
                                drop(typ);

                                let mut all_generics =
                                    self.interner.function_meta(&func_id).all_generics.iter();
                                let generic =
                                    all_generics.find(|generic| generic.type_var.id() == id);
                                if let Some(generic) = generic {
                                    let is_numeric =
                                        matches!(generic.type_var.kind(), Kind::Numeric(..));
                                    self.push_err(TypeCheckError::TypeAnnotationNeededOnItem {
                                        location,
                                        generic_name: generic.name.to_string(),
                                        item_kind,
                                        item_name: item_name.clone(),
                                        is_numeric,
                                    });
                                }
                            }
                            DefinitionKind::Global(global_id) => {
                                // Check if this global points to an enum variant, then get the enum's generics
                                // and find the type variable there.
                                let global = self.interner.get_global(global_id);
                                let GlobalValue::Resolved(Value::Enum(_, _, Type::Forall(_, typ))) =
                                    &global.value
                                else {
                                    continue;
                                };

                                let typ: &Type = typ;
                                let Type::DataType(def, _) = typ else {
                                    continue;
                                };

                                let def = def.borrow();
                                let item_name = def.name.to_string();
                                let mut generics = def.generics.iter();
                                let generic =
                                    generics.find(|generic| generic.type_var.id() == id).cloned();
                                drop(def);
                                if let Some(generic) = generic {
                                    let is_numeric =
                                        matches!(generic.type_var.kind(), Kind::Numeric(..));
                                    self.push_err(TypeCheckError::TypeAnnotationNeededOnItem {
                                        location,
                                        generic_name: generic.name.to_string(),
                                        item_kind: "enum",
                                        item_name: item_name.clone(),
                                        is_numeric,
                                    });
                                }
                            }
                            _ => (),
                        }
                    }
                }
            }
        }
    }
}
