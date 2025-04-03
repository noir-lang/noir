#![allow(unused)] // TODO(#7879): Remove when done.
use acir::FieldElement;
use nargo::errors::Location;
use std::{collections::HashSet, fmt::Debug};
use strum::IntoEnumIterator;

use arbitrary::{Arbitrary, Unstructured};
use noirc_frontend::{
    ast::{IntegerBitSize, UnaryOp},
    hir_def::{self, expr::HirIdent, stmt::HirPattern},
    monomorphization::ast::{
        ArrayLiteral, Binary, BinaryOp, Expression, FuncId, GlobalId, Index, InlineType, Let,
        Literal, LocalId, Parameters, Type, Unary,
    },
    node_interner::DefinitionId,
    shared::{Signedness, Visibility},
};

use crate::Config;

use super::{Context, Name, VariableId, expr, make_name, types};

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

                let typ = types::to_hir_type(typ);

                (pat, typ, *vis)
            })
            .collect();

        let return_type =
            (self.return_type != Type::Unit).then(|| types::to_hir_type(&self.return_type));

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
        for typ in types::types_produced(&typ) {
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
        if vs.is_empty() {
            return Ok(None);
        }
        u.choose_iter(vs.iter()).map(Some).map(|v| v.cloned())
    }
}

/// Context used during the generation of a function body.
pub(super) struct FunctionContext<'a> {
    /// Top level context, to access global variables and other functions.
    ctx: &'a mut Context,
    /// Self ID.
    id: FuncId,
    /// Every variable created in the function will have an increasing ID,
    /// which does not reset when variables go out of scope.
    next_local_id: u32,
    /// Number of statements remaining to be generated in the function.
    budget: usize,
    /// Global variables.
    globals: Scope<GlobalId>,
    /// Variables accumulated during the generation of the function body,
    /// initially consisting of the function parameters, then extended
    /// by locally defined variables. Block scopes add and remove layers.
    locals: Vec<Scope<LocalId>>,
}

impl<'a> FunctionContext<'a> {
    pub fn new(ctx: &'a mut Context, id: FuncId) -> Self {
        let decl = ctx.function_decl(id);
        let next_local_id = decl.params.iter().map(|p| p.0.0 + 1).max().unwrap_or_default();
        let budget = ctx.config.max_function_size;

        let globals = Scope::new(
            ctx.globals.iter().map(|(id, (name, typ, _expr))| (*id, name.clone(), typ.clone())),
        );

        let locals = Scope::new(
            decl.params.iter().map(|(id, _, name, typ)| (*id, name.clone(), typ.clone())),
        );

        Self { ctx, id, next_local_id, budget, globals, locals: vec![locals] }
    }

    /// Generate the function body.
    pub fn gen_body(mut self, u: &mut Unstructured) -> arbitrary::Result<Expression> {
        let ret = self.decl().return_type.clone();
        self.gen_expr(u, &ret, self.start_depth())
    }

    /// Get the function declaration.
    fn decl(&self) -> &FunctionDeclaration {
        self.ctx.function_decl(self.id)
    }

    /// The default maximum depth to start from. We use `max_depth` to limit the
    /// complexity of expressions such as binary ones, array indexes, etc.
    fn start_depth(&self) -> usize {
        self.ctx.config.max_depth
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

    /// Take some of the available budget.
    fn choose_budget(&mut self, u: &mut Unstructured) -> arbitrary::Result<usize> {
        if self.budget == 0 {
            return Ok(self.budget);
        }
        let budget = u.choose_index(self.budget)?;
        // Limit it so we don't blow it on the first block.
        let budget = budget.min(self.ctx.config.max_function_size / 2);
        self.budget -= budget;
        Ok(budget)
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
    ///
    /// This will always succeed, because we can always return a literal expression.
    fn gen_expr(
        &mut self,
        u: &mut Unstructured,
        typ: &Type,
        max_depth: usize,
    ) -> arbitrary::Result<Expression> {
        let mut freq = Freq::new(u, 100, max_depth, &self.ctx.config)?;

        // Unary
        if freq.prob_nested(10, types::can_unary_return(typ)) {
            if let Some(expr) = self.gen_unary(u, typ, max_depth)? {
                return Ok(expr);
            }
        }

        // Binary
        if freq.prob_nested(25, types::can_binary_return(typ)) {
            if let Some(expr) = self.gen_binary(u, typ, max_depth)? {
                return Ok(expr);
            }
        }

        // Block of statements
        if freq.prob_block(20, self.budget) {
            return self.gen_block(u, typ);
        }

        // TODO: If, Match, Call

        // We can always try to just derive a value from the variables we have.
        if freq.prob(50) {
            if let Some(expr) = self.gen_expr_from_vars(u, typ, max_depth)? {
                return Ok(expr);
            }
        }

        // If nothing else worked out we can always produce a random literal.
        let lit = expr::gen_literal(u, typ)?;

        Ok(lit)
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
                    for i in 0..*len {
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
                let e = if bool::arbitrary(u)? {
                    Expression::Clone(Box::new(src_expr))
                } else {
                    expr::deref(src_expr, tgt_type.clone())
                };
                Ok(Some(e))
            }
            (Type::Array(len, item_typ), _) if *len > 0 => {
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
        assert!(len > 0, "cannot index empty array");
        if max_depth > 0 && u.ratio(1, 3)? {
            let idx = self.gen_expr(u, &types::U32, max_depth.saturating_sub(1))?;
            Ok(expr::index_modulo(idx, len))
        } else {
            let idx = u.choose_index(len)?;
            Ok(expr::u32_literal(idx as u32))
        }
    }

    /// Try to generate a unary expression of a certain type, if it's amenable to it, otherwise return `None`.
    fn gen_unary(
        &mut self,
        u: &mut Unstructured,
        typ: &Type,
        max_depth: usize,
    ) -> arbitrary::Result<Option<Expression>> {
        let mut make_unary = |op| {
            self.gen_expr(u, typ, max_depth.saturating_sub(1))
                .map(|rhs| Some(expr::unary(op, rhs, typ.clone())))
        };
        if types::is_signed(typ) {
            make_unary(UnaryOp::Minus)
        } else if types::is_bool(typ) {
            make_unary(UnaryOp::Not)
        } else {
            Ok(None)
        }
    }

    /// Try to generate a binary expression of a certain type, if it's amenable to it, otherwise return `None`.
    fn gen_binary(
        &mut self,
        u: &mut Unstructured,
        typ: &Type,
        max_depth: usize,
    ) -> arbitrary::Result<Option<Expression>> {
        // Collect the operations can produce the expected type.
        let ops =
            BinaryOp::iter().filter(|op| types::can_binary_op_return(op, typ)).collect::<Vec<_>>();

        // Ideally we checked that the target type can be returned, but just in case.
        if ops.is_empty() {
            return Ok(None);
        }

        // Choose a random operation.
        let op = u.choose_iter(ops)?;

        // Find a type we can produce in the current scope which works with this operation.
        let lhs_opts = self
            .current_scope()
            .producers
            .keys()
            .filter(|typ| types::can_binary_op_take(&op, typ))
            .collect::<Vec<_>>();

        // We might not have any input that works for this operation.
        // We could generate literals, but that's not super interesting.
        if lhs_opts.is_empty() {
            return Ok(None);
        }

        // Choose a type for the LHS and RHS.
        let lhs_type = u.choose_iter(lhs_opts)?.clone();
        let rhs_type = match op {
            BinaryOp::ShiftLeft | BinaryOp::ShiftRight => &types::U8,
            _ => &lhs_type,
        };

        // Generate expressions for LHS and RHS.
        let lhs_expr = self.gen_expr(u, &lhs_type, max_depth.saturating_sub(1))?;
        let rhs_expr = self.gen_expr(u, rhs_type, max_depth.saturating_sub(1))?;

        Ok(Some(expr::binary(lhs_expr, op, rhs_expr)))
    }

    /// Generate a block of statements, finally returning a target type.
    ///
    /// This should always succeed, as we can always create a literal in the end.
    fn gen_block(&mut self, u: &mut Unstructured, typ: &Type) -> arbitrary::Result<Expression> {
        /// The `max_depth` resets here, because that's only relevant in complex expressions.
        let max_depth = self.start_depth();
        let size = self.choose_budget(u)?;
        if size == 0 {
            return self.gen_expr(u, typ, max_depth);
        }
        let mut stmts = Vec::new();

        self.enter_scope();
        for i in 0..size - 1 {
            stmts.push(self.gen_statement(u)?);
        }
        // TODO: If the `typ` was `Unit`, we could use `Semi` in the last position.
        stmts.push(self.gen_expr(u, typ, max_depth)?);
        self.exit_scope();

        Ok(Expression::Block(stmts))
    }

    /// Generate a statement, which is an expression that doesn't return anything,
    /// for example loops, variable declarations, etc.
    fn gen_statement(&mut self, u: &mut Unstructured) -> arbitrary::Result<Expression> {
        let mut _freq = Freq::new(u, 100, self.start_depth(), &self.ctx.config)?;
        // TODO: For, Loop, While, If, Match, Call, Assign
        self.gen_let(u)
    }

    /// Generate a `Let` statement.
    fn gen_let(&mut self, u: &mut Unstructured) -> arbitrary::Result<Expression> {
        // Generate a type or choose an existing one.
        let max_depth = self.start_depth();
        let typ = self.ctx.gen_type(u, max_depth)?;
        let id = self.next_local_id();
        let mutable = bool::arbitrary(u)?;
        let name = make_name(id.0 as usize, false);

        let expr = self.gen_expr(u, &typ, max_depth)?;
        Ok(Expression::Let(Let { id, mutable, name, expression: Box::new(expr) }))
    }
}

/// Help with cumulative frequency distributions.
struct Freq {
    x: usize,
    total: usize,
    depth: usize,
    max_depth: usize,
}

impl Freq {
    fn new(
        u: &mut Unstructured,
        total: usize,
        depth: usize,
        config: &Config,
    ) -> arbitrary::Result<Self> {
        let x = u.choose_index(total)?;
        Ok(Self { x: u.choose_index(total)?, total: 0, depth, max_depth: config.max_depth })
    }

    /// Check if we're in the next `p` size window on top of the already checked cumulative values.
    fn prob(&mut self, p: usize) -> bool {
        self.total += p;
        self.x < self.total
    }

    /// Like `prob`, but if `cond` is `false` it does not increase the cumulative value,
    /// so as not to distort the next call, ie. if we have have 5% then another 5%,
    /// if the first one is disabled, the second doesn't become 10%.
    fn prob_if(&mut self, p: usize, cond: bool) -> bool {
        cond && self.prob(p)
    }

    /// Like `prob_if`, but only if we're not in the bottom depth yet, to limit how
    /// deeply nested expressions we generate.
    fn prob_nested(&mut self, p: usize, cond: bool) -> bool {
        self.prob_if(p, self.depth > 0 && cond)
    }

    /// Like `prob`, but only if we're not in a nested expression, so as not to
    /// generate blocks, and loops where we're generating e.g. array indexes
    /// or loop range values; the compiler might be okay with it, but it would
    /// be hard to read and unidiomatic.
    fn prob_block(&mut self, p: usize, budget: usize) -> bool {
        self.prob_if(p, self.depth == self.max_depth && budget > 0)
    }
}
