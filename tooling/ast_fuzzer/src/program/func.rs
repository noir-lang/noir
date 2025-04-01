use acir::FieldElement;
use nargo::errors::Location;

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

/// Context used during the generation of a function body.
#[allow(unused)]
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
    variables: Vec<im::OrdMap<LocalId, Type>>,
}

#[allow(unused)]
impl<'a> FunctionContext<'a> {
    pub fn new(ctx: &'a Context, id: FuncId) -> Self {
        let decl = ctx.function_decl(id);

        let params = decl.params.iter().map(|(id, _, _, typ)| (*id, typ.clone())).collect();
        let next_local_id = decl.params.iter().map(|p| p.0.0).max().unwrap_or_default();

        Self { ctx, decl, id, next_local_id, variables: vec![params] }
    }

    /// Generate the function body.
    pub fn gen_body(self, u: &mut Unstructured) -> arbitrary::Result<Expression> {
        // TODO: Generate a random AST using the variables, and return the expected type.
        gen_expr_literal(u, &self.decl.return_type)
    }

    /// Local variables currently in scope.
    fn current_scope(&self) -> &im::OrdMap<LocalId, Type> {
        self.variables.last().expect("there is always the params layer")
    }

    /// Add a layer of block variables.
    fn enter_scope(&mut self) {
        // Instead of shallow cloning an immutable map, we could loop through layers when looking up variables.
        self.variables.push(self.current_scope().clone());
    }

    /// Remove the last layer of block variables.
    fn exit_scope(&mut self) {
        self.variables.pop();
        assert!(!self.variables.is_empty(), "never pop the params layer");
    }

    /// Look up a local variable.
    ///
    /// Panics if it doesn't exist.
    fn local_variable(&self, id: &LocalId) -> &Type {
        self.current_scope().get(id).expect("local variable doesn't exist")
    }

    /// Get and increment the next local ID.
    fn next_local_id(&mut self) -> LocalId {
        let id = LocalId(self.next_local_id);
        self.next_local_id += 1;
        id
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
