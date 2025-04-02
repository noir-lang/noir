#![allow(unused)] // TODO(#7879): Remove when done.
use acir::FieldElement;
use nargo::errors::Location;
use std::{collections::HashSet, fmt::Debug};
use strum::IntoEnumIterator;

use arbitrary::Unstructured;
use noirc_frontend::{
    ast::IntegerBitSize,
    hir_def::{self, expr::HirIdent, stmt::HirPattern},
    monomorphization::ast::{
        ArrayLiteral, Expression, FuncId, GlobalId, Index, InlineType, Literal, LocalId,
        Parameters, Type,
    },
    node_interner::DefinitionId,
    shared::{Signedness, Visibility},
};

use super::{Context, Name, VariableId, expr};

/// Something akin to a forward declaration of a function, capturing the details required to:
/// 1. call the function from the other function bodies
/// 2. generate the final HIR function signature
pub(super) struct FunctionDeclaration {
    pub name: String,
    pub params: Parameters,
    pub param_visibilities: Vec<Visibility>,
    pub return_type: Type,
    pub return_visibility: Visibility,
    pub inline_type: InlineType,
    pub unconstrained: bool,
}

impl FunctionDeclaration {
    /// Generate a HIR function signature.
    pub fn signature(&self) -> hir_def::function::FunctionSignature {
        let param_types = self
            .params
            .iter()
            .zip(self.param_visibilities.iter())
            .map(|((_id, mutable, _name, typ), vis)| {
                // The pattern doesn't seem to be used in `ssa::create_program`,
                // apart from its location, so it shouldn't matter what we put into it.
                let mut pat = HirPattern::Identifier(HirIdent {
                    location: Location::dummy(),
                    id: DefinitionId::dummy_id(),
                    impl_kind: hir_def::expr::ImplKind::NotATraitMethod,
                });
                if *mutable {
                    pat = HirPattern::Mutable(Box::new(pat), Location::dummy());
                }

                let typ = to_hir_type(typ);

                (pat, typ, *vis)
            })
            .collect();

        let return_type = (self.return_type != Type::Unit).then(|| to_hir_type(&self.return_type));

        (param_types, return_type)
    }
}

/// A layer of variables available to choose from in blocks.
#[derive(Debug, Clone)]
struct Scope<K: Ord> {
    /// ID and type of variables created in all visible scopes,
    /// which includes this scope and its ancestors.
    variables: im::OrdMap<K, (Name, Type)>,
    /// Reverse index of variables which can produce a type.
    /// For example an `(u8, [u64; 4])` can produce the tuple itself,
    /// the array in it, and both primitive types.
    producers: im::OrdMap<Type, im::OrdSet<K>>,
}

impl<K> Scope<K>
where
    K: Ord + Clone + Copy + Debug,
{
    /// Create the initial scope from function parameters.
    fn new(vars: impl Iterator<Item = (K, Name, Type)>) -> Self {
        let mut scope = Self { variables: im::OrdMap::new(), producers: im::OrdMap::new() };
        for (id, name, typ) in vars {
            scope.add(id, name, typ);
        }
        scope
    }

    /// Add a new variable to the scope.
    fn add(&mut self, id: K, name: String, typ: Type) {
        for typ in types_produced(&typ) {
            self.producers.entry(typ).or_default().insert(id);
        }
        self.variables.insert(id, (name, typ));
    }

    /// Get a variable in scope.
    fn get_variable(&self, id: &K) -> &(Name, Type) {
        self.variables.get(id).unwrap_or_else(|| panic!("variable doesn't exist: {:?}", id))
    }

    /// Choose a random producer of a type, if there is one.
    fn choose_producer(&self, u: &mut Unstructured, typ: &Type) -> arbitrary::Result<Option<K>> {
        let Some(vs) = self.producers.get(typ) else {
            return Ok(None);
        };
        u.choose_iter(vs.iter()).map(Some).map(|v| v.cloned())
    }
}

/// Context used during the generation of a function body.
pub(super) struct FunctionContext<'a> {
    /// Top level context, to access global variables and other functions.
    ctx: &'a Context,
    /// Declaration of this function.
    decl: &'a FunctionDeclaration,
    /// Self ID.
    id: FuncId,
    /// Every variable created in the function will have an increasing ID,
    /// which does not reset when variables go out of scope.
    next_local_id: u32,
    /// Global variables.
    globals: Scope<GlobalId>,
    /// Variables accumulated during the generation of the function body,
    /// initially consisting of the function parameters, then extended
    /// by locally defined variables. Block scopes add and remove layers.
    locals: Vec<Scope<LocalId>>,
}

impl<'a> FunctionContext<'a> {
    pub fn new(ctx: &'a Context, id: FuncId) -> Self {
        let decl = ctx.function_decl(id);
        let next_local_id = decl.params.iter().map(|p| p.0.0).max().unwrap_or_default();

        let globals = Scope::new(
            ctx.globals.iter().map(|(id, (name, typ, _expr))| (*id, name.clone(), typ.clone())),
        );

        let locals = Scope::new(
            decl.params.iter().map(|(id, _, name, typ)| (*id, name.clone(), typ.clone())),
        );

        Self { ctx, decl, id, next_local_id, globals, locals: vec![locals] }
    }

    /// Generate the function body.
    pub fn gen_body(mut self, u: &mut Unstructured) -> arbitrary::Result<Expression> {
        self.gen_expr(u, &self.decl.return_type, self.ctx.config.max_depth)
    }

    /// Local variables currently in scope.
    fn current_scope(&self) -> &Scope<LocalId> {
        self.locals.last().expect("there is always the params layer")
    }

    /// Add a layer of block variables.
    fn enter_scope(&mut self) {
        // Instead of shallow cloning an immutable map, we could loop through layers when looking up variables.
        self.locals.push(self.current_scope().clone());
    }

    /// Remove the last layer of block variables.
    fn exit_scope(&mut self) {
        self.locals.pop();
        assert!(!self.locals.is_empty(), "never pop the params layer");
    }

    /// Get and increment the next local ID.
    fn next_local_id(&mut self) -> LocalId {
        let id = LocalId(self.next_local_id);
        self.next_local_id += 1;
        id
    }

    /// Choose a producer for a type, preferring local variables over global ones.
    fn choose_producer(
        &self,
        u: &mut Unstructured,
        typ: &Type,
    ) -> arbitrary::Result<Option<VariableId>> {
        if u.ratio(7, 10)? {
            if let Some(id) = self.current_scope().choose_producer(u, typ)? {
                return Ok(Some(VariableId::Local(id)));
            }
        }
        self.globals.choose_producer(u, typ).map(|id| id.map(VariableId::Global))
    }

    /// Get a local or global variable.
    ///
    /// Panics if it doesn't exist.
    fn get_variable(&self, id: &VariableId) -> &(Name, Type) {
        match id {
            VariableId::Local(id) => self.current_scope().get_variable(id),
            VariableId::Global(id) => self.globals.get_variable(id),
        }
    }

    /// Generate an expression of a certain type.
    ///
    /// While doing so, enter and exit blocks, and add variables declared to the context,
    /// so expressions down the line can refer to earlier variables.
    fn gen_expr(
        &mut self,
        u: &mut Unstructured,
        typ: &Type,
        max_depth: usize,
    ) -> arbitrary::Result<Expression> {
        let i = u.choose_index(100)?;

        if i < 75 {
            if let Some(expr) = self.gen_expr_from_vars(u, typ, max_depth)? {
                return Ok(expr);
            }
        }

        // If nothing else worked out we can always produce a random literal.
        expr::gen_literal(u, typ)
    }

    /// Try to generate an expression with a certain type out of the variables in scope.
    fn gen_expr_from_vars(
        &mut self,
        u: &mut Unstructured,
        typ: &Type,
        max_depth: usize,
    ) -> arbitrary::Result<Option<Expression>> {
        if let Some(id) = self.choose_producer(u, typ)? {
            let (src_name, src_type) = self.get_variable(&id).clone();
            let src_expr = expr::ident(id, src_name, src_type.clone());
            if let Some(expr) = self.gen_expr_from_source(u, src_expr, &src_type, typ, max_depth)? {
                return Ok(Some(expr));
            }
        } else {
            // If we can't produce the exact we're looking for, maybe we can produce parts of it.
            match typ {
                Type::Array(len, item_type) => {
                    let mut arr = ArrayLiteral { contents: Vec::new(), typ: typ.clone() };
                    for _ in 0..*len {
                        arr.contents.push(self.gen_expr(u, item_type, max_depth)?);
                    }
                    return Ok(Some(Expression::Literal(Literal::Array(arr))));
                }
                Type::Tuple(items) => {
                    let mut values = Vec::new();
                    for item_type in items {
                        values.push(self.gen_expr(u, item_type, max_depth)?);
                    }
                    return Ok(Some(Expression::Tuple(values)));
                }
                _ => {}
            }
        }
        Ok(None)
    }

    /// Try to generate an expression that produces a target type from a source,
    /// e.g. given a source type of `[(u32, bool); 4]` and a target of `u64`
    /// it might generate `my_var[2].0 as u64`.
    ///
    /// Returns `None` if there is no way to produce the target from the source.
    fn gen_expr_from_source(
        &mut self,
        u: &mut Unstructured,
        src_expr: Expression,
        src_type: &Type,
        tgt_type: &Type,
        max_depth: usize,
    ) -> arbitrary::Result<Option<Expression>> {
        // If we found our type, return it without further ado.
        if src_type == tgt_type {
            return Ok(Some(src_expr));
        }

        // Cast the source into the target type.
        let src_as_tgt = || Ok(Some(expr::cast(src_expr.clone(), tgt_type.clone())));

        // See how we can produce tgt from src.
        match (src_type, tgt_type) {
            (
                Type::Field,
                Type::Integer(Signedness::Unsigned, IntegerBitSize::HundredTwentyEight),
            ) => src_as_tgt(),
            (Type::Bool, Type::Field) => src_as_tgt(),
            (Type::Integer(Signedness::Unsigned, _), Type::Field) => src_as_tgt(),
            (Type::Integer(sign_from, ibs_from), Type::Integer(sign_to, ibs_to))
                if sign_from == sign_to && ibs_from.bit_size() < ibs_to.bit_size() =>
            {
                src_as_tgt()
            }
            (Type::Reference(typ, _), _) if typ.as_ref() == tgt_type => {
                Ok(Some(Expression::Clone(Box::new(src_expr))))
            }
            (Type::Array(len, item_typ), _) => {
                // Choose a random index.
                let idx_expr = self.gen_index(u, *len as usize, max_depth)?;
                // Access the item.
                let item_expr = Expression::Index(Index {
                    collection: Box::new(src_expr),
                    index: Box::new(idx_expr),
                    element_type: *item_typ.clone(),
                    location: Location::dummy(),
                });
                // Produce the target type from the item.
                self.gen_expr_from_source(u, item_expr, item_typ, tgt_type, max_depth)
            }
            (Type::Tuple(items), _) => {
                // Any of the items might be able to produce the target type.
                let mut opts = Vec::new();
                for (i, item_type) in items.iter().enumerate() {
                    let item_expr = Expression::ExtractTupleField(Box::new(src_expr.clone()), i);
                    if let Some(expr) =
                        self.gen_expr_from_source(u, item_expr, item_type, tgt_type, max_depth)?
                    {
                        opts.push(expr);
                    }
                }
                if opts.is_empty() { Ok(None) } else { Ok(Some(u.choose_iter(opts)?)) }
            }
            (Type::Slice(_), _) => {
                // TODO: We don't know the length of the slice at compile time,
                // so we need to call the builtin function to get its length,
                // generate a random number here, and take its modulo:
                //      let idx = u32::arbitrary(u)?;
                //      let len_expr = ???;
                //      let idx_expr = expr::modulo(expr::u32_literal(idx), len_expr);
                // For now return nothing.
                Ok(None)
            }
            _ => {
                // We have already considered the case when the two types equal.
                // Normally we would call this function knowing that source can produce the target,
                // but in case we missed a case, let's return None and let the caller fall back to
                // a different strategy. In some cases we could return a literal, but it wouldn't
                // work in the recursive case of producing a type from an array item, which needs
                // to be wrapped with an accessor.
                Ok(None)
            }
        }
    }

    /// Generate an arbitrary index for an array.
    ///
    /// This can be either a random int literal, or a complex expression that produces an int.
    fn gen_index(
        &mut self,
        u: &mut Unstructured,
        len: usize,
        max_depth: usize,
    ) -> arbitrary::Result<Expression> {
        if max_depth > 0 && u.ratio(1, 3)? {
            let idx = self.gen_expr(u, &expr::u32_type(), max_depth - 1)?;
            Ok(expr::index_modulo(idx, len))
        } else {
            let idx = u.choose_index(len)?;
            Ok(expr::u32_literal(idx as u32))
        }
    }
}

fn to_hir_type(typ: &Type) -> hir_def::types::Type {
    use hir_def::types::{Kind as HirKind, Type as HirType};

    // Meet the expectations of `Type::evaluate_to_u32`.
    let size_const = |size: u32| {
        Box::new(HirType::Constant(
            FieldElement::from(size),
            HirKind::Numeric(Box::new(HirType::Integer(
                Signedness::Unsigned,
                IntegerBitSize::ThirtyTwo,
            ))),
        ))
    };

    match typ {
        Type::Unit => HirType::Unit,
        Type::Bool => HirType::Bool,
        Type::Field => HirType::FieldElement,
        Type::Integer(signedness, integer_bit_size) => {
            HirType::Integer(*signedness, *integer_bit_size)
        }
        Type::String(size) => HirType::String(size_const(*size)),
        Type::Array(size, typ) => HirType::Array(size_const(*size), Box::new(to_hir_type(typ))),
        Type::Tuple(items) => HirType::Tuple(items.iter().map(to_hir_type).collect()),
        Type::FmtString(_, _)
        | Type::Slice(_)
        | Type::Reference(_, _)
        | Type::Function(_, _, _, _) => {
            unreachable!("unexpected type converting to HIR: {}", typ)
        }
    }
}

/// Collect all the sub-types produced by a type.
///
/// It's like a _power set_ of the type.
fn types_produced(typ: &Type) -> HashSet<Type> {
    /// Recursively visit subtypes.
    fn visit(acc: &mut HashSet<Type>, typ: &Type) {
        if acc.contains(typ) {
            return;
        }

        // Trivially produce self.
        acc.insert(typ.clone());

        match typ {
            Type::Array(len, typ) => {
                if *len > 0 {
                    visit(acc, typ);
                }
                // Technically we could produce `[T; N]` from `[S; N]` if
                // we can produce `T` from `S`, but let's ignore that;
                // instead we will produce `[T; N]` from any source that can
                // supply `T`, one of which would be the `[S; N]` itself.
                // So if we have `let foo = [1u32, 2u32];` and we need `[u64; 2]`
                // we might generate `[foo[1] as u64, 3u64]` instead of "mapping"
                // over the entire foo. Same goes for tuples.
            }
            Type::Tuple(types) => {
                for typ in types {
                    visit(acc, typ);
                }
            }
            Type::String(_) => {
                // Maybe it could produce substrings, but it would be an overkill to enumerate.
            }
            Type::Field => {
                // There are `try_to_*` methods, but let's consider only what is safe.
                acc.insert(Type::Integer(Signedness::Unsigned, IntegerBitSize::HundredTwentyEight));
            }
            Type::Integer(sign, integer_bit_size) => {
                // Casting up is safe.
                for size in IntegerBitSize::iter()
                    .filter(|size| size.bit_size() > integer_bit_size.bit_size())
                {
                    acc.insert(Type::Integer(*sign, size));
                }
                // There are `From<u*>` for Field
                if !sign.is_signed() {
                    acc.insert(Type::Field);
                }
            }
            Type::Bool => {
                // Maybe we can also cast to u1 or u8 etc?
                acc.insert(Type::Field);
            }
            Type::Slice(typ) => {
                visit(acc, typ);
            }
            Type::Reference(typ, _) => {
                visit(acc, typ);
            }
            Type::Function(_, ret, _, _) => {
                visit(acc, ret);
            }
            Type::Unit | Type::FmtString(_, _) => {}
        }
    }

    let mut acc = HashSet::new();
    visit(&mut acc, typ);
    acc
}
