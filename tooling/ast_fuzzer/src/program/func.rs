use nargo::errors::Location;
use std::{
    collections::{BTreeMap, BTreeSet, HashSet},
    fmt::Debug,
};
use strum::IntoEnumIterator;

use arbitrary::{Arbitrary, Unstructured};
use noirc_frontend::{
    ast::{IntegerBitSize, UnaryOp},
    hir_def::{self, expr::HirIdent, stmt::HirPattern},
    monomorphization::ast::{
        ArrayLiteral, Assign, BinaryOp, Call, Definition, Expression, For, FuncId, GlobalId, Ident,
        IdentId, Index, InlineType, LValue, Let, Literal, LocalId, Parameters, Program, Type,
        While,
    },
    node_interner::DefinitionId,
    shared::{Signedness, Visibility},
};

use super::{
    Context, VariableId, expr,
    freq::Freq,
    make_name,
    scope::{Scope, ScopeStack, Variable},
    types,
};

/// Something akin to a forward declaration of a function, capturing the details required to:
/// 1. call the function from the other function bodies
/// 2. generate the final HIR function signature
#[derive(Debug, Clone)]
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
            .map(|((_id, mutable, _name, typ), vis)| hir_param(*mutable, typ, *vis))
            .collect();

        let return_type =
            (!types::is_unit(&self.return_type)).then(|| types::to_hir_type(&self.return_type));

        (param_types, return_type)
    }

    fn is_acir(&self) -> bool {
        !self.unconstrained
    }

    fn is_brillig(&self) -> bool {
        self.unconstrained
    }
}

/// HIR representation of a function parameter.
pub(crate) fn hir_param(
    mutable: bool,
    typ: &Type,
    vis: Visibility,
) -> (HirPattern, hir_def::types::Type, Visibility) {
    // The pattern doesn't seem to be used in `ssa::create_program`,
    // apart from its location, so it shouldn't matter what we put into it.
    let mut pat = HirPattern::Identifier(HirIdent {
        location: Location::dummy(),
        id: DefinitionId::dummy_id(),
        impl_kind: hir_def::expr::ImplKind::NotATraitMethod,
    });
    if mutable {
        pat = HirPattern::Mutable(Box::new(pat), Location::dummy());
    }

    let typ = types::to_hir_type(typ);

    (pat, typ, vis)
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
    /// In `for` ranges we can use `if` expressions, but let's not do blocks.
    const RANGE: Self = Self { allow_blocks: false, allow_if_then: true };
    /// In call arguments we can use `if` expressions, but avoid blocks.
    /// The arg expressions themselves might call other functions.
    const CALL: Self = Self { allow_blocks: false, allow_if_then: true };
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
    /// Every identifier created in the function will have an increasing ID,
    /// which does not reset when variables go out of scope.
    next_ident_id: u32,
    /// Number of statements remaining to be generated in the function.
    budget: usize,
    /// Global variables.
    globals: Scope<GlobalId>,
    /// Variables accumulated during the generation of the function body,
    /// initially consisting of the function parameters, then extended
    /// by locally defined variables. Block scopes add and remove layers.
    locals: ScopeStack<LocalId>,
    /// Indicator of being in a loop (and hence able to generate
    /// break and continue statements)
    in_loop: bool,
    /// All the functions callable from this one, with the types we can
    /// produce from their return value.
    call_targets: BTreeMap<FuncId, HashSet<Type>>,
    /// Indicate that we have generated a `Call`.
    has_call: bool,
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

        // Collect all the functions we can call from this one.
        let call_targets = ctx
            .function_declarations
            .iter()
            .filter_map(|(callee_id, callee_decl)| {
                // We can't call `main`.
                if *callee_id == Program::main_id() {
                    return None;
                }

                // From an ACIR function we can call any Brillig function,
                // but we avoid creating infinite recursive ACIR calls by
                // only calling functions with higher IDs than ours,
                // otherwise the inliner could get stuck.
                if decl.is_acir() && callee_decl.is_acir() && *callee_id <= id {
                    return None;
                }

                // From a Brillig function we restrict ourselves to only call
                // other Brillig functions. That's because the `Monomorphizer`
                // would make an unconstrained copy of any ACIR function called
                // from Brillig, and this is expected by the inliner for example,
                // but if we did similarly in the generator after we know who
                // calls who, we would incur two drawbacks:
                // 1) it would make programs bigger for little benefit
                // 2) it would skew calibration frequencies as ACIR freqs would overlay Brillig ones
                if decl.is_brillig() && !callee_decl.is_brillig() {
                    return None;
                }

                Some((*callee_id, types::types_produced(&callee_decl.return_type)))
            })
            .collect();

        Self {
            ctx,
            id,
            next_local_id,
            budget,
            globals,
            locals,
            in_loop: false,
            call_targets,
            next_ident_id: 0,
            has_call: false,
        }
    }

    /// Generate the function body.
    pub fn gen_body(mut self, u: &mut Unstructured) -> arbitrary::Result<Expression> {
        // If we don't limit the budget according to the available data,
        // it gives us a lot of `false` and 0 and we end up with deep `!(!false)` if expressions.
        self.budget = self.budget.min(u.len());
        let ret = self.decl().return_type.clone();
        let mut body = self.gen_expr(u, &ret, self.max_depth(), Flags::TOP)?;
        if let Some(call) = self.gen_guaranteed_call_from_main(u)? {
            expr::prepend(&mut body, call);
        }
        Ok(body)
    }

    /// Generate the function body, wrapping a function call with literal arguments.
    /// This is used to test comptime functions, which can only take those.
    pub fn gen_body_with_lit_call(
        mut self,
        u: &mut Unstructured,
        callee_id: FuncId,
    ) -> arbitrary::Result<Expression> {
        self.gen_lit_call(u, callee_id)
    }

    /// Get the function declaration.
    fn decl(&self) -> &FunctionDeclaration {
        self.ctx.function_decl(self.id)
    }

    /// Is this function unconstrained.
    fn unconstrained(&self) -> bool {
        self.decl().unconstrained
    }

    /// Is this the main function?
    fn is_main(&self) -> bool {
        self.id == Program::main_id()
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

    /// Get and increment the next ident ID.
    fn next_ident_id(&mut self) -> IdentId {
        let id = IdentId(self.next_ident_id);
        self.next_ident_id += 1;
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

        // Function calls returning a value.
        if freq.enabled_when("call", allow_nested && self.budget > 0) {
            // Decreasing the max depth in expression position because it can be very difficult to read.
            if let Some(expr) = self.gen_call(u, typ, max_depth.saturating_sub(1))? {
                return Ok(expr);
            }
        }

        // We can always try to just derive a value from the variables we have.
        if freq.enabled("vars") {
            if let Some(expr) = self.gen_expr_from_vars(u, typ, max_depth)? {
                return Ok(expr);
            }
        }

        // TODO(#7926): Match

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
            let ident_id = self.next_ident_id();
            let src_expr = expr::ident(id, ident_id, mutable, src_name, src_type.clone());
            if let Some(expr) = self.gen_expr_from_source(u, src_expr, &src_type, typ, max_depth)? {
                return Ok(Some(expr));
            }
        } else {
            // If we can't produce the exact we're looking for, maybe we can produce parts of it.
            match typ {
                Type::Array(len, item_type) => {
                    let mut arr = ArrayLiteral {
                        contents: Vec::with_capacity(*len as usize),
                        typ: typ.clone(),
                    };
                    for _ in 0..*len {
                        let item = self.gen_expr(u, item_type, max_depth, Flags::NESTED)?;
                        arr.contents.push(item);
                    }
                    return Ok(Some(Expression::Literal(Literal::Array(arr))));
                }
                Type::Tuple(items) => {
                    let mut values = Vec::with_capacity(items.len());
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
                // TODO(#7929): We don't know the length of the slice at compile time,
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
        if types::is_numeric(typ) {
            // Assume we already checked with `can_unary_return` that it's signed.
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
        // Collect the operations can return the expected type.
        let ops = BinaryOp::iter()
            .filter(|op| {
                types::can_binary_op_return(op, typ)
                    && (!self.ctx.config.avoid_overflow || !types::can_binary_op_overflow(op))
                    && (!self.ctx.config.avoid_err_by_zero || !types::can_binary_op_err_by_zero(op))
            })
            .collect::<Vec<_>>();

        // Ideally we checked that the target type can be returned, but just in case.
        if ops.is_empty() {
            return Ok(None);
        }

        // Choose a random operation.
        let op = u.choose_iter(ops)?;

        // Find a type we can produce in the current scope which we can pass as input
        // to the operations we selected, and it returns the desired output.
        fn collect_input_types<'a, K: Ord>(
            this: &FunctionContext,
            op: BinaryOp,
            type_out: &Type,
            scope: &'a Scope<K>,
        ) -> Vec<&'a Type> {
            scope
                .types_produced()
                .filter(|type_in| types::can_binary_op_return_from_input(&op, type_in, type_out))
                .filter(|type_in| !this.ctx.should_avoid_literals(type_in))
                .collect::<Vec<_>>()
        }

        // Try local variables first.
        let mut lhs_opts = collect_input_types(self, op, typ, self.locals.current());

        // If the locals don't have any type compatible with `op`, try the globals.
        if lhs_opts.is_empty() {
            lhs_opts = collect_input_types(self, op, typ, &self.globals);
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
        let mut expr = expr::binary(lhs_expr, op, rhs_expr);

        // If we have chosen e.g. u8 and need u32 we need to cast.
        if !(lhs_type == *typ || types::is_bool(typ) && op.is_comparator()) {
            expr = expr::cast(expr, typ.clone());
        }

        Ok(Some(expr))
    }

    /// Generate a block of statements, finally returning a target type.
    ///
    /// This should always succeed, as we can always create a literal in the end.
    fn gen_block(&mut self, u: &mut Unstructured, typ: &Type) -> arbitrary::Result<Expression> {
        // The `max_depth` resets here, because that's only relevant in complex expressions.
        let max_depth = self.max_depth();
        let max_size = self.ctx.config.max_block_size.min(self.budget);

        // If we want blocks to be empty, or we don't have a budget for statements, just return an expression.
        if max_size == 0 {
            return self.gen_expr(u, typ, max_depth, Flags::TOP);
        }

        // Choose a positive number of statements.
        let size = u.int_in_range(1..=max_size)?;
        let mut stmts = Vec::with_capacity(size);

        self.locals.enter();
        self.decrease_budget(1);
        for _ in 0..size - 1 {
            if self.budget == 0 {
                break;
            }
            self.decrease_budget(1);
            stmts.push(self.gen_stmt(u)?);
        }
        if types::is_unit(typ) && u.ratio(4, 5)? {
            // ending a unit block with `<stmt>;` looks better than a `()` but both are valid.
            // NB the AST printer puts a `;` between all statements, including after `if` and `for`.
            stmts.push(Expression::Semi(Box::new(self.gen_stmt(u)?)));
        } else {
            stmts.push(self.gen_expr(u, typ, max_depth, Flags::TOP)?);
        }
        self.locals.exit();

        Ok(Expression::Block(stmts))
    }

    /// Generate a statement, which is an expression that doesn't return anything,
    /// for example loops, variable declarations, etc.
    fn gen_stmt(&mut self, u: &mut Unstructured) -> arbitrary::Result<Expression> {
        let mut freq = if self.unconstrained() {
            Freq::new(u, &self.ctx.config.stmt_freqs_brillig)?
        } else {
            Freq::new(u, &self.ctx.config.stmt_freqs_acir)?
        };
        // TODO(#7926): Match
        // TODO(#7931): print
        // TODO(#7932): Constrain

        // Start with `drop`, it doesn't need to be frequent even if others are disabled.
        if freq.enabled("drop") {
            if let Some(e) = self.gen_drop(u)? {
                return Ok(e);
            }
        }

        // Require a positive budget, so that we have some for the block itself and its contents.
        if freq.enabled_when("if", self.budget > 1) {
            return self.gen_if(u, &Type::Unit, self.max_depth(), Flags::TOP);
        }

        if freq.enabled_when("for", self.budget > 1) {
            return self.gen_for(u);
        }

        if freq.enabled_when("call", self.budget > 0) {
            if let Some(e) = self.gen_call(u, &Type::Unit, self.max_depth())? {
                return Ok(e);
            }
        }

        if self.unconstrained() {
            // Get loop out of the way quick, as it's always disabled for ACIR.
            if freq.enabled_when("loop", self.budget > 1) {
                return self.gen_loop(u);
            }

            if freq.enabled_when("while", self.budget > 1) {
                return self.gen_while(u);
            }

            if freq.enabled_when("break", self.in_loop) {
                return Ok(Expression::Break);
            }

            if freq.enabled_when("continue", self.in_loop) {
                return Ok(Expression::Continue);
            }
        }

        if freq.enabled("assign") {
            if let Some(e) = self.gen_assign(u)? {
                return Ok(e);
            }
        }

        self.gen_let(u)
    }

    /// Generate a `Let` statement with arbitrary type and value.
    fn gen_let(&mut self, u: &mut Unstructured) -> arbitrary::Result<Expression> {
        // Generate a type or choose an existing one.
        let max_depth = self.max_depth();
        let typ = self.ctx.gen_type(u, max_depth, false, true)?;
        let expr = self.gen_expr(u, &typ, max_depth, Flags::TOP)?;
        let mutable = bool::arbitrary(u)?;
        Ok(self.let_var(mutable, typ, expr, true))
    }

    /// Add a new local variable and return a `Let` expression.
    ///
    /// If `add_to_scope` is `false`, the value will not be added to the `locals`.
    fn let_var(
        &mut self,
        mutable: bool,
        typ: Type,
        expr: Expression,
        add_to_scope: bool,
    ) -> Expression {
        let id = self.next_local_id();
        let name = make_name(id.0 as usize, false);

        // Add the variable so we can use it in subsequent expressions.
        if add_to_scope {
            self.locals.add(id, mutable, name.clone(), typ.clone());
        }

        expr::let_var(id, mutable, name, expr)
    }

    /// Drop a local variable, if we have anything to drop.
    ///
    /// The `ownership` module has a comment saying it will be the only one inserting `Clone` and `Drop`,
    /// so this shouldn't be needed unless a user can do it via a `drop`-like method.
    ///
    /// Leaving it here for reference, but its frequency is adjusted to be 0.
    fn gen_drop(&mut self, u: &mut Unstructured) -> arbitrary::Result<Option<Expression>> {
        if self.locals.current().is_empty() {
            return Ok(None);
        }
        let id = *u.choose_iter(self.locals.current().variable_ids())?;
        let (mutable, name, typ) = self.locals.current().get_variable(&id).clone();

        // Remove variable so we stop using it.
        self.locals.remove(&id);

        let ident_id = self.next_ident_id();
        Ok(Some(Expression::Drop(Box::new(expr::ident(
            VariableId::Local(id),
            ident_id,
            mutable,
            name,
            typ,
        )))))
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
        let ident_id = self.next_ident_id();
        let ident = expr::ident_inner(VariableId::Local(id), ident_id, mutable, name, typ.clone());
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

        // Generate the assigned value.
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
        // Decrease the budget so we avoid a potential infinite nesting of if expressions in the arms.
        self.decrease_budget(1);

        let condition = self.gen_expr(u, &Type::Bool, max_depth, Flags::CONDITION)?;

        let consequence = {
            if flags.allow_blocks {
                self.gen_block(u, typ)?
            } else {
                self.gen_expr(u, typ, max_depth, flags)?
            }
        };

        let alternative = if types::is_unit(typ) && bool::arbitrary(u)? {
            None
        } else {
            self.decrease_budget(1);
            let expr = if flags.allow_blocks {
                self.gen_block(u, typ)?
            } else {
                self.gen_expr(u, typ, max_depth, flags)?
            };
            Some(expr)
        };

        Ok(expr::if_then(condition, consequence, alternative, typ.clone()))
    }

    /// Generate a `for` loop.
    fn gen_for(&mut self, u: &mut Unstructured) -> arbitrary::Result<Expression> {
        // The index can be signed or unsigned int, 8 to 128 bits, except i128,
        // but currently the frontend expects it to be u32 unless it's declared as a separate variable.
        let idx_type = {
            let bit_size = if self.ctx.config.avoid_large_int_literals {
                IntegerBitSize::ThirtyTwo
            } else {
                u.choose(&[8, 16, 32, 64, 128]).map(|s| IntegerBitSize::try_from(*s).unwrap())?
            };

            Type::Integer(
                if bit_size == IntegerBitSize::HundredTwentyEight
                    || self.ctx.config.avoid_negative_int_literals
                    || bool::arbitrary(u)?
                {
                    Signedness::Unsigned
                } else {
                    Signedness::Signed
                },
                bit_size,
            )
        };

        let (start_range, end_range) = if self.unconstrained() && bool::arbitrary(u)? {
            // Choosing a maximum range size because changing it immediately brought out some bug around modulo.
            let max_size = u.int_in_range(1..=self.ctx.config.max_loop_size)?;
            // Generate random expression.
            let s = self.gen_expr(u, &idx_type, self.max_depth(), Flags::RANGE)?;
            let e = self.gen_expr(u, &idx_type, self.max_depth(), Flags::RANGE)?;
            // The random expressions might end up being huge to be practical for execution,
            // so take the modulo maximum range on both ends.
            let s = expr::range_modulo(s, idx_type.clone(), max_size);
            let e = expr::range_modulo(e, idx_type.clone(), max_size);
            (s, e)
        } else {
            // `gen_range` will choose a size up to the max.
            let max_size = self.ctx.config.max_loop_size;
            // If the function is constrained, we need a range we can determine at compile time.
            // For now do it with literals, although we should be able to use constant variables as well.
            let (s, e) = expr::gen_range(u, &idx_type, max_size)?;
            // The compiler allows the end to be lower than the start.
            if u.ratio(1, 5)? { (e, s) } else { (s, e) }
        };

        // Declare index variable, but only visible in the loop body, not the range.
        let idx_id = self.next_local_id();
        let idx_name = format!("idx_{}", make_name(idx_id.0 as usize, false));

        // Add a scope which will hold the index variable.
        self.locals.enter();
        self.locals.add(idx_id, false, idx_name.clone(), idx_type.clone());

        // Decrease budget so we don't nest for loops endlessly.
        self.decrease_budget(1);

        let was_in_loop = std::mem::replace(&mut self.in_loop, true);
        let block = self.gen_block(u, &Type::Unit)?;
        self.in_loop = was_in_loop;

        let expr = Expression::For(For {
            index_variable: idx_id,
            index_name: idx_name,
            index_type: idx_type,
            start_range: Box::new(start_range),
            end_range: Box::new(end_range),
            block: Box::new(block),
            start_range_location: Location::dummy(),
            end_range_location: Location::dummy(),
        });

        // Remove the loop scope.
        self.locals.exit();

        Ok(expr)
    }

    /// Generate a function call to any function in the global context except `main`,
    /// if the function returns the target type, or something we can use to produce that type.
    fn gen_call(
        &mut self,
        u: &mut Unstructured,
        typ: &Type,
        max_depth: usize,
    ) -> arbitrary::Result<Option<Expression>> {
        // Decrease the budget so we avoid a potential infinite nesting of calls.
        self.decrease_budget(1);

        let opts = self
            .call_targets
            .iter()
            .filter_map(|(id, types)| types.contains(typ).then_some(id))
            .collect::<Vec<_>>();

        if opts.is_empty() {
            return Ok(None);
        }

        // Remember that we will have made a call to something.
        self.has_call = true;

        let callee_id = *u.choose_iter(opts)?;
        let callee = self.ctx.function_decl(callee_id).clone();
        let param_types = callee.params.iter().map(|p| p.3.clone()).collect::<Vec<_>>();

        let mut args = Vec::new();
        for typ in &param_types {
            args.push(self.gen_expr(u, typ, max_depth, Flags::CALL)?);
        }

        let call_expr = Expression::Call(Call {
            func: Box::new(Expression::Ident(Ident {
                location: None,
                definition: Definition::Function(callee_id),
                mutable: false,
                name: callee.name.clone(),
                typ: Type::Function(
                    param_types,
                    Box::new(callee.return_type.clone()),
                    Box::new(Type::Unit),
                    callee.unconstrained,
                ),
                id: self.next_ident_id(),
            })),
            arguments: args,
            return_type: callee.return_type.clone(),
            location: Location::dummy(),
        });

        // Derive the final result from the call, e.g. by casting, or accessing a member.
        self.gen_expr_from_source(u, call_expr, &callee.return_type, typ, self.max_depth())
    }

    /// Generate a call to a specific function, with arbitrary literals
    /// for arguments (useful for generating comptime wrapper calls)
    fn gen_lit_call(
        &mut self,
        u: &mut Unstructured,
        callee_id: FuncId,
    ) -> arbitrary::Result<Expression> {
        let callee = self.ctx.function_decl(callee_id).clone();
        let param_types = callee.params.iter().map(|p| p.3.clone()).collect::<Vec<_>>();

        let mut args = Vec::new();
        for typ in &param_types {
            args.push(expr::gen_literal(u, typ)?);
        }

        let call_expr = Expression::Call(Call {
            func: Box::new(Expression::Ident(Ident {
                location: None,
                definition: Definition::Function(callee_id),
                mutable: false,
                name: callee.name.clone(),
                typ: Type::Function(
                    param_types,
                    Box::new(callee.return_type.clone()),
                    Box::new(Type::Unit),
                    callee.unconstrained,
                ),
                id: self.next_ident_id(),
            })),
            arguments: args,
            return_type: callee.return_type,
            location: Location::dummy(),
        });

        Ok(call_expr)
    }

    /// Generate a `loop` loop.
    fn gen_loop(&mut self, u: &mut Unstructured) -> arbitrary::Result<Expression> {
        // Declare break index variable visible in the loop body. Do not include it
        // in the locals the generator would be able to manipulate, as it could
        // lead to the loop becoming infinite.
        let idx_type = types::U32;
        let idx_local_id = self.next_local_id();
        let idx_id = self.next_ident_id();
        let idx_name = format!("idx_{}", make_name(idx_local_id.0 as usize, false));
        let idx_variable_id = VariableId::Local(idx_local_id);
        let idx_ident =
            expr::ident_inner(idx_variable_id, idx_id, true, idx_name.clone(), idx_type);
        let idx_expr = Expression::Ident(idx_ident.clone());

        // Decrease budget so we don't nest endlessly.
        self.decrease_budget(1);

        // Start building the loop harness, initialize index to 0
        let let_idx = expr::let_var(idx_local_id, true, idx_name, expr::u32_literal(0));

        // Get the randomized loop body
        let was_in_loop = std::mem::replace(&mut self.in_loop, true);
        let mut loop_body = self.gen_block(u, &Type::Unit)?;
        self.in_loop = was_in_loop;

        // Increment the index in the beginning of the body.
        expr::prepend(
            &mut loop_body,
            expr::assign_ident(
                idx_ident,
                expr::binary(idx_expr.clone(), BinaryOp::Add, expr::u32_literal(1)),
            ),
        );

        // Put everything into if/else
        let max_loop_size = self.gen_loop_size(u)?;
        let loop_body = expr::if_else(
            expr::binary(idx_expr, BinaryOp::Equal, expr::u32_literal(max_loop_size as u32)),
            Expression::Break,
            loop_body,
            Type::Unit,
        );

        Ok(Expression::Block(vec![let_idx, Expression::Loop(Box::new(loop_body))]))
    }

    /// Generate a `while` loop.
    fn gen_while(&mut self, u: &mut Unstructured) -> arbitrary::Result<Expression> {
        // Declare break index variable visible in the loop body. Do not include it
        // in the locals the generator would be able to manipulate, as it could
        // lead to the loop becoming infinite.
        let idx_type = types::U32;
        let idx_local_id = self.next_local_id();
        let idx_id = self.next_ident_id();
        let idx_name = format!("idx_{}", make_name(idx_local_id.0 as usize, false));
        let idx_variable_id = VariableId::Local(idx_local_id);
        let idx_ident =
            expr::ident_inner(idx_variable_id, idx_id, true, idx_name.clone(), idx_type);
        let idx_expr = Expression::Ident(idx_ident.clone());

        // Decrease budget so we don't nest endlessly.
        self.decrease_budget(1);

        // Start building the loop harness, initialize index to 0
        let mut stmts = vec![Expression::Let(Let {
            id: idx_local_id,
            mutable: true,
            name: idx_name,
            expression: Box::new(expr::u32_literal(0)),
        })];

        // Get the randomized loop body
        let was_in_loop = std::mem::replace(&mut self.in_loop, true);
        let mut loop_body = self.gen_block(u, &Type::Unit)?;
        self.in_loop = was_in_loop;

        // Increment the index in the beginning of the body.
        expr::prepend(
            &mut loop_body,
            expr::assign_ident(
                idx_ident,
                expr::binary(idx_expr.clone(), BinaryOp::Add, expr::u32_literal(1)),
            ),
        );

        // Put everything into if/else
        let max_loop_size = self.gen_loop_size(u)?;
        let inner_block = Expression::Block(vec![expr::if_else(
            expr::binary(idx_expr, BinaryOp::Equal, expr::u32_literal(max_loop_size as u32)),
            Expression::Break,
            loop_body,
            Type::Unit,
        )]);

        // Generate the `while` condition with depth 1
        let condition = self.gen_expr(u, &Type::Bool, 1, Flags::CONDITION)?;

        stmts.push(Expression::While(While {
            condition: Box::new(condition),
            body: Box::new(inner_block),
        }));

        Ok(Expression::Block(stmts))
    }

    /// Choose a random maximum guard size for `loop` and `while` to match the average of the size of a `for`.
    fn gen_loop_size(&self, u: &mut Unstructured) -> arbitrary::Result<usize> {
        if self.ctx.config.vary_loop_size {
            u.choose_index(self.ctx.config.max_loop_size)
        } else {
            Ok(self.ctx.config.max_loop_size)
        }
    }

    /// If this is main, and we could have made a call to another function, but we didn't,
    /// ensure we do, so as not to let all the others we generate go to waste.
    fn gen_guaranteed_call_from_main(
        &mut self,
        u: &mut Unstructured,
    ) -> arbitrary::Result<Option<Expression>> {
        if self.is_main() && !self.has_call && !self.call_targets.is_empty() {
            // Choose a type we'll return.
            let opts = self.call_targets.values().fold(BTreeSet::new(), |mut acc, types| {
                acc.extend(types.iter());
                acc
            });
            let typ = (*u.choose_iter(opts.iter())?).clone();
            // Assign the result of the call to a variable we won't use.
            if let Some(call) = self.gen_call(u, &typ, self.max_depth())? {
                return Ok(Some(self.let_var(false, typ, call, false)));
            }
        }
        Ok(None)
    }
}

#[test]
fn test_loop() {
    let mut u = Unstructured::new(&[0u8; 1]);
    let mut ctx = Context::default();
    ctx.config.max_loop_size = 10;
    ctx.config.vary_loop_size = false;
    ctx.gen_main_decl(&mut u);
    let mut fctx = FunctionContext::new(&mut ctx, FuncId(0));
    fctx.budget = 2;
    let loop_code = format!("{}", fctx.gen_loop(&mut u).unwrap()).replace(" ", "");

    println!("{loop_code}");
    assert!(
        loop_code.starts_with(
            &r#"{
    let mut idx_a$l0 = 0;
    loop {
        if (idx_a$l0 == 10) {
            break
        } else {
            idx_a$l0 = (idx_a$l0 + 1);"#
                .replace(" ", "")
        )
    );
}

#[test]
fn test_while() {
    let mut u = Unstructured::new(&[0u8; 1]);
    let mut ctx = Context::default();
    ctx.config.max_loop_size = 10;
    ctx.config.vary_loop_size = false;
    ctx.gen_main_decl(&mut u);
    let mut fctx = FunctionContext::new(&mut ctx, FuncId(0));
    fctx.budget = 2;
    let while_code = format!("{}", fctx.gen_while(&mut u).unwrap()).replace(" ", "");

    println!("{while_code}");
    assert!(
        while_code.starts_with(
            &r#"{
    let mut idx_a$l0 = 0;
    while (!false) {
        if (idx_a$l0 == 10) {
            break
        } else {
            idx_a$l0 = (idx_a$l0 + 1)"#
                .replace(" ", "")
        )
    );
}
