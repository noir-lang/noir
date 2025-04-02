use acir::FieldElement;
use nargo::errors::Location;
use std::collections::HashSet;
use strum::IntoEnumIterator;

use arbitrary::Unstructured;
use noirc_frontend::{
    ast::IntegerBitSize,
    hir_def::{self, expr::HirIdent, stmt::HirPattern},
    monomorphization::ast::{Expression, FuncId, InlineType, LocalId, Parameters, Type},
    node_interner::DefinitionId,
    shared::{Signedness, Visibility},
};

use super::{Context, expr::gen_expr_literal};

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
#[derive(Debug, Clone, Default)]
struct Scope {
    /// ID and type of variables created in all visible scopes,
    /// which includes this scope and its ancestors.
    variables: im::OrdMap<LocalId, Type>,
    /// Reverse index of local variables which can produce a type.
    /// For example an `(u8, [u64; 4])` can produce the tuple itself,
    /// the array in it, and both primitive types.
    producers: im::OrdMap<Type, im::OrdSet<LocalId>>,
}

impl Scope {
    /// Create the initial scope from function parameters.
    fn new(params: impl Iterator<Item = (LocalId, Type)>) -> Self {
        let mut scope = Self::default();
        for (id, typ) in params {
            scope.add(id, typ);
        }
        scope
    }

    /// Add a new local variable to the scope.
    fn add(&mut self, id: LocalId, typ: Type) {
        for typ in types_produced(&typ) {
            self.producers.entry(typ).or_default().insert(id);
        }
        self.variables.insert(id, typ);
    }

    /// Type of a local variable.
    fn get_variable(&self, id: &LocalId) -> &Type {
        self.variables.get(id).expect("local variable doesn't exist")
    }

    /// Get a random producer of a type, if there is one.
    fn get_producer(
        &self,
        u: &mut Unstructured,
        typ: &Type,
    ) -> arbitrary::Result<Option<&LocalId>> {
        let Some(ps) = self.producers.get(typ) else {
            return Ok(None);
        };
        u.choose_iter(ps.iter()).map(Some)
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
    /// Variables accumulated during the generation of the function body,
    /// initially consisting of the function parameters, then extended
    /// by locally defined variables. Block scopes add and remove layers.
    scopes: Vec<Scope>,
}

impl<'a> FunctionContext<'a> {
    pub fn new(ctx: &'a Context, id: FuncId) -> Self {
        let decl = ctx.function_decl(id);

        let scope = Scope::new(decl.params.iter().map(|(id, _, _, typ)| (*id, typ.clone())));
        let next_local_id = decl.params.iter().map(|p| p.0.0).max().unwrap_or_default();

        Self { ctx, decl, id, next_local_id, scopes: vec![scope] }
    }

    /// Generate the function body.
    pub fn gen_body(mut self, u: &mut Unstructured) -> arbitrary::Result<Expression> {
        self.gen_expr(u, &self.decl.return_type)
    }

    /// Local variables currently in scope.
    fn current_scope(&self) -> &Scope {
        self.scopes.last().expect("there is always the params layer")
    }

    /// Add a layer of block variables.
    fn enter_scope(&mut self) {
        // Instead of shallow cloning an immutable map, we could loop through layers when looking up variables.
        self.scopes.push(self.current_scope().clone());
    }

    /// Remove the last layer of block variables.
    fn exit_scope(&mut self) {
        self.scopes.pop();
        assert!(!self.scopes.is_empty(), "never pop the params layer");
    }

    /// Get and increment the next local ID.
    fn next_local_id(&mut self) -> LocalId {
        let id = LocalId(self.next_local_id);
        self.next_local_id += 1;
        id
    }

    /// Generate an expression of a certain type.
    ///
    /// While doing so, enter and exit blocks, and add variables declared to the context,
    /// so expressions down the line can refer to earlier variables.
    fn gen_expr(&mut self, u: &mut Unstructured, typ: &Type) -> arbitrary::Result<Expression> {
        gen_expr_literal(u, typ)
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
