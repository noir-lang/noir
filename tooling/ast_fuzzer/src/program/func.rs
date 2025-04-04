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
        ArrayLiteral, Assign, Binary, BinaryOp, Expression, FuncId, GlobalId, Ident, If, Index,
        InlineType, LValue, Let, Literal, LocalId, Parameters, Type, Unary,
    },
    node_interner::DefinitionId,
    shared::{Signedness, Visibility},
};

use crate::{Config, Freqs};

use super::{
    Context, Name, VariableId, expr,
    freq::Freq,
    make_name,
    scope::{Scope, ScopeStack, Variable},
    types,
};

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
            (!types::is_unit(&self.return_type)).then(|| types::to_hir_type(&self.return_type));

        (param_types, return_type)
    }
}

/// Control what kind of expressions we can generate, depending on the surrounding context.
#[derive(Debug, Clone, Copy)]
struct Flags {
    allow_blocks: bool,
    allow_if_then: bool,
}

impl Flags {
    /// In a top level context, everything is allowed.
    const TOP: Self = Self { allow_blocks: true, allow_if_then: true };
    /// In complex nested expressions, avoid generating blocks;
    /// they would be unreadable and non-idiomatic.
    const NESTED: Self = Self { allow_blocks: false, allow_if_then: true };
    /// In `if` conditions avoid nesting more ifs, like `if if if false ...`.
    const CONDITION: Self = Self { allow_blocks: false, allow_if_then: false };
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
    locals: ScopeStack<LocalId>,
}

impl<'a> FunctionContext<'a> {
    pub fn new(ctx: &'a mut Context, id: FuncId) -> Self {
        let decl = ctx.function_decl(id);
        let next_local_id = decl.params.iter().map(|p| p.0.0 + 1).max().unwrap_or_default();
        let budget = ctx.config.max_function_size;

        let globals = Scope::new(
            ctx.globals
                .iter()
                .map(|(id, (name, typ, _expr))| (*id, false, name.clone(), typ.clone())),
        );

        let locals = ScopeStack::new(
            decl.params
                .iter()
                .map(|(id, mutable, name, typ)| (*id, *mutable, name.clone(), typ.clone())),
        );

        Self { ctx, id, next_local_id, budget, globals, locals }
    }

    /// Generate the function body.
    pub fn gen_body(mut self, u: &mut Unstructured) -> arbitrary::Result<Expression> {
        let ret = self.decl().return_type.clone();
        self.gen_expr(u, &ret, self.max_depth(), Flags::TOP)
    }

    /// Get the function declaration.
    fn decl(&self) -> &FunctionDeclaration {
        self.ctx.function_decl(self.id)
    }

    /// The default maximum depth to start from. We use `max_depth` to limit the
    /// complexity of expressions such as binary ones, array indexes, etc.
    fn max_depth(&self) -> usize {
        self.ctx.config.max_depth
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
            if let Some(id) = self.locals.current().choose_producer(u, typ)? {
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
        let budget = budget.min(self.ctx.config.max_function_size / 5);
        self.decrease_budget(budget);
        Ok(budget)
    }

    /// Decrease the budget by some amount.
    fn decrease_budget(&mut self, amount: usize) {
        self.budget = self.budget.saturating_sub(amount);
    }

    /// Get a local or global variable.
    ///
    /// Panics if it doesn't exist.
    fn get_variable(&self, id: &VariableId) -> &Variable {
        match id {
            VariableId::Local(id) => self.locals.current().get_variable(id),
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
        flags: Flags,
    ) -> arbitrary::Result<Expression> {
        let mut freq = Freq::new(u, &self.ctx.config.expr_freqs)?;

        // Stop nesting if we reached the bottom.
        let allow_nested = max_depth > 0;

        let allow_blocks = flags.allow_blocks
            && allow_nested
            && max_depth == self.ctx.config.max_depth
            && self.budget > 0;

        let allow_if_then = flags.allow_if_then && allow_nested && self.budget > 0;

        if freq.enabled_when("unary", allow_nested && types::can_unary_return(typ)) {
            if let Some(expr) = self.gen_unary(u, typ, max_depth)? {
                return Ok(expr);
            }
        }

        if freq.enabled_when("binary", allow_nested && types::can_binary_return(typ)) {
            if let Some(expr) = self.gen_binary(u, typ, max_depth)? {
                return Ok(expr);
            }
        }

        // if-then-else returning a value
        // Unlike blocks/loops it can appear in nested expressions.
        if freq.enabled_when("if", allow_if_then) {
            return self.gen_if(u, typ, max_depth, flags);
        }

        // Block of statements returning a value
        if freq.enabled_when("block", allow_blocks) {
            return self.gen_block(u, typ);
        }

        // We can always try to just derive a value from the variables we have.
        if freq.enabled("vars") {
            if let Some(expr) = self.gen_expr_from_vars(u, typ, max_depth)? {
                return Ok(expr);
            }
        }

        // TODO: Match, Call

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
            let (mutable, src_name, src_type) = self.get_variable(&id).clone();
            let src_expr = expr::ident(id, mutable, src_name, src_type.clone());
            if let Some(expr) = self.gen_expr_from_source(u, src_expr, &src_type, typ, max_depth)? {
                return Ok(Some(expr));
            }
        } else {
            // If we can't produce the exact we're looking for, maybe we can produce parts of it.
            match typ {
                Type::Array(len, item_type) => {
                    let mut arr = ArrayLiteral { contents: Vec::new(), typ: typ.clone() };
                    for i in 0..*len {
                        let item = self.gen_expr(u, item_type, max_depth, Flags::NESTED)?;
                        arr.contents.push(item);
                    }
                    return Ok(Some(Expression::Literal(Literal::Array(arr))));
                }
                Type::Tuple(items) => {
                    let mut values = Vec::new();
                    for item_type in items {
                        let item = self.gen_expr(u, item_type, max_depth, Flags::NESTED)?;
                        values.push(item);
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
                let idx_expr = self.gen_index(u, *len, max_depth)?;
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
        len: u32,
        max_depth: usize,
    ) -> arbitrary::Result<Expression> {
        assert!(len > 0, "cannot index empty array");
        if max_depth > 0 && u.ratio(1, 3)? {
            let idx = self.gen_expr(u, &types::U32, max_depth.saturating_sub(1), Flags::NESTED)?;
            Ok(expr::index_modulo(idx, len))
        } else {
            let idx = u.choose_index(len as usize)?;
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
            self.gen_expr(u, typ, max_depth.saturating_sub(1), Flags::NESTED)
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
        fn collect_input_types<K: Ord>(op: BinaryOp, scope: &Scope<K>) -> Vec<&Type> {
            scope
                .types_produced()
                .filter(|typ| types::can_binary_op_take(&op, typ))
                .collect::<Vec<_>>()
        }

        // Try local variables first.
        let mut lhs_opts = collect_input_types(op, self.locals.current());

        // If the locals don't have any type compatible with `op`, try the globals.
        if lhs_opts.is_empty() {
            lhs_opts = collect_input_types(op, &self.globals);
        }

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
        let lhs_expr = self.gen_expr(u, &lhs_type, max_depth.saturating_sub(1), Flags::NESTED)?;
        let rhs_expr = self.gen_expr(u, rhs_type, max_depth.saturating_sub(1), Flags::NESTED)?;

        Ok(Some(expr::binary(lhs_expr, op, rhs_expr)))
    }

    /// Generate a block of statements, finally returning a target type.
    ///
    /// This should always succeed, as we can always create a literal in the end.
    fn gen_block(&mut self, u: &mut Unstructured, typ: &Type) -> arbitrary::Result<Expression> {
        /// The `max_depth` resets here, because that's only relevant in complex expressions.
        let max_depth = self.max_depth();
        let size = self.choose_budget(u)?;
        if size == 0 {
            return self.gen_expr(u, typ, max_depth, Flags::TOP);
        }
        let mut stmts = Vec::new();

        self.locals.enter();
        for i in 0..size - 1 {
            stmts.push(self.gen_stmt(u)?);
        }
        if types::is_unit(typ) && bool::arbitrary(u)? {
            stmts.push(Expression::Semi(Box::new(self.gen_stmt(u)?)))
        } else {
            stmts.push(self.gen_expr(u, typ, max_depth, Flags::TOP)?);
        }
        self.locals.exit();

        Ok(Expression::Block(stmts))
    }

    /// Generate a statement, which is an expression that doesn't return anything,
    /// for example loops, variable declarations, etc.
    fn gen_stmt(&mut self, u: &mut Unstructured) -> arbitrary::Result<Expression> {
        let mut freq = Freq::new(u, &self.ctx.config.stmt_freqs)?;
        // TODO: For, Loop, While, Match, Call

        if freq.enabled("drop") {
            if let Some(e) = self.gen_drop(u)? {
                return Ok(e);
            }
        }

        if freq.enabled("assign") {
            if let Some(e) = self.gen_assign(u)? {
                return Ok(e);
            }
        }

        if freq.enabled("if") {
            return self.gen_if(u, &Type::Unit, self.max_depth(), Flags::TOP);
        }

        self.gen_let(u)
    }

    /// Generate a `Let` statement.
    fn gen_let(&mut self, u: &mut Unstructured) -> arbitrary::Result<Expression> {
        // Generate a type or choose an existing one.
        let max_depth = self.max_depth();
        let typ = self.ctx.gen_type(u, max_depth)?;
        let id = self.next_local_id();
        let mutable = bool::arbitrary(u)?;
        let name = make_name(id.0 as usize, false);
        let expr = self.gen_expr(u, &typ, max_depth, Flags::TOP)?;

        // Add the variable so we can use it in subsequent expressions.
        self.locals.add(id, mutable, name.clone(), typ.clone());

        Ok(Expression::Let(Let { id, mutable, name, expression: Box::new(expr) }))
    }

    /// Drop a local variable, if we have anything to drop.
    fn gen_drop(&mut self, u: &mut Unstructured) -> arbitrary::Result<Option<Expression>> {
        if self.locals.current().is_empty() {
            return Ok(None);
        }
        let id = *u.choose_iter(self.locals.current().variable_ids())?;
        let (mutable, name, typ) = self.locals.current().get_variable(&id).clone();

        // Remove variable so we stop using it.
        self.locals.remove(&id);

        Ok(Some(Expression::Drop(Box::new(expr::ident(VariableId::Local(id), mutable, name, typ)))))
    }

    /// Assign to a mutable variable, if we have one in scope.
    fn gen_assign(&mut self, u: &mut Unstructured) -> arbitrary::Result<Option<Expression>> {
        let opts = self
            .locals
            .current()
            .variables()
            .filter_map(|(id, (mutable, _, _))| mutable.then_some(id))
            .collect::<Vec<_>>();

        if opts.is_empty() {
            return Ok(None);
        }

        let id = *u.choose_iter(opts)?;
        let (mutable, name, typ) = self.locals.current().get_variable(&id).clone();
        let expr = self.gen_expr(u, &typ, self.max_depth(), Flags::TOP)?;
        let ident = expr::ident_inner(VariableId::Local(id), mutable, name, typ.clone());
        let ident = LValue::Ident(ident);

        // For arrays and tuples we can consider assigning to their items.
        let (lvalue, typ) = match typ {
            Type::Array(len, typ) if len > 0 && bool::arbitrary(u)? => {
                let idx = self.gen_index(u, len, self.max_depth())?;
                let lvalue = LValue::Index {
                    array: Box::new(ident),
                    index: Box::new(idx),
                    element_type: typ.as_ref().clone(),
                    location: Location::dummy(),
                };
                (lvalue, *typ)
            }
            Type::Tuple(items) if bool::arbitrary(u)? => {
                let idx = u.choose_index(items.len())?;
                let typ = items[idx].clone();
                let lvalue = LValue::MemberAccess { object: Box::new(ident), field_index: idx };
                (lvalue, typ)
            }
            _ => (ident, typ),
        };

        let expr = self.gen_expr(u, &typ, self.max_depth(), Flags::TOP)?;

        Ok(Some(Expression::Assign(Assign { lvalue, expression: Box::new(expr) })))
    }

    /// Generate an if-then-else statement or expression.
    fn gen_if(
        &mut self,
        u: &mut Unstructured,
        typ: &Type,
        max_depth: usize,
        flags: Flags,
    ) -> arbitrary::Result<Expression> {
        // Decrease the budget so we avoid a potential infinite nesting of ifs in the arms.
        self.decrease_budget(2);

        let condition = self.gen_expr(u, &Type::Bool, max_depth, Flags::CONDITION)?;

        let consequence = {
            self.locals.enter();
            let expr = self.gen_expr(u, typ, max_depth, flags)?;
            self.locals.exit();
            expr
        };

        let alternative = if types::is_unit(typ) && bool::arbitrary(u)? {
            None
        } else {
            self.decrease_budget(1);
            self.locals.enter();
            let expr = self.gen_expr(u, typ, max_depth, flags)?;
            self.locals.exit();
            Some(expr)
        };

        Ok(Expression::If(If {
            condition: Box::new(condition),
            consequence: Box::new(consequence),
            alternative: alternative.map(Box::new),
            typ: typ.clone(),
        }))
    }
}
