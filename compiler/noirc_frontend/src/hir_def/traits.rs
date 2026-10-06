use iter_extended::vecmap;
use rustc_hash::FxHashMap as HashMap;
use std::rc::Rc;

use crate::ResolvedGeneric;
use crate::ast::{DocComment, Ident, ItemVisibility, NoirFunction};
use crate::elaborator::types::SELF_TYPE_NAME;
use crate::hir::type_check::generics::TraitGenerics;
use crate::node_interner::{
    DefinitionId, ImplSearchErrorKind, NodeInterner, TraitImplKind, TraitLookupMode,
};
use crate::{
    Kind, NamedGeneric, ResolvedGenerics, Type, TypeBinding, TypeBindings, TypeVariable,
    TypeVariableId,
    graph::CrateId,
    node_interner::{FuncId, TraitId},
};
use fm::FileId;
use noirc_errors::{Location, Span};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TraitFunction {
    pub name: Ident,
    pub typ: Type,
    pub location: Location,
    pub default_impl: Option<Box<NoirFunction>>,
    pub default_impl_module_id: crate::hir::def_map::LocalModuleId,
    pub trait_constraints: Vec<TraitConstraint>,
    pub direct_generics: ResolvedGenerics,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TraitConstant {
    pub name: Ident,
    pub typ: Type,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub struct NamedType {
    pub name: Ident,
    pub typ: Type,
}

impl std::fmt::Display for NamedType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} = {}", self.name, self.typ)
    }
}

/// Represents a trait in the type system. Each instance of this struct
/// will be shared across all `Type::Trait` variants that represent
/// the same trait.
#[derive(Debug, Eq)]
pub struct Trait {
    /// A unique id representing this trait type. Used to check if two
    /// struct traits are equal.
    pub id: TraitId,

    pub crate_id: CrateId,

    pub methods: Vec<TraitFunction>,

    /// Maps `method_name` -> method id.
    /// This map is separate from methods since `TraitFunction` ids
    /// are created during collection where we don't yet have all
    /// the information needed to create the full `TraitFunction`.
    pub method_ids: HashMap<String, FuncId>,

    /// Named generics of the trait.
    pub associated_types: ResolvedGenerics,
    pub associated_type_bounds: HashMap<String, Vec<DeclaredBound>>,

    pub name: Ident,
    /// Ordered generics of the trait.
    pub generics: ResolvedGenerics,
    pub location: Location,
    pub visibility: ItemVisibility,

    /// The `Self` that everything declared on this trait refers to.
    pub self_param: TraitSelfType,

    /// The trait's where clause, including its parent bounds. Use [`Trait::parent_bounds`] for
    /// the parent bounds alone.
    pub where_clause: DeclaredWhereClause,

    /// Bounds implied on associated types reached through this trait's own where clause.
    /// E.g. for `trait Baz<T> where T: Foo` with `trait Foo { type E: Bar; }`, this holds
    /// the constraint `<T as Foo>::E: Bar`. These are assumed when elaborating the trait's
    /// default method bodies, but are deliberately kept out of `where_clause` so they do not
    /// participate in trait-impl where-clause matching or trait-method signature matching.
    pub implicit_associated_type_constraints: Vec<(TraitConstraint, Location)>,

    pub all_generics: ResolvedGenerics,

    /// Map from each associated constant's name to a unique `DefinitionId` for that constant.
    pub associated_constant_ids: HashMap<String, DefinitionId>,
}

/// A completed trait implementation.
///
/// Note that ordered generics and named arguments (associated types) are stored separately
/// in the `NodeInterner`. This is because they're required to resolve types before the impl
/// as a whole is finished resolving.
#[derive(Debug)]
pub struct TraitImpl {
    pub ident: Ident,
    pub location: Location,
    pub typ: Type,
    pub trait_id: TraitId,

    pub file: FileId,
    pub crate_id: CrateId,
    pub methods: Vec<FuncId>, // methods[i] is the implementation of trait.methods[i] for Type typ

    /// The where clause, if present, contains each trait requirement which must
    /// be satisfied for this impl to be selected. E.g. in `impl Eq for [T] where T: Eq`,
    /// `where_clause` would contain the one `T: Eq` constraint. If there is no where clause,
    /// this Vec is empty.
    pub where_clause: Vec<TraitConstraint>,
}

/// A completed inherent `impl` block, i.e. one that does not implement a trait,
/// such as `impl<T> Foo<T> where T: Bar { ... }`.
#[derive(Debug)]
pub struct Impl {
    pub location: Location,
    pub typ: Type,

    pub file: FileId,
    pub crate_id: CrateId,
    pub module_id: crate::hir::def_map::ModuleId,

    /// The generics introduced by the impl block, in declaration order
    /// (e.g. `T` in `impl<T> Foo<T>`).
    pub generics: ResolvedGenerics,

    pub methods: Vec<FuncId>,

    /// The impl's where clause. Empty if there is no where clause.
    pub where_clause: Vec<TraitConstraint>,

    /// The doc comments written on top of the impl block. Empty if there are none.
    pub doc_comments: Vec<DocComment>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TraitConstraint {
    pub typ: Type,
    pub trait_bound: ResolvedTraitBound,
}

impl TraitConstraint {
    /// Update the type in the constraint by substituting the bindings onto it,
    /// then apply the bindings onto the trait bounds as well.
    pub fn apply_bindings(&mut self, type_bindings: &TypeBindings) {
        self.typ = self.typ.substitute(type_bindings);
        self.trait_bound.apply_bindings(type_bindings);
    }

    pub fn to_string(&self, interner: &NodeInterner) -> String {
        interner.trait_constraint_string(
            &self.typ,
            self.trait_bound.trait_id,
            &self.trait_bound.trait_generics.ordered,
            &self.trait_bound.trait_generics.named,
        )
    }

    /// Whether `self` and `other` denote the same bound, treating an unspecified associated
    /// type as matching any other unspecified one.
    ///
    /// This is weaker than equality, for comparing a constraint against a copy of it that was
    /// resolved independently — e.g. an inherent impl's where clause copied onto a method, or a
    /// trait's supertrait bound propagated onto a method. Each resolution fills in any
    /// associated type the bound leaves unspecified with a *fresh* type variable, so the copies
    /// aren't `==` even though they're the same bound. Associated types the user bound to a
    /// concrete type (`Foo<Bar = u32>`) are still compared, so those bounds aren't conflated.
    pub fn matches_ignoring_unspecified_associated_types(&self, other: &TraitConstraint) -> bool {
        if self.typ != other.typ
            || self.trait_bound.trait_id != other.trait_bound.trait_id
            || self.trait_bound.trait_generics.ordered != other.trait_bound.trait_generics.ordered
        {
            return false;
        }

        let self_named = &self.trait_bound.trait_generics.named;
        let other_named = &other.trait_bound.trait_generics.named;
        if self_named.len() != other_named.len() {
            return false;
        }

        // An associated type the bound leaves unspecified is filled in with a fresh type
        // variable each time the bound is resolved, so two copies of the same bound carry
        // different ones there. Depending on the resolution path that filler is a bare type
        // variable or a named generic, so accept either (when unbound) as matching, while still
        // comparing associated types the user bound to a concrete type (e.g. `Foo<Bar = u32>`).
        let is_unbound = |typ: &Type| match typ {
            Type::TypeVariable(var) | Type::NamedGeneric(NamedGeneric { type_var: var, .. }) => {
                var.binding().is_unbound()
            }
            _ => false,
        };
        self_named.iter().zip(other_named).all(|(a, b)| {
            a.name == b.name && (a.typ == b.typ || (is_unbound(&a.typ) && is_unbound(&b.typ)))
        })
    }

    /// Looks up a trait implementation which satisfies this constraint and returns it.
    ///
    /// Note that if successful, any type bindings from the impl search will be automatically
    /// applied, unless `current_trait_self` is `Some` type variable matching the constraint,
    /// representing a trait definition, in which case we don't bind it, but also don't reject
    /// it as a missing type.
    pub fn find_impl(
        &self,
        interner: &NodeInterner,
        current_trait_self: Option<&Type>,
    ) -> Result<(TraitImplKind, TypeBindings), ImplSearchErrorKind> {
        match current_trait_self {
            Some(typ) if *typ == self.typ && typ.is_bindable() => {
                let (impl_kind, _bindings, instantiation_bindings) = interner
                    .try_lookup_trait_implementation(
                        &self.typ,
                        self.trait_bound.trait_id,
                        &self.trait_bound.trait_generics.ordered,
                        &self.trait_bound.trait_generics.named,
                        TraitLookupMode::SelfAssumedOnly,
                    )?;
                // Do not bind it the self type.
                Ok((impl_kind, instantiation_bindings))
            }
            _ => interner.lookup_trait_implementation(
                &self.typ,
                self.trait_bound.trait_id,
                &self.trait_bound.trait_generics.ordered,
                &self.trait_bound.trait_generics.named,
            ),
        }
    }
}

#[derive(Debug, Clone, Eq)]
pub struct ResolvedTraitBound {
    pub trait_id: TraitId,
    pub trait_generics: TraitGenerics,
    pub location: Location,
}

/// A trait's `Self`: one type variable shared by everything declared on the trait.
///
/// The variable is never handed out in its bindable `Type::TypeVariable` form. Unifying that
/// form with a concrete type binds `Self` for every use of the trait in the whole program, so a
/// trait default body or an impl search could change which impl other code dispatches to.
/// Inside the trait, `Self` is rigid ([`TraitSelfType::rigid`], [`TraitSelfType::generic`]);
/// at a use of the trait it is substituted ([`TraitSelfType::bind`]).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TraitSelfType(TypeVariable);

impl TraitSelfType {
    pub fn new(id: TypeVariableId) -> Self {
        Self(TypeVariable::unbound(id, Kind::Normal))
    }

    pub fn id(&self) -> TypeVariableId {
        self.0.id()
    }

    /// `Self` as a named generic, which unifies with nothing but itself.
    pub fn rigid(&self) -> Type {
        let self_type_name = Rc::new(SELF_TYPE_NAME.to_string());
        self.0.clone().into_named_generic(&self_type_name, None)
    }

    /// `Self` as a generic to put in scope while resolving an item declared on the trait.
    pub fn generic(&self, location: Location) -> ResolvedGeneric {
        ResolvedGeneric {
            name: Rc::new(SELF_TYPE_NAME.to_string()),
            type_var: self.0.clone(),
            location,
        }
    }

    /// Substitute `self_type` for `Self` in anything declared on the trait.
    pub fn bind(&self, self_type: &Type, bindings: &mut TypeBindings) {
        let kind = self.0.kind().into_owned();
        bindings.insert(self.0.id(), (self.0.clone(), kind, self_type.clone()));
    }

    /// The type `Self` has been bound to. Always `None` unless something unified the trait's
    /// own `Self` with another type, which is a compiler bug.
    pub fn bound_to(&self) -> Option<Type> {
        match self.0.binding() {
            TypeBinding::Bound(typ) => Some(typ.clone()),
            TypeBinding::Unbound(..) => None,
        }
    }
}

/// A trait's where clause, as declared. Parent bounds (`trait Foo: Bar`) are lowered into it as
/// constraints on the trait's `Self`, so it holds both those and the other constraints of the
/// trait's `where` clause.
///
/// Like a [`DeclaredBound`], it is written in terms of the trait's own `Self`, generics and
/// associated items, and in terms of the placeholders its parent bounds put in for associated
/// items they leave out, all shared by every use of the trait. Outside the trait it means
/// something only once those are substituted, so its contents are reached through
/// [`DeclaredWhereClause::as_written`], whose callers substitute or only display.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DeclaredWhereClause(Vec<TraitConstraint>);

impl DeclaredWhereClause {
    /// The constraints exactly as declared, for display, for use inside the trait, or to be
    /// substituted by the caller.
    pub fn as_written(&self) -> &[TraitConstraint] {
        &self.0
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

/// A trait bound as declared on a trait: a parent bound (the `Bar<Self>` in
/// `trait Foo: Bar<Self>`) or a bound on one of its associated types (the `Baz<Self>` in
/// `trait Foo { type Out: Baz<Self>; }`).
///
/// It is written in terms of the declaring trait's own `Self`, generics and associated types,
/// which are shared by every use of that trait. It only describes a real bound once those are
/// replaced by the ones of a particular `T: Foo<..>`, so the declared form is not exposed except
/// for display: read it with [`DeclaredBound::instantiate`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeclaredBound {
    bound: ResolvedTraitBound,
    /// For each associated item of the bounded trait that the bound leaves out (the `Out` of
    /// `Bar` in `trait Foo: Bar`), its index in `bound.trait_generics.named` and the placeholder
    /// variable resolution put there. The placeholder is shared by every use of the declaring
    /// trait.
    left_out: Vec<(usize, TypeVariable)>,
}

impl DeclaredBound {
    /// Name resolution fills an associated item a bound leaves out with a fresh, unbound type
    /// variable, and nothing else puts a bare type variable in a declared bound: that is how the
    /// left-out items are told apart from the given ones.
    fn new(bound: ResolvedTraitBound) -> Self {
        let left_out = bound
            .trait_generics
            .named
            .iter()
            .enumerate()
            .filter_map(|(index, named)| match &named.typ {
                Type::TypeVariable(placeholder) => Some((index, placeholder.clone())),
                _ => None,
            })
            .collect();
        DeclaredBound { bound, left_out }
    }

    pub fn trait_id(&self) -> TraitId {
        self.bound.trait_id
    }

    /// The bound for one use of the declaring trait. `bindings` maps the declaring trait's
    /// `Self`, generics and associated types to that use's (see [`Trait::substitution_for_use`]).
    ///
    /// An associated item the bound leaves out (`trait Foo: Bar` where `Bar` has `type Out`) is
    /// a placeholder declared once for the whole trait; `fresh_placeholder` replaces it with one
    /// for this instantiation, so that resolving one use cannot bind it for every other.
    ///
    /// That replacement is an inference variable, not a rigid `<T as Bar>::Out`: an instantiated
    /// parent bound is matched against the bounds in scope or against an impl, and the variable is
    /// solved by that match. The rigid `<T as Bar>::Out` a scope assumes comes from the bound that
    /// declares it (see `collect_parent_associated_types`).
    pub fn instantiate(
        &self,
        bindings: &TypeBindings,
        mut fresh_placeholder: impl FnMut(Kind) -> Type,
    ) -> ResolvedTraitBound {
        let mut trait_generics = self.bound.trait_generics.map(|typ| typ.substitute(bindings));
        for (index, placeholder) in &self.left_out {
            trait_generics.named[*index].typ = fresh_placeholder(placeholder.kind().into_owned());
        }
        ResolvedTraitBound { trait_generics, ..self.bound }
    }

    /// Whether this bound leaves out the associated item `name` of the bounded trait (the `Out`
    /// of `Bar` in `trait Foo: Bar`), so that its value differs between instantiations.
    pub fn leaves_out(&self, name: &str) -> bool {
        self.left_out
            .iter()
            .any(|(index, _)| self.bound.trait_generics.named[*index].name.as_str() == name)
    }

    /// Binds each placeholder this bound declares for an associated item it leaves out to that
    /// item's value in `instantiated`, an instantiation of this bound. A signature on the
    /// declaring trait that names such an item (`Self::Out` in a method of `trait Foo: Bar`) reads
    /// the placeholder, so it needs these bindings to mean that use's `Out` rather than the one
    /// shared by every use of the trait.
    pub fn bind_placeholders(
        &self,
        instantiated: &ResolvedTraitBound,
        bindings: &mut TypeBindings,
    ) {
        for (index, placeholder) in &self.left_out {
            let declared = &self.bound.trait_generics.named[*index];
            let value = instantiated.trait_generics.named.iter().find(|n| n.name == declared.name);
            if let Some(value) = value {
                let kind = placeholder.kind().into_owned();
                bindings.insert(placeholder.id(), (placeholder.clone(), kind, value.typ.clone()));
            }
        }
    }

    /// The bound exactly as written in the trait declaration, mentioning the declaring trait's
    /// own `Self` and generics. Only for displaying the declaration.
    pub fn as_written(&self) -> &ResolvedTraitBound {
        &self.bound
    }
}

impl ResolvedTraitBound {
    /// Update all [Type]s in the bound generics by substituting some [`TypeBindings`] onto them.
    pub fn apply_bindings(&mut self, type_bindings: &TypeBindings) {
        for typ in &mut self.trait_generics.ordered {
            *typ = typ.substitute(type_bindings);
        }

        for named in &mut self.trait_generics.named {
            named.typ = named.typ.substitute(type_bindings);
        }
    }
}

impl PartialEq for ResolvedTraitBound {
    fn eq(&self, other: &Self) -> bool {
        // Location doesn't matter for equality
        self.trait_id == other.trait_id && self.trait_generics == other.trait_generics
    }
}

impl std::hash::Hash for Trait {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl PartialEq for Trait {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Trait {
    pub fn set_methods(&mut self, methods: Vec<TraitFunction>) {
        self.methods = methods;
    }

    pub fn set_where_clause(&mut self, where_clause: Vec<TraitConstraint>) {
        self.where_clause = DeclaredWhereClause(where_clause);
    }

    /// The parent-trait bounds of this trait (the `Bar` in `trait Foo: Bar`).
    ///
    /// Parent bounds are stored in `where_clause` as constraints whose `typ` is this
    /// trait's `Self` (see [`Self::is_self_type`]); this accessor filters them back out.
    pub fn parent_bounds(&self) -> impl Iterator<Item = DeclaredBound> + '_ {
        self.where_clause
            .0
            .iter()
            .filter(|c| self.is_self_type(&c.typ))
            .map(|c| DeclaredBound::new(c.trait_bound.clone()))
    }

    /// The substitution that gives something declared on this trait (a parent bound, a bound on
    /// one of its associated types, its where clause) its meaning for one use of the trait,
    /// `self_type: ThisTrait<generics>`. Without `self_type`, the trait's `Self` is left as is.
    ///
    /// A variable whose argument mentions the variable itself gets no entry, and an associated
    /// type that `generics` leaves out is bound to `Type::Error`.
    ///
    /// Panics if `generics` doesn't give exactly one argument per ordered generic, or names an
    /// associated type this trait doesn't have.
    pub fn substitution_for_use(
        &self,
        self_type: Option<&Type>,
        generics: &TraitGenerics,
    ) -> TypeBindings {
        let mut bindings = TypeBindings::default();
        if let Some(self_type) = self_type {
            self.self_param.bind(self_type, &mut bindings);
        }

        assert_eq!(
            self.generics.len(),
            generics.ordered.len(),
            "unexpected number of ordered generics"
        );
        for (param, arg) in self.generics.iter().zip(&generics.ordered) {
            bind_unless_recursive(param, arg, &mut bindings);
        }

        assert!(
            generics.named.len() <= self.associated_types.len(),
            "trait bound has more named generics than associated types"
        );
        for named in &generics.named {
            assert!(
                self.get_associated_type(named.name.as_str()).is_some(),
                "Expected to find associated type named {}",
                named.name
            );
        }
        for associated in &self.associated_types {
            let arg = generics.named.iter().find(|named| named.name.as_str() == *associated.name);
            let arg = arg.map_or(Type::Error, |named| named.typ.clone());
            bind_unless_recursive(associated, &arg, &mut bindings);
        }
        bindings
    }

    /// Binds this trait's `Self` to `self_type`, and each of its ordered generics and associated
    /// types to the argument `generics` gives for it, adding to `bindings`. Used to seed the
    /// instantiation of one of the trait's methods for one use, so that instantiating the
    /// method's type keeps each of the trait's variables pointing at that use's argument.
    ///
    /// Unlike [`Self::substitution_for_use`], every variable that `generics` gives an argument for
    /// gets an entry, even one mapped to itself (an assumed `Self: CurrentTrait` constraint's
    /// arguments are the trait's own variables): an entry tells instantiation to leave the
    /// variable alone rather than replace it with a fresh one. An associated type that `generics`
    /// leaves out gets no entry.
    pub fn bind_given_arguments(
        &self,
        self_type: &Type,
        generics: &TraitGenerics,
        bindings: &mut TypeBindings,
    ) {
        self.self_param.bind(self_type, bindings);
        for (param, arg) in self.generics.iter().zip(&generics.ordered) {
            bind(param, arg, bindings);
        }
        for associated in &self.associated_types {
            let arg = generics.named.iter().find(|named| named.name.as_str() == *associated.name);
            if let Some(arg) = arg {
                bind(associated, &arg.typ, bindings);
            }
        }
    }

    /// Whether `typ` is this trait's own (rigid) `Self`.
    pub fn is_self_type(&self, typ: &Type) -> bool {
        matches!(typ, Type::NamedGeneric(NamedGeneric { type_var, .. }) if type_var.id() == self.self_param.id())
    }

    /// The bounds declared on the associated type `name` (the `Baz` in `type Out: Baz;`).
    pub fn associated_type_bounds(&self, name: &str) -> &[DeclaredBound] {
        self.associated_type_bounds.get(name).map(Vec::as_slice).unwrap_or_default()
    }

    /// The bounds declared on all of this trait's associated types.
    pub fn all_associated_type_bounds(&self) -> impl Iterator<Item = &DeclaredBound> {
        self.associated_type_bounds.values().flatten()
    }

    /// Whether `bound`, declared on this trait, mentions this trait's own `Self`, generics or
    /// associated types, so that it differs between uses of the trait.
    pub fn bound_depends_on_use(&self, bound: &DeclaredBound) -> bool {
        let own = std::iter::once(self.self_param.id())
            .chain(self.generics.iter().chain(&self.associated_types).map(|g| g.type_var.id()))
            .collect::<Vec<_>>();
        let generics = &bound.bound.trait_generics;
        let types = generics.ordered.iter().chain(generics.named.iter().map(|named| &named.typ));
        types.into_iter().any(|typ| own.iter().any(|id| typ.occurs(*id)))
    }

    pub fn set_visibility(&mut self, visibility: ItemVisibility) {
        self.visibility = visibility;
    }

    pub fn set_all_generics(&mut self, generics: ResolvedGenerics) {
        self.all_generics = generics;
    }

    pub fn set_associated_type_bounds(
        &mut self,
        associated_type_bounds: HashMap<String, Vec<ResolvedTraitBound>>,
    ) {
        self.associated_type_bounds = associated_type_bounds
            .into_iter()
            .map(|(name, bounds)| (name, bounds.into_iter().map(DeclaredBound::new).collect()))
            .collect();
    }

    pub fn find_method(&self, name: &str, interner: &NodeInterner) -> Option<DefinitionId> {
        for method in &self.methods {
            if &method.name == name {
                let id = *self.method_ids.get(name).unwrap();
                return Some(interner.function_definition_id(id));
            }
        }
        None
    }

    pub fn find_method_or_constant(
        &self,
        name: &str,
        interner: &NodeInterner,
    ) -> Option<DefinitionId> {
        if let Some(method) = self.find_method(name, interner) {
            return Some(method);
        }
        self.associated_constant_ids.get(name).copied()
    }

    /// Find an associated type by the last segments in its name.
    ///
    /// For example if a method returns `Self::Foo`, here we will be looking for it by the name `"Foo"`.
    pub fn get_associated_type(&self, last_name: &str) -> Option<&ResolvedGeneric> {
        self.associated_types.iter().find(|typ| typ.name.as_ref() == last_name)
    }

    /// Returns both the ordered generics of this type, and its named, associated types.
    /// These types are all as-is and are not instantiated.
    pub fn get_generics(&self) -> (Vec<Type>, Vec<Type>) {
        let ordered = vecmap(&self.generics, |generic| generic.clone().into_named_generic(None));
        let named = vecmap(&self.associated_types, |generic| {
            generic.clone().into_named_generic(Some((SELF_TYPE_NAME, self.name.as_str())))
        });
        (ordered, named)
    }

    pub fn get_trait_generics(&self, location: Location) -> TraitGenerics {
        let ordered = vecmap(&self.generics, |generic| generic.clone().into_named_generic(None));
        let named = vecmap(&self.associated_types, |generic| {
            let name = Ident::new(generic.name.to_string(), location);
            NamedType {
                name,
                typ: generic.clone().into_named_generic(Some((SELF_TYPE_NAME, self.name.as_str()))),
            }
        });
        TraitGenerics { ordered, named }
    }

    /// Returns a `TraitConstraint` for this trait using Self as the object
    /// type and the uninstantiated generics for any trait generics.
    ///
    /// `Self` is the rigid named generic over the trait's `Self` variable, the same form trait method
    /// signatures use. A bindable `Type::TypeVariable` here would let a default body unify
    /// `Self` with a concrete type, and that binding would be seen by every other default
    /// method of the trait type-checked afterwards.
    pub fn as_constraint(&self, location: Location) -> TraitConstraint {
        let trait_generics = self.get_trait_generics(location);
        TraitConstraint {
            typ: self.self_type(),
            trait_bound: ResolvedTraitBound { trait_generics, trait_id: self.id, location },
        }
    }

    /// The rigid `Self` type for this trait (see [`TraitSelfType::rigid`]).
    pub fn self_type(&self) -> Type {
        self.self_param.rigid()
    }
}

impl std::fmt::Display for Trait {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl TraitFunction {
    pub fn arguments(&self) -> &[Type] {
        match &self.typ {
            Type::Function(args, _, _, _) => args,
            Type::Forall(_, typ) => match typ.as_ref() {
                Type::Function(args, _, _, _) => args,
                _ => unreachable!("Trait function does not have a function type"),
            },
            _ => unreachable!("Trait function does not have a function type"),
        }
    }

    pub fn generics(&self) -> &[TypeVariable] {
        match &self.typ {
            Type::Function(..) => &[],
            Type::Forall(generics, _) => generics,
            _ => unreachable!("Trait function does not have a function type"),
        }
    }

    pub fn return_type(&self) -> &Type {
        match &self.typ {
            Type::Function(_, return_type, _, _) => return_type,
            Type::Forall(_, typ) => match typ.as_ref() {
                Type::Function(_, return_type, _, _) => return_type,
                _ => unreachable!("Trait function does not have a function type"),
            },
            _ => unreachable!("Trait function does not have a function type"),
        }
    }
}

/// Binds `param`, one of a trait's own type variables, to `arg`.
fn bind(param: &ResolvedGeneric, arg: &Type, bindings: &mut TypeBindings) {
    let kind = param.kind().into_owned();
    bindings.insert(param.type_var.id(), (param.type_var.clone(), kind, arg.clone()));
}

/// Like [`bind`], but adds nothing when `arg` mentions `param` itself (such as `T ↦ T`).
fn bind_unless_recursive(param: &ResolvedGeneric, arg: &Type, bindings: &mut TypeBindings) {
    if !arg.occurs(param.type_var.id()) {
        bind(param, arg, bindings);
    }
}
