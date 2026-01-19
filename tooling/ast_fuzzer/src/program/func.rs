use iter_extended::vecmap;
use nargo::errors::Location;
use std::{
    collections::{BTreeMap, BTreeSet, HashSet},
    fmt::Debug,
};
use strum::IntoEnumIterator;

use arbitrary::{Arbitrary, Unstructured};
use noirc_frontend::{
    ast::{IntegerBitSize, UnaryOp},
    hir_def::expr::Constructor,
    monomorphization::{
        append_printable_type_info_for_type,
        ast::{
            ArrayLiteral, Assign, BinaryOp, Call, Definition, Expression, For, FuncId, GlobalId,
            Ident, IdentId, Index, InlineType, LValue, Let, Literal, LocalId, Match, MatchCase,
            Parameters, Program, Type, While,
        },
    },
    shared::{Signedness, Visibility},
    signed_field::SignedField,
};

use super::{
    CallableId, Config, Context, VariableId, expr,
    freq::Freq,
    make_name,
    scope::{Scope, ScopeStack, Stack, Variable},
    types,
};

/// Use random strings to identify constraints.
const CONSTRAIN_MSG_TYPE: Type = Type::String(super::CONSTRAIN_MSG_LENGTH);

/// We need to track whether expressions are coming from dynamic program inputs.
type TrackedExpression = (Expression, bool);

/// Something akin to a forward declaration of a function, capturing the details required to:
/// 1. call the function from the other function bodies
/// 2. generate the final HIR function signature
#[derive(Debug, Clone)]
pub(super) struct FunctionDeclaration {
    pub name: String,
    pub params: Parameters,
    pub return_type: Type,
    pub return_visibility: Visibility,
    pub inline_type: InlineType,
    pub unconstrained: bool,
}

impl FunctionDeclaration {
    /// Check if the return type contain a reference.
    pub(super) fn returns_refs(&self) -> bool {
        types::contains_reference(&self.return_type)
    }

    /// Check if the return type contains a vector.
    pub(super) fn returns_vectors(&self) -> bool {
        types::contains_vector(&self.return_type)
    }

    /// Check if any of the parameters or return value contain a reference.
    pub(super) fn has_refs(&self) -> bool {
        self.returns_refs()
            || self.params.iter().any(|(_, _, _, typ, _)| types::contains_reference(typ))
    }
}

/// Help avoid infinite recursion by limiting which function can call which other one.
pub(super) fn can_call(
    caller_id: FuncId,
    caller_unconstrained: bool,
    caller_returns_ref: bool,
    callee_id: FuncId,
    callee_decl: &FunctionDeclaration,
) -> bool {
    // Nobody should call `main`.
    if callee_id == Program::main_id() {
        return false;
    }

    // The compiler cannot handle returning references from `if-then-else` in ACIR.
    // Since the `limit` module currently inserts an `if ctx_limit == 0`,
    // returning a literal, it would violate this if the return has `&mut`,
    // therefore we don't make recursive calls from such functions, so the
    // limit strategy is not applied to them.
    if caller_returns_ref && caller_unconstrained {
        return false;
    }

    // From a Brillig function we restrict ourselves to only call
    // other Brillig functions. That's because the `Monomorphizer`
    // would make an unconstrained copy of any ACIR function called
    // from Brillig, and this is expected by the inliner for example,
    // but if we did similarly in the generator after we know who
    // calls who, we would incur two drawbacks:
    // 1) it would make programs bigger for little benefit
    // 2) it would skew calibration frequencies as ACIR freqs would overlay Brillig ones
    if caller_unconstrained {
        return callee_decl.unconstrained;
    }

    // When calling ACIR from ACIR, we avoid creating infinite
    // recursion by only calling functions with lower IDs,
    // otherwise the inliner could get stuck.
    if !callee_decl.unconstrained {
        // Higher calls lower, so we can use this rule to pick function parameters
        // as we create the declarations: we can pass functions already declared.
        return callee_id < caller_id;
    }

    // When calling Brillig from ACIR, we avoid calling functions that take or return
    // references or vectors, which cannot be passed between the two.
    !callee_decl.has_refs() && !callee_decl.returns_vectors()
}

/// Make a name for a local variable.
fn local_name(id: LocalId) -> String {
    make_name(id.0 as usize, false)
}

/// Make a name for a local index variable.
fn index_name(id: LocalId) -> String {
    format!("idx_{}", local_name(id))
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

/// Helper data structure for generating the lvalue of assignments.
struct LValueWithMeta {
    /// The lvalue to assign to, e.g. `a[i]`.
    lvalue: LValue,
    /// The type of the value that needs to be assigned, e.g. an array item type.
    typ: Type,
    /// Indicate whether any dynamic input was used to generate the lvalue, e.g. for an array index.
    /// This does not depend on whether the variable that we assign to was dynamic *before* the assignment.
    is_dyn: bool,
    /// Indicate whether we are assigning to just a part of a complex type.
    is_compound: bool,
    /// Any statements that had to be broken out to control the side effects of indexing.
    statements: Option<Vec<Expression>>,
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
    /// Indicate which local variables are derived from function inputs.
    dynamics: Stack<im::HashMap<LocalId, bool>>,
    /// Indicator of being in a loop (and hence able to generate
    /// break and continue statements)
    in_loop: bool,
    /// Indicator of computing an expression that should not contain dynamic input.
    in_no_dynamic: bool,
    /// Indicator of being affected by dynamic input, in which case we should refrain
    /// from using expression that requires no-dynamic mode.
    in_dynamic: bool,
    /// All the functions callable from this one, with the types we can
    /// produce from their return value.
    call_targets: BTreeMap<CallableId, HashSet<Type>>,
    /// Indicate that we have generated a `Call`.
    has_call: bool,
}

impl<'a> FunctionContext<'a> {
    pub(super) fn new(ctx: &'a mut Context, id: FuncId) -> Self {
        let decl = ctx.function_decl(id);
        let next_local_id = decl.params.iter().map(|p| p.0.0 + 1).max().unwrap_or_default();
        let budget = ctx.config.max_function_size;

        let globals = Scope::from_variables(
            ctx.globals
                .iter()
                .map(|(id, (name, typ, _expr))| (*id, false, name.clone(), typ.clone())),
        );

        // The function parameters are the base layer for local variables.
        let locals = ScopeStack::from_variables(
            decl.params
                .iter()
                .map(|(id, mutable, name, typ, _vis)| (*id, *mutable, name.clone(), typ.clone())),
        );

        // Function parameters are by definition considered to be dynamic input.
        let dynamics = Stack::new(locals.current().variable_ids().map(|id| (*id, true)).collect());

        // Collect all the functions we can call from this one.
        let mut call_targets = BTreeMap::new();

        // Consider calling any allowed global function.
        for (callee_id, callee_decl) in &ctx.function_declarations {
            if !can_call(id, decl.unconstrained, decl.returns_refs(), *callee_id, callee_decl) {
                continue;
            }
            let produces = types::types_produced(&callee_decl.return_type);
            call_targets.insert(CallableId::Global(*callee_id), produces);
        }

        // Consider function pointers as callable; they are already filtered during construction.
        for (callee_id, _, _, typ, _) in &decl.params {
            let Type::Function(_, return_type, _, _) = types::unref(typ) else {
                continue;
            };
            let produces = types::types_produced(return_type);
            call_targets.insert(CallableId::Local(*callee_id), produces);
        }

        Self {
            ctx,
            id,
            next_local_id,
            budget,
            globals,
            locals,
            dynamics,
            in_loop: false,
            in_no_dynamic: false,
            in_dynamic: false,
            call_targets,
            next_ident_id: 0,
            has_call: false,
        }
    }

    /// Generate the function body.
    pub(super) fn gen_body(mut self, u: &mut Unstructured) -> arbitrary::Result<Expression> {
        // If we don't limit the budget according to the available data,
        // it gives us a lot of `false` and 0 and we end up with deep `!(!false)` if expressions.
        self.budget = self.budget.min(u.len());
        let ret = self.decl().return_type.clone();
        let (mut body, _) = self.gen_expr(u, &ret, self.max_depth(), Flags::TOP)?;
        if let Some(call) = self.gen_guaranteed_call_from_main(u)? {
            expr::prepend(&mut body, call);
        }
        Ok(body)
    }

    /// Generate the function body, wrapping a function call with literal arguments.
    /// This is used to test comptime functions, which can only take those.
    pub(super) fn gen_body_with_lit_call(
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

    fn config(&self) -> &Config {
        &self.ctx.config
    }

    /// The default maximum depth to start from. We use `max_depth` to limit the
    /// complexity of expressions such as binary ones, array indexes, etc.
    fn max_depth(&self) -> usize {
        self.config().max_depth
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

    /// Enter a new local scope.
    fn enter_scope(&mut self) {
        self.locals.enter();
        self.dynamics.enter();
    }

    /// Exit the current local scope.
    fn exit_scope(&mut self) {
        self.locals.exit();
        self.dynamics.exit();
    }

    /// Check if a variable is derived from dynamic input.
    ///
    /// A variable can become statically known after re-assignment.
    fn is_dynamic(&self, id: &LocalId) -> bool {
        self.dynamics.current().get(id).cloned().unwrap_or_default()
    }

    /// Mark a variable as dynamic or not dynamic.
    fn set_dynamic(&mut self, id: LocalId, is_dynamic: bool) {
        // When a dynamic variable is assigned a constant value, only the current
        // scope and any future lower scopes are affected. After this scope we
        // will revert to whatever it was before.
        let is_new = self.dynamics.current_mut().insert(id, is_dynamic).is_none();

        // Becoming dynamic is contagious: if we assign a dynamic value to a mutable
        // variable in one of the branches of a conditional statement, we have to
        // consider it dynamic in the outer scopes as well from then on.
        if !is_new && is_dynamic {
            for layer in self.dynamics.iter_mut() {
                if layer.contains_key(&id) {
                    layer.insert(id, true);
                }
            }
        }
    }

    /// Check if a source type can be used inside a dynamic input context to produce some target type.
    fn can_be_used_in_dynamic(&self, src_type: &Type, tgt_type: &Type) -> bool {
        // Dynamic inputs are restricted only in ACIR
        self.unconstrained()
        // If we are looking for the exact type, it's okay.
            || src_type == tgt_type
            // But we can't index an array with references.
            || !(types::is_array_or_vector(src_type) && types::contains_reference(src_type) )
    }

    /// Choose a producer for a type, preferring local variables over global ones.
    fn choose_producer(
        &self,
        u: &mut Unstructured,
        typ: &Type,
    ) -> arbitrary::Result<Option<VariableId>> {
        // Check if we have something that produces this exact type.
        if u.ratio(7, 10)? {
            let producer = if self.in_no_dynamic || self.in_dynamic {
                self.locals.current().choose_producer_filtered(
                    u,
                    typ,
                    |id, (_, _, producer_type)| {
                        (!self.in_no_dynamic || !self.is_dynamic(id))
                            && (!self.in_dynamic || self.can_be_used_in_dynamic(producer_type, typ))
                    },
                )?
            } else {
                self.locals.current().choose_producer(u, typ)?
            };
            if let Some(id) = producer {
                return Ok(Some(VariableId::Local(id)));
            }
        }
        // If we're looking for a mutable reference, we have to choose some
        // mutable local variable and take a reference over it.
        // We can't use a global for this, because they are immutable.
        if let Type::Reference(typ, true) = typ {
            // Find an underlying mutable variable we can take a reference over.
            // We cannot have mutable references to array elements.
            return self
                .locals
                .current()
                .choose_producer_filtered(u, typ.as_ref(), |id, (mutable, _, producer_type)| {
                    *mutable
                        && (typ.as_ref() == producer_type
                            || !types::is_array_or_vector(producer_type))
                        && (!self.in_no_dynamic || !self.is_dynamic(id))
                        && (!self.in_dynamic
                            || self.can_be_used_in_dynamic(producer_type, typ.as_ref()))
                })
                .map(|id| id.map(VariableId::Local));
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

    /// Generate a literal expression of a certain type.
    fn gen_literal(&self, u: &mut Unstructured, typ: &Type) -> arbitrary::Result<Expression> {
        expr::gen_literal(u, typ, self.config())
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
    ) -> arbitrary::Result<TrackedExpression> {
        // For now if we need a function, return one without further nesting, e.g. avoid `if <cond> { func_1 } else { func_2 }`,
        // because it makes it harder to rewrite functions to add recursion limit: we would need to replace functions in the
        // expressions to proxy version if we call Brillig from ACIR, but we would also need to keep track whether we are calling a function,
        // For example if we could return function pointers, we could have something like this:
        //  `acir_func_1(if c { brillig_func_2 } else { unsafe { brillig_func_3(brillig_func_4) } })`
        // We could replace `brillig_func_2` with `brillig_func_2_proxy`, but we wouldn't replace `brillig_func_4` with `brillig_func_4_proxy`
        // because that is a parameter of another call. But we would have to deal with the return value.
        // For this reason we handle function parameters directly here.
        if types::is_function(types::unref(typ)) {
            if let Type::Reference(typ, _) = typ {
                // We might have an `&mut &mut fn(...) -> ...`; we recursively peel
                // off layers of references until we can generate an expression that
                // returns an `fn`, and then take a reference over it to restore.
                let (expr, is_dyn) = self.gen_expr(u, typ, max_depth, flags)?;
                // If the expression is a read-only global ident, then assign a variable first.
                let expr = if expr::is_immutable_ident(&expr) {
                    self.indirect_ref_mut((expr, is_dyn), typ.as_ref().clone())
                } else {
                    expr::ref_mut(expr, typ.as_ref().clone())
                };
                return Ok((expr, is_dyn));
            } else {
                // Prefer functions in variables over globals.
                return match self.gen_expr_from_vars(u, typ, max_depth)? {
                    Some(expr) => Ok(expr),
                    None => {
                        self.find_global_function_with_signature(u, typ).map(|expr| (expr, false))
                    }
                };
            }
        };

        let mut freq = Freq::new(u, &self.config().expr_freqs)?;

        // Stop nesting if we reached the bottom.
        let allow_nested = max_depth > 0;

        let allow_blocks = flags.allow_blocks
            && allow_nested
            && max_depth == self.config().max_depth
            && self.budget > 0;

        let allow_if_then = flags.allow_if_then
            && allow_nested
            && self.budget > 0
            && (self.unconstrained() || !types::contains_reference(typ));

        let allow_match = allow_if_then && !self.ctx.config.avoid_match;

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

        // Match expressions, returning a value.
        // Treating them similarly to if-then-else.
        if freq.enabled_when("match", allow_match) {
            // It might not be able to generate the type, if we don't have a suitable variable to match on.
            if let Some(expr) = self.gen_match(u, typ, max_depth)? {
                return Ok(expr);
            }
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

        // If nothing else worked out we can always produce a random literal.
        self.gen_literal(u, typ).map(|expr| (expr, false))
    }

    /// Try to generate an expression with a certain type out of the variables in scope.
    fn gen_expr_from_vars(
        &mut self,
        u: &mut Unstructured,
        typ: &Type,
        max_depth: usize,
    ) -> arbitrary::Result<Option<TrackedExpression>> {
        if let Some(id) = self.choose_producer(u, typ)? {
            let (src_mutable, src_name, src_type) = self.get_variable(&id).clone();
            let ident_id = self.next_ident_id();
            let src_expr = expr::ident(id, ident_id, src_mutable, src_name, src_type.clone());
            let src_dyn = match id {
                VariableId::Global(_) => false,
                VariableId::Local(id) => self.is_dynamic(&id),
            };
            if let Some(expr) = self.gen_expr_from_source(
                u,
                (src_expr, src_dyn),
                &src_type,
                src_mutable,
                typ,
                max_depth,
            )? {
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
                    let mut arr_dyn = false;
                    for _ in 0..*len {
                        let (item, item_dyn) =
                            self.gen_expr(u, item_type, max_depth, Flags::NESTED)?;
                        arr_dyn |= item_dyn;
                        arr.contents.push(item);
                    }
                    return Ok(Some((Expression::Literal(Literal::Array(arr)), arr_dyn)));
                }
                Type::Tuple(items) => {
                    let mut values = Vec::with_capacity(items.len());
                    let mut tup_dyn = false;
                    for item_type in items {
                        let (item, item_dyn) =
                            self.gen_expr(u, item_type, max_depth, Flags::NESTED)?;
                        tup_dyn |= item_dyn;
                        values.push(item);
                    }
                    return Ok(Some((Expression::Tuple(values), tup_dyn)));
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
    /// The `src_mutable` parameter indicates whether we can take a mutable reference over the source.
    ///
    /// Returns `None` if there is no way to produce the target from the source.
    fn gen_expr_from_source(
        &mut self,
        u: &mut Unstructured,
        (src_expr, src_dyn): TrackedExpression,
        src_type: &Type,
        mut src_mutable: bool,
        tgt_type: &Type,
        max_depth: usize,
    ) -> arbitrary::Result<Option<TrackedExpression>> {
        // If we found our type, we can return it without further ado.
        if src_type == tgt_type {
            // If we want a vector, we can push onto it.
            if let Type::Vector(item_type) = src_type {
                if bool::arbitrary(u)? {
                    let (item, item_dyn) = self.gen_expr(u, item_type, max_depth, Flags::TOP)?;
                    // We can use push_back, push_front, or insert.
                    if bool::arbitrary(u)? {
                        let push_expr = self.call_vector_push(
                            src_type.clone(),
                            item_type.as_ref().clone(),
                            src_expr,
                            bool::arbitrary(u)?,
                            item,
                        );
                        return Ok(Some((push_expr, src_dyn || item_dyn)));
                    } else {
                        // Generate a random index and insert the item at it.
                        return self.gen_vector_access(
                            u,
                            (src_expr, src_dyn || item_dyn),
                            src_type,
                            src_mutable,
                            tgt_type,
                            max_depth,
                            |this, ident, idx| {
                                this.call_vector_insert(
                                    src_type.clone(),
                                    item_type.as_ref().clone(),
                                    Expression::Ident(ident),
                                    idx,
                                    item,
                                )
                            },
                        );
                    }
                }
            }
            // Otherwise just return as-is.
            return Ok(Some((src_expr, src_dyn)));
        }

        // Some types cannot be accessed in certain contexts.
        if self.in_dynamic && !self.can_be_used_in_dynamic(src_type, tgt_type) {
            return Ok(None);
        }

        // Mutable references to array elements are currently unsupported.
        if types::is_array_or_vector(src_type) {
            src_mutable = false;
        }

        // Cast the source into the target type.
        let src_as_tgt = || {
            let expr = expr::cast(src_expr.clone(), tgt_type.clone());
            Ok(Some((expr, src_dyn)))
        };

        // See how we can produce tgt from src.
        match (src_type, tgt_type) {
            // Simple numeric conversions.
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
            // Dereference right into the target type.
            (Type::Reference(typ, _), _) if typ.as_ref() == tgt_type => {
                let expr = expr::deref(src_expr, tgt_type.clone());
                Ok(Some((expr, src_dyn)))
            }
            // Mutable reference over the source type.
            (_, Type::Reference(typ, true)) if typ.as_ref() == src_type => {
                let expr = if src_mutable {
                    expr::ref_mut(src_expr, typ.as_ref().clone())
                } else {
                    self.indirect_ref_mut((src_expr, src_dyn), typ.as_ref().clone())
                };
                Ok(Some((expr, src_dyn)))
            }
            // Index a non-empty array.
            (Type::Array(len, item_type), _) if *len > 0 => {
                // Indexing arrays that contains references with dynamic indexes was banned in #8888
                // If we are already looking for an index where we can't use dynamic inputs,
                // don't switch to using them again, as the result can indirectly poison the outer array.
                // For example this would be wrong:
                //
                // fn main(i: u32) -> pub u32 {
                //     let a = [&mut 0, &mut 1];
                //     let b = [0, 1];
                //     *a[b[i]]
                // }
                let (idx_expr, idx_dyn) = {
                    let no_dynamic = self.in_no_dynamic
                        || !self.unconstrained() && types::contains_reference(item_type);
                    let was_in_no_dynamic = std::mem::replace(&mut self.in_no_dynamic, no_dynamic);

                    // Choose a random index.
                    let (idx_expr, idx_dyn) = self.gen_index(u, *len, max_depth)?;
                    assert!(!(no_dynamic && idx_dyn), "non-dynamic index expected");

                    self.in_no_dynamic = was_in_no_dynamic;
                    (idx_expr, idx_dyn)
                };

                // Access the item.
                let item_expr = Expression::Index(Index {
                    collection: Box::new(src_expr),
                    index: Box::new(idx_expr),
                    element_type: *item_type.clone(),
                    location: Location::dummy(),
                });
                // Produce the target type from the item.
                self.gen_expr_from_source(
                    u,
                    (item_expr, src_dyn || idx_dyn),
                    item_type,
                    src_mutable,
                    tgt_type,
                    max_depth,
                )
            }
            // Pop from the front of a vector.
            (Type::Vector(item_type), Type::Tuple(fields))
                if fields.len() == 2
                    && &fields[0] == item_type.as_ref()
                    && &fields[1] == src_type =>
            {
                let pop_front = self.call_vector_pop(
                    src_type.clone(),
                    item_type.as_ref().clone(),
                    src_expr,
                    true,
                );
                Ok(Some((pop_front, src_dyn)))
            }
            // Pop from the back of a vector, or remove an item.
            (Type::Vector(item_type), Type::Tuple(fields))
                if fields.len() == 2
                    && &fields[0] == src_type
                    && &fields[1] == item_type.as_ref() =>
            {
                if bool::arbitrary(u)? {
                    let pop_back = self.call_vector_pop(
                        src_type.clone(),
                        item_type.as_ref().clone(),
                        src_expr,
                        false,
                    );
                    Ok(Some((pop_back, src_dyn)))
                } else {
                    self.gen_vector_access(
                        u,
                        (src_expr, src_dyn),
                        src_type,
                        src_mutable,
                        tgt_type,
                        max_depth,
                        |this, ident, idx| {
                            this.call_vector_remove(
                                src_type.clone(),
                                item_type.as_ref().clone(),
                                Expression::Ident(ident),
                                idx,
                            )
                        },
                    )
                }
            }
            // Index a vector (might fail at runtime if empty).
            (Type::Vector(item_type), _) => self.gen_vector_access(
                u,
                (src_expr, src_dyn),
                src_type,
                src_mutable,
                tgt_type,
                max_depth,
                |_, ident, idx| {
                    // Index the vector, represented by a variable, using the generated index.
                    Expression::Index(Index {
                        collection: Box::new(Expression::Ident(ident)),
                        index: Box::new(idx),
                        element_type: *item_type.clone(),
                        location: Location::dummy(),
                    })
                },
            ),
            // Extract a tuple field.
            (Type::Tuple(items), _) => {
                // Any of the items might be able to produce the target type.
                let mut opts = Vec::new();
                for (i, item_type) in items.iter().enumerate() {
                    let item_expr = Expression::ExtractTupleField(Box::new(src_expr.clone()), i);
                    if let Some(expr) = self.gen_expr_from_source(
                        u,
                        (item_expr, src_dyn),
                        item_type,
                        src_mutable,
                        tgt_type,
                        max_depth,
                    )? {
                        opts.push(expr);
                    }
                }
                if opts.is_empty() { Ok(None) } else { Ok(Some(u.choose_iter(opts)?)) }
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

    /// Generate code to index an arbitrary item in a vector.
    ///
    /// Since we don't know the length of the vector at compile time,
    /// this can involve creating a temporary variable, getting the length at runtime,
    /// and using modulo to limit some random index to the runtime length.
    #[allow(clippy::too_many_arguments)]
    fn gen_vector_access<F>(
        &mut self,
        u: &mut Unstructured,
        (src_expr, src_dyn): TrackedExpression,
        src_type: &Type,
        src_mutable: bool,
        tgt_type: &Type,
        max_depth: usize,
        access_item: F,
    ) -> arbitrary::Result<Option<TrackedExpression>>
    where
        F: FnOnce(&mut Self, Ident, Expression) -> Expression,
    {
        let Type::Vector(item_type) = src_type else {
            unreachable!("only expected to be called with Vector");
        };

        // Unless the vector is coming from an identifier or literal, we should create a let binding for it
        // to avoid doubling up any side effects, or double using local variables when it was created.
        let (let_expr, ident_1) = if let Expression::Ident(ident) = src_expr {
            (None, ident)
        } else {
            let (let_expr, let_ident) = self.let_var_and_ident(
                false,
                src_type.clone(),
                src_expr,
                false,
                src_dyn,
                local_name,
            );
            (Some(let_expr), let_ident)
        };

        // We'll need the ident again to access the item.
        let ident_2 = Ident { id: self.next_ident_id(), ..ident_1.clone() };

        // Get the runtime length.
        let len_expr = self.call_array_len(Expression::Ident(ident_1), src_type.clone());

        // The rules around dynamic indexing is the same as for arrays.
        let (idx_expr, idx_dyn) = if max_depth == 0 || bool::arbitrary(u)? {
            // Avoid any stack overflow where we look for an index in the vector itself.
            (self.gen_literal(u, &types::U32)?, false)
        } else {
            let no_dynamic =
                self.in_no_dynamic || !self.unconstrained() && types::contains_reference(item_type);
            let was_in_no_dynamic = std::mem::replace(&mut self.in_no_dynamic, no_dynamic);

            // Choose a random index.
            let (mut idx_expr, idx_dyn) =
                self.gen_expr(u, &types::U32, max_depth.saturating_sub(1), Flags::NESTED)?;

            assert!(!(no_dynamic && idx_dyn), "non-dynamic index expected");

            // Take the modulo, but leave a small chance for index OOB.
            if self.avoid_index_out_of_bounds(u)? {
                idx_expr = expr::modulo(idx_expr, len_expr);
            }

            self.in_no_dynamic = was_in_no_dynamic;
            (idx_expr, idx_dyn)
        };

        // Access the item by index
        let item_expr = access_item(self, ident_2, idx_expr);

        // Produce the target type from the item.
        let Some((expr, is_dyn)) = self.gen_expr_from_source(
            u,
            (item_expr, src_dyn || idx_dyn),
            item_type,
            src_mutable,
            tgt_type,
            max_depth,
        )?
        else {
            return Ok(None);
        };

        // Append the let and the final expression if we needed a block,
        // so we avoid suffixing a block with e.g. indexing, which would
        // not be parsable by the frontend. Another way to do this would
        // be to surround the block with parentheses.
        // So either of this should work:
        // * { let s = todo!(); s[123 % s.len()][456] }
        // * ( { let s = todo!(); s[123 % s.len()] } )[456]
        // But not this:
        // * { let s = todo!(); s[123 % s.len()] }[123]
        let expr = if let Some(let_expr) = let_expr {
            Expression::Block(vec![let_expr, expr])
        } else {
            expr
        };

        Ok(Some((expr, is_dyn)))
    }

    /// Generate an arbitrary index for an array.
    ///
    /// This can be either a random int literal, or a complex expression that produces an int.
    fn gen_index(
        &mut self,
        u: &mut Unstructured,
        len: u32,
        max_depth: usize,
    ) -> arbitrary::Result<TrackedExpression> {
        assert!(len > 0, "cannot index empty array");
        if max_depth > 0 && u.ratio(1, 3)? {
            let (mut idx, idx_dyn) =
                self.gen_expr(u, &types::U32, max_depth.saturating_sub(1), Flags::NESTED)?;

            // Limit the index to be in the valid range for the array length, with a small chance of index OOB.
            if self.avoid_index_out_of_bounds(u)? {
                idx = expr::index_modulo(idx, len);
            }

            Ok((idx, idx_dyn))
        } else {
            let idx = u.choose_index(len as usize)?;
            Ok((expr::u32_literal(idx as u32), false))
        }
    }

    /// Try to generate a unary expression of a certain type, if it's amenable to it, otherwise return `None`.
    fn gen_unary(
        &mut self,
        u: &mut Unstructured,
        typ: &Type,
        max_depth: usize,
    ) -> arbitrary::Result<Option<TrackedExpression>> {
        // Negation can cause overflow: for example `-1*i8::MIN` does not fit into `i8`, because `i8` is [-128, 127].
        let avoid_overflow = self.config().avoid_overflow || self.in_no_dynamic;

        let mut make_unary = |op| {
            self.gen_expr(u, typ, max_depth.saturating_sub(1), Flags::NESTED)
                .map(|(rhs, is_dyn)| Some((expr::unary(op, rhs, typ.clone()), is_dyn)))
        };

        if matches!(typ, Type::Field)
            || matches!(typ, Type::Integer(Signedness::Signed, _)) && !avoid_overflow
        {
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
    ) -> arbitrary::Result<Option<TrackedExpression>> {
        // Collect the operations can return the expected type.
        // Avoid operations that can fail in no-dynamic mode, otherwise they will be considered non-constant indexes.
        let ops = BinaryOp::iter()
            .filter(|op| {
                types::can_binary_op_return(op, typ)
                    && (!(self.config().avoid_overflow || self.in_no_dynamic)
                        || !types::can_binary_op_overflow(op))
                    && (!(self.config().avoid_err_by_zero || self.in_no_dynamic)
                        || !types::can_binary_op_err_by_zero(op))
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
            op: BinaryOp,
            type_out: &Type,
            scope: &'a Scope<K>,
        ) -> Vec<&'a Type> {
            scope
                .types_produced()
                .filter(|type_in| types::can_binary_op_return_from_input(&op, type_in, type_out))
                .collect::<Vec<_>>()
        }

        // Try local variables first.
        let mut lhs_opts = collect_input_types(op, typ, self.locals.current());

        // If the locals don't have any type compatible with `op`, try the globals.
        if lhs_opts.is_empty() {
            lhs_opts = collect_input_types(op, typ, &self.globals);
        }

        // We might not have any input that works for this operation.
        // We could generate literals, but that's not super interesting.
        if lhs_opts.is_empty() {
            return Ok(None);
        }

        // Choose a type for the LHS and RHS.
        let lhs_type = u.choose_iter(lhs_opts)?.clone();

        // Generate expressions for LHS and RHS.
        let (lhs_expr, lhs_dyn) =
            self.gen_expr(u, &lhs_type, max_depth.saturating_sub(1), Flags::NESTED)?;
        let (rhs_expr, rhs_dyn) =
            self.gen_expr(u, &lhs_type, max_depth.saturating_sub(1), Flags::NESTED)?;

        let mut expr = expr::binary(lhs_expr, op, rhs_expr);

        // If we have chosen e.g. u8 and need u32 we need to cast.
        if !(lhs_type == *typ || types::is_bool(typ) && op.is_comparator()) {
            expr = expr::cast(expr, typ.clone());
        }

        Ok(Some((expr, lhs_dyn || rhs_dyn)))
    }

    /// Generate a block of statements, finally returning a target type.
    ///
    /// This should always succeed, as we can always create a literal in the end.
    ///
    /// We might use dynamic input in the block and _not_ return a dynamic result from it. For example:
    /// ```ignore
    /// fn main(a: u32) -> u32 {
    ///     let b: u32 = {
    ///         let c = a + 1;
    ///         2
    ///     };
    ///     b
    /// }
    /// ```
    fn gen_block(
        &mut self,
        u: &mut Unstructured,
        typ: &Type,
    ) -> arbitrary::Result<TrackedExpression> {
        // The `max_depth` resets here, because that's only relevant in complex expressions.
        let max_depth = self.max_depth();
        let max_size = self.config().max_block_size.min(self.budget);

        // If we want blocks to be empty, or we don't have a budget for statements, just return an expression.
        if max_size == 0 {
            return self.gen_expr(u, typ, max_depth, Flags::TOP);
        }

        // Choose a positive number of statements.
        let size = u.int_in_range(1..=max_size)?;
        let mut stmts = Vec::with_capacity(size);
        // Only the last statement counts into whether the block is dynamic.
        let mut is_dyn = false;

        self.enter_scope();
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
            let (expr, expr_dyn) = self.gen_expr(u, typ, max_depth, Flags::TOP)?;
            is_dyn = expr_dyn;
            stmts.push(expr);
        }
        self.exit_scope();

        Ok((Expression::Block(stmts), is_dyn))
    }

    /// Generate a statement, which is an expression that doesn't return anything,
    /// for example loops, variable declarations, etc.
    ///
    /// Because the statement doesn't return anything (return unit) we don't track
    /// whether it used dynamic inputs; there is nothing to propagate.
    fn gen_stmt(&mut self, u: &mut Unstructured) -> arbitrary::Result<Expression> {
        let mut freq = if self.unconstrained() {
            Freq::new(u, &self.config().stmt_freqs_brillig)?
        } else {
            Freq::new(u, &self.config().stmt_freqs_acir)?
        };
        // TODO(#7926): Match

        // We don't want constraints to get too frequent, as it could dominate all outcome.
        if freq.enabled_when("constrain", !self.config().avoid_constrain) {
            if let Some(e) = self.gen_constrain(u)? {
                return Ok(e);
            }
        }

        // Require a positive budget, so that we have some for the block itself and its contents.
        if freq.enabled_when("if", self.budget > 1) {
            return self.gen_if(u, &Type::Unit, self.max_depth(), Flags::TOP).map(|(e, _)| e);
        }

        if freq.enabled_when("match", self.budget > 1 && !self.ctx.config.avoid_match) {
            if let Some((e, _)) = self.gen_match(u, &Type::Unit, self.max_depth())? {
                return Ok(e);
            }
        }

        if freq.enabled_when("for", self.budget > 1) {
            return self.gen_for(u);
        }

        if freq.enabled_when("call", self.budget > 0) {
            if let Some((e, _)) = self.gen_call(u, &Type::Unit, self.max_depth())? {
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

            if freq.enabled_when("break", self.in_loop && !self.config().avoid_loop_control) {
                return Ok(Expression::Break);
            }

            if freq.enabled_when("continue", self.in_loop && !self.config().avoid_loop_control) {
                return Ok(Expression::Continue);
            }

            // For now only try prints in unconstrained code, were we don't need to create a proxy.
            if freq.enabled_when("print", !self.config().avoid_print) {
                if let Some(e) = self.gen_print(u)? {
                    return Ok(e);
                }
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
        let comptime_friendly = self.config().comptime_friendly;
        let mut typ = self.ctx.gen_type(u, max_depth, false, false, comptime_friendly, true)?;

        // If we picked the target type to be a vector, we can consider popping from it.
        if let Type::Vector(ref item_type) = typ {
            if bool::arbitrary(u)? {
                let fields = if bool::arbitrary(u)? {
                    // ([T], T) <- pop_back or remove
                    vec![typ.clone(), item_type.as_ref().clone()]
                } else {
                    // (T, [T]) <- pop_front
                    vec![item_type.as_ref().clone(), typ.clone()]
                };
                typ = Type::Tuple(fields);
            }
        }

        let (expr, is_dyn) = self.gen_expr(u, &typ, max_depth, Flags::TOP)?;
        let mutable = bool::arbitrary(u)?;
        Ok(self.let_var(mutable, typ, expr, true, is_dyn, local_name))
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
        is_dynamic: bool,
        make_name: impl Fn(LocalId) -> String,
    ) -> Expression {
        let id = self.next_local_id();
        let name = make_name(id);

        // Add the variable so we can use it in subsequent expressions.
        if add_to_scope {
            self.locals.add(id, mutable, name.clone(), typ.clone());
        }

        self.set_dynamic(id, is_dynamic);

        expr::let_var(id, mutable, name, expr)
    }

    /// Add a new local variable and return a `Let` expression along with an `Ident` to refer it by.
    fn let_var_and_ident(
        &mut self,
        mutable: bool,
        typ: Type,
        expr: Expression,
        add_to_scope: bool,
        is_dynamic: bool,
        make_name: impl Fn(LocalId) -> String,
    ) -> (Expression, Ident) {
        let v = self.let_var(mutable, typ.clone(), expr, add_to_scope, is_dynamic, make_name);
        let Expression::Let(Let { id, name, .. }) = &v else {
            unreachable!("expected to Let; got {v:?}");
        };
        let i = expr::ident_inner(
            VariableId::Local(*id),
            self.next_ident_id(),
            mutable,
            name.clone(),
            typ,
        );
        (v, i)
    }

    /// Assign to a mutable variable, if we have one in scope.
    ///
    /// It resets the dynamic flag of the assigned variable.
    fn gen_assign(&mut self, u: &mut Unstructured) -> arbitrary::Result<Option<Expression>> {
        let opts = self
            .locals
            .current()
            .variables()
            .filter(|(_, (mutable, _, typ))| {
                // We banned reassigning variables which contain mutable references in ACIR (#8790)
                *mutable && (self.unconstrained() || !types::contains_reference(typ))
            })
            .map(|(id, _)| id)
            .collect::<Vec<_>>();

        if opts.is_empty() {
            return Ok(None);
        }

        let id = *u.choose_iter(opts)?;
        let ident = LValue::Ident(self.local_ident(id));
        let typ = self.local_type(id).clone();
        let lvalue = self.gen_lvalue(u, ident, typ)?;

        // Generate the assigned value.
        let (expr, expr_dyn) = self.gen_expr(u, &lvalue.typ, self.max_depth(), Flags::TOP)?;

        if lvalue.is_dyn || expr_dyn || self.in_dynamic {
            self.set_dynamic(id, true);
        } else if !lvalue.is_dyn && !expr_dyn && !lvalue.is_compound {
            // This value is no longer considered dynamic, unless we assigned to a member of an array or tuple,
            // in which case we don't know if other members have dynamic properties.
            self.set_dynamic(id, false);
        }

        let assign =
            Expression::Assign(Assign { lvalue: lvalue.lvalue, expression: Box::new(expr) });

        if let Some(mut statements) = lvalue.statements {
            statements.push(assign);
            Ok(Some(Expression::Block(statements)))
        } else {
            Ok(Some(assign))
        }
    }

    /// Generate an lvalue to assign to a local variable, or some part of it, if it's a compound type.
    ///
    /// Say we have an array: `a: [[u32; 2]; 3]`; we call it with `a`, and it might return `a`, `a[i]`, or `a[i][j]`.
    fn gen_lvalue(
        &mut self,
        u: &mut Unstructured,
        lvalue: LValue,
        typ: Type,
    ) -> arbitrary::Result<LValueWithMeta> {
        /// Accumulate statements for sub-indexes of multi-dimensional arrays.
        /// For example `a[1+2][3+4] = 5;` becomes `let i = 1+2; let j = 3+4; a[i][j] = 5;`
        fn merge_statements(
            a: Option<Vec<Expression>>,
            b: Option<Vec<Expression>>,
        ) -> Option<Vec<Expression>> {
            match (a, b) {
                (x, None) | (None, x) => x,
                (Some(mut a), Some(mut b)) => {
                    a.append(&mut b);
                    Some(a)
                }
            }
        }
        // For arrays and tuples we can consider assigning to their items.
        let lvalue = match typ {
            Type::Array(len, typ) if len > 0 && bool::arbitrary(u)? => {
                let (idx, idx_dyn) = self.gen_index(u, len, self.max_depth())?;

                // If the index expressions can have side effects, we need to assign it to a
                // temporary variable to match the sequencing done by the frontend; see #8384.
                // In the compiler the `Elaborator::fresh_definition_for_lvalue_index` decides.
                let needs_prefix = !matches!(idx, Expression::Ident(_) | Expression::Literal(_));

                let (idx, statements) = if needs_prefix {
                    let (let_idx, idx_ident) =
                        self.let_var_and_ident(false, types::U32, idx, false, idx_dyn, index_name);
                    (Expression::Ident(idx_ident), Some(vec![let_idx]))
                } else {
                    (idx, None)
                };

                let typ = typ.as_ref().clone();
                let index = LValue::Index {
                    array: Box::new(lvalue),
                    index: Box::new(idx),
                    element_type: typ.clone(),
                    location: Location::dummy(),
                };

                let mut lvalue = self.gen_lvalue(u, index, typ)?;
                lvalue.is_compound = true;
                lvalue.is_dyn |= idx_dyn;
                lvalue.statements = merge_statements(statements, lvalue.statements);
                lvalue
            }
            Type::Tuple(items) if bool::arbitrary(u)? => {
                let idx = u.choose_index(items.len())?;
                let typ = items[idx].clone();
                let member = LValue::MemberAccess { object: Box::new(lvalue), field_index: idx };
                let mut lvalue = self.gen_lvalue(u, member, typ)?;
                lvalue.is_compound = true;
                lvalue
            }
            typ => {
                LValueWithMeta { lvalue, typ, is_dyn: false, is_compound: false, statements: None }
            }
        };

        Ok(lvalue)
    }

    /// Generate a `println` statement, if there is some printable local variable.
    ///
    /// For now this only works in unconstrained code. For constrained code we will
    /// need to generate a proxy function, which we can do as a follow-up pass,
    /// as it has to be done once per function signature.
    fn gen_print(&mut self, u: &mut Unstructured) -> arbitrary::Result<Option<Expression>> {
        let opts = self
            .locals
            .current()
            .variables()
            .filter_map(|(id, (_, _, typ))| types::is_printable(typ).then_some((id, typ)))
            // TODO(#10499): comptime function representations are at the moment just "(function)"
            // (disable printing functions if comptime_friendly is on)
            .filter(|(_, typ)| !types::is_function(typ) || !self.config().comptime_friendly)
            .collect::<Vec<_>>();

        if opts.is_empty() {
            return Ok(None);
        }

        // Print one of the variables as-is.
        let (id, typ) = u.choose_iter(opts)?;
        let id = *id;

        // The print oracle takes 2 parameters: the newline marker and the value,
        // but it takes 2 more arguments: the type descriptor and the format string marker,
        // which are inserted automatically by the monomorphizer.
        let param_types = vec![Type::Bool, typ.clone()];
        let hir_type = types::to_hir_type(typ);
        let ident = self.local_ident(id);

        // Functions need to be passed as a tuple.
        let arg = if types::is_function(&ident.typ) {
            Expression::Tuple(vec![
                Expression::Ident(ident),
                Expression::Ident(self.local_ident(id)),
            ])
        } else {
            Expression::Ident(ident)
        };

        let mut args = vec![
            expr::lit_bool(true), // include newline,
            arg,
        ];

        append_printable_type_info_for_type(hir_type, &mut args);

        let print_oracle_ident = Ident {
            location: None,
            definition: Definition::Oracle("print".to_string()),
            mutable: false,
            name: "print_oracle".to_string(),
            typ: Type::Function(param_types, Box::new(Type::Unit), Box::new(Type::Unit), true),
            id: self.next_ident_id(),
        };

        let call = Expression::Call(Call {
            func: Box::new(Expression::Ident(print_oracle_ident)),
            arguments: args,
            return_type: Type::Unit,
            location: Location::dummy(),
        });

        Ok(Some(call))
    }

    /// Generate a `constrain` statement, if there is some local variable we can do it on.
    ///
    /// Arbitrary constraints are very likely to fail, so we don't want too many of them,
    /// otherwise they might mask disagreements in return values.
    fn gen_constrain(&mut self, u: &mut Unstructured) -> arbitrary::Result<Option<Expression>> {
        // Generate a condition that evaluates to bool.
        let Some((cond, _)) = self.gen_binary(u, &Type::Bool, self.max_depth())? else {
            return Ok(None);
        };
        // Generate a unique message for the assertion, so it's easy to find which one failed.
        let msg = self.gen_literal(u, &CONSTRAIN_MSG_TYPE)?;
        let cons = Expression::Constrain(
            Box::new(cond),
            Location::dummy(),
            Some(Box::new((msg, types::to_hir_type(&CONSTRAIN_MSG_TYPE)))),
        );
        Ok(Some(cons))
    }

    /// Generate an if-then-else statement or expression.
    fn gen_if(
        &mut self,
        u: &mut Unstructured,
        typ: &Type,
        max_depth: usize,
        flags: Flags,
    ) -> arbitrary::Result<TrackedExpression> {
        // Decrease the budget so we avoid a potential infinite nesting of if expressions in the arms.
        self.decrease_budget(1);

        let (cond, cond_dyn) = self.gen_expr(u, &Type::Bool, max_depth, Flags::CONDITION)?;

        // If the `if` condition uses dynamic input, we cannot access certain constructs in the body in ACIR.
        // Note that this would be the case for `while` and `for` as well, however `while` is not used in ACIR,
        // and `for` has its own restriction of having to have compile-time boundaries, so this is only done where necessary.
        let in_dynamic = self.in_dynamic || cond_dyn;
        let was_in_dynamic = std::mem::replace(&mut self.in_dynamic, in_dynamic);

        let (cons, cons_dyn) = {
            if flags.allow_blocks {
                self.gen_block(u, typ)?
            } else {
                self.gen_expr(u, typ, max_depth, flags)?
            }
        };

        let alt = if types::is_unit(typ) && bool::arbitrary(u)? {
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

        self.in_dynamic = was_in_dynamic;
        let alt_dyn = alt.as_ref().is_some_and(|(_, d)| *d);
        let is_dyn = cond_dyn || cons_dyn || alt_dyn;

        Ok((expr::if_then(cond, cons, alt.map(|(a, _)| a), typ.clone()), is_dyn))
    }

    /// Generate a `for` loop.
    fn gen_for(&mut self, u: &mut Unstructured) -> arbitrary::Result<Expression> {
        // The index can be signed or unsigned int, 8 to 128 bits, except i128,
        // but currently the frontend expects it to be u32 unless it's declared as a separate variable.
        let idx_type = {
            let bit_size = if self.config().avoid_large_int_literals {
                IntegerBitSize::ThirtyTwo
            } else {
                u.choose(&[8, 16, 32, 64, 128]).map(|s| IntegerBitSize::try_from(*s).unwrap())?
            };

            Type::Integer(
                if bit_size == IntegerBitSize::HundredTwentyEight
                    || self.config().avoid_negative_int_literals
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
            let max_size = u.int_in_range(1..=self.config().max_loop_size)?;
            // Generate random expression.
            let (s, _) = self.gen_expr(u, &idx_type, self.max_depth(), Flags::RANGE)?;
            let (e, _) = self.gen_expr(u, &idx_type, self.max_depth(), Flags::RANGE)?;
            // The random expressions might end up being huge to be practical for execution,
            // so take the modulo maximum range on both ends.
            let s = expr::range_modulo(s, idx_type.clone(), max_size);
            let e = expr::range_modulo(e, idx_type.clone(), max_size);
            (s, e)
        } else {
            // `gen_range` will choose a size up to the max.
            let max_size = self.config().max_loop_size;
            // If the function is constrained, we need a range we can determine at compile time.
            // For now do it with literals, although we should be able to use constant variables as well.
            let (s, e) = expr::gen_range(u, &idx_type, max_size)?;
            // The compiler allows the end to be lower than the start.
            if u.ratio(1, 5)? { (e, s) } else { (s, e) }
        };

        // Declare index variable, but only visible in the loop body, not the range.
        let idx_id = self.next_local_id();
        let idx_name = index_name(idx_id);

        // Add a scope which will hold the index variable.
        self.enter_scope();
        self.locals.add(idx_id, false, idx_name.clone(), idx_type.clone());

        // Decrease budget so we don't nest for loops endlessly.
        self.decrease_budget(1);

        let was_in_loop = std::mem::replace(&mut self.in_loop, true);
        let (block, _) = self.gen_block(u, &Type::Unit)?;
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
        self.exit_scope();

        Ok(expr)
    }

    /// Generate a function call to any function in the global context except `main`,
    /// if the function returns the target type, or something we can use to produce that type.
    ///
    /// Whether a call is dynamic depends on whether it has dynamic arguments,
    /// and whether we are crossing an ACIR-to-Brillig boundary.
    fn gen_call(
        &mut self,
        u: &mut Unstructured,
        typ: &Type,
        max_depth: usize,
    ) -> arbitrary::Result<Option<TrackedExpression>> {
        // Decrease the budget so we avoid a potential infinite nesting of calls.
        self.decrease_budget(1);

        let opts = self
            .call_targets
            .iter()
            .filter(|(id, types)|
                // We need to be able to generate the type from what the function returns.
                types.contains(typ) &&
                // We might not be able to call this function, depending on context.
                !(self.in_no_dynamic && !self.unconstrained() && self.callable_signature(**id).2))
            .map(|(id, _)| id)
            .collect::<Vec<_>>();

        if opts.is_empty() {
            return Ok(None);
        }

        // Remember that we will have made a call to something.
        self.has_call = true;

        let callee_id = *u.choose_iter(opts)?;
        let callee_expr = self.callable_expr(callee_id);
        let (param_types, return_type, callee_unconstrained) = self.callable_signature(callee_id);

        // Generate an expression for each argument.
        let mut args = Vec::new();
        let mut call_dyn = !self.unconstrained() && callee_unconstrained;
        for typ in &param_types {
            let (arg, arg_dyn) = self.gen_expr(u, typ, max_depth, Flags::CALL)?;
            call_dyn |= arg_dyn;
            args.push(arg);
        }

        let call_expr = Expression::Call(Call {
            func: Box::new(callee_expr),
            arguments: args,
            return_type: return_type.clone(),
            location: Location::dummy(),
        });

        // Derive the final result from the call, e.g. by casting, or accessing a member.
        self.gen_expr_from_source(
            u,
            (call_expr, call_dyn),
            &return_type,
            true,
            typ,
            self.max_depth(),
        )
    }

    /// Generate a call to a specific function, with arbitrary literals
    /// for arguments (useful for generating comptime wrapper calls)
    fn gen_lit_call(
        &mut self,
        u: &mut Unstructured,
        callee_id: FuncId,
    ) -> arbitrary::Result<Expression> {
        let callee_ident = self.func_ident(callee_id);
        let (param_types, return_type, _) = self.callable_signature(CallableId::Global(callee_id));

        let mut args = Vec::new();
        for typ in &param_types {
            args.push(self.gen_literal(u, typ)?);
        }

        let call_expr = Expression::Call(Call {
            func: Box::new(Expression::Ident(callee_ident)),
            arguments: args,
            return_type,
            location: Location::dummy(),
        });

        Ok(call_expr)
    }

    /// Generate a `loop` loop.
    fn gen_loop(&mut self, u: &mut Unstructured) -> arbitrary::Result<Expression> {
        // Declare break index variable visible in the loop body. Do not include it
        // in the locals the generator would be able to manipulate, as it could
        // lead to the loop becoming infinite.
        let (idx_local_id, idx_name, idx_ident) = self.next_loop_index();
        let idx_expr = Expression::Ident(idx_ident.clone());

        // Decrease budget so we don't nest endlessly.
        self.decrease_budget(1);

        // Start building the loop harness, initialize index to 0
        let let_idx = expr::let_var(idx_local_id, true, idx_name, expr::u32_literal(0));

        // Get the randomized loop body
        let was_in_loop = std::mem::replace(&mut self.in_loop, true);
        let (mut loop_body, _) = self.gen_block(u, &Type::Unit)?;
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
        let (idx_local_id, idx_name, idx_ident) = self.next_loop_index();
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
        let (mut loop_body, _) = self.gen_block(u, &Type::Unit)?;
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
        let (condition, _) = self.gen_expr(u, &Type::Bool, 1, Flags::CONDITION)?;

        stmts.push(Expression::While(While {
            condition: Box::new(condition),
            body: Box::new(inner_block),
        }));

        Ok(Expression::Block(stmts))
    }

    /// Choose a random maximum guard size for `loop` and `while` to match the average of the size of a `for`.
    fn gen_loop_size(&self, u: &mut Unstructured) -> arbitrary::Result<usize> {
        if self.config().vary_loop_size {
            u.choose_index(self.config().max_loop_size)
        } else {
            Ok(self.config().max_loop_size)
        }
    }

    /// Generate a `match` expression, returning a given type.
    ///
    /// Match needs a variable; if we don't have one to produce the target type from, it returns `None`.
    fn gen_match(
        &mut self,
        u: &mut Unstructured,
        typ: &Type,
        max_depth: usize,
    ) -> arbitrary::Result<Option<TrackedExpression>> {
        // Decrease the budget so we avoid a potential infinite nesting of match expressions in the rows.
        self.decrease_budget(1);

        // Pick a variable that can produce the type we are looking for.
        let id = if types::is_unit(typ) {
            // If we are generating a statement (return unit), then let's just pick any local variable.
            if self.locals.current().is_empty() {
                None
            } else {
                let id = u.choose_iter(self.locals.current().variable_ids())?;
                Some(VariableId::Local(*id))
            }
        } else {
            self.choose_producer(u, typ)?
        };

        // If we have no viable candidate then do something else.
        let Some(id) = id else {
            return Ok(None);
        };

        // If we picked a global variable, we need to create a local binding first,
        // because the match only works with local variable IDs.
        // We also need to create a secondary local binding for a local variable,
        // because of how the ownership analysis works: it only tracks the use of
        // identifiers, but the `Match::variable_to_match` only contains a `LocalId`.
        // We could change it to an `Ident`, but that's not enough: when the ownership
        // inserts `Clone` expressions for all but the last use of an ident, it could
        // not do so with the `Match`, because it would need match on an `Expression`.
        let (src_id, src_name, src_typ, src_dyn, src_expr) = match id {
            VariableId::Global(id) => {
                let typ = self.globals.get_variable(&id).2.clone();
                // The source is a technical variable that we don't want to access in the match rows.
                let (id, name, let_expr) = self.indirect_global(id, false, false);
                (id, name, typ, false, let_expr)
            }
            VariableId::Local(id) => {
                let typ = self.local_type(id).clone();
                let (id, name, let_expr) = self.indirect_local(id, false, false);
                let is_dyn = self.is_dynamic(&id);
                (id, name, typ, is_dyn, let_expr)
            }
        };

        // We could add some filtering to `choose_producer`, but it's already complicated; maybe next time.
        if !types::can_be_matched(&src_typ) {
            return Ok(None);
        }

        // Similar to an `if` statement, if the source variable is dynamic, we can't do certain things in the body.
        let in_dynamic = self.in_dynamic || src_dyn;
        let was_in_dynamic = std::mem::replace(&mut self.in_dynamic, in_dynamic);

        let mut match_expr = Match {
            variable_to_match: (src_id, src_name),
            cases: vec![],
            default_case: None,
            typ: typ.clone(),
        };

        let num_cases = u.int_in_range(0..=self.ctx.config.max_match_cases)?;

        // The dynamic nature of the final expression depends on the source and the rules.
        let mut is_dyn = src_dyn;

        // Generate a number of rows, depending on what we can do with the source type.
        // See `MatchCompiler::compile_rows` for what is currently supported.
        let gen_default = match &src_typ {
            Type::Bool => {
                // There are only two possible values. Repeating one of them results in a warning,
                // but let's allow it just so we cover that case, since it's not an error.
                for _ in 0..num_cases {
                    let constructor = u.choose_iter([Constructor::True, Constructor::False])?;
                    let (branch, branch_dyn) = self.gen_expr(u, typ, max_depth, Flags::TOP)?;
                    is_dyn |= branch_dyn;
                    let case = MatchCase { constructor, arguments: Vec::new(), branch };
                    match_expr.cases.push(case);
                }

                // If we have a non-exhaustive match we have to have a default; otherwise it's optional.
                #[allow(clippy::mutable_key_type)]
                let cs =
                    match_expr.cases.iter().map(|c| c.constructor.clone()).collect::<HashSet<_>>();

                if cs.len() < 2 { true } else { bool::arbitrary(u)? }
            }
            Type::Field | Type::Integer(_, _) => {
                for _ in 0..num_cases {
                    let constructor = self.gen_num_match_constructor(u, &src_typ)?;
                    let (branch, branch_dyn) = self.gen_expr(u, typ, max_depth, Flags::TOP)?;
                    is_dyn |= branch_dyn;
                    let case = MatchCase { constructor, arguments: Vec::new(), branch };
                    match_expr.cases.push(case);
                }
                // We won't have an exhaustive match with random integers, so we need a default.
                true
            }
            Type::Tuple(item_types) => {
                // There is only one case in the AST that we can generate, which is to unpack the tuple
                // into its constituent fields. The compiler would do this, and then generate further
                // matches on individual fields. We don't do that here, just make the fields available.
                let constructor = Constructor::Tuple(vecmap(item_types, types::to_hir_type));
                let mut arguments = Vec::new();
                self.enter_scope();
                for item_type in item_types {
                    let item_id = self.next_local_id();
                    let item_name = format!("item_{}", local_name(item_id));
                    self.locals.add(item_id, false, item_name.clone(), item_type.clone());
                    self.set_dynamic(item_id, src_dyn);
                    arguments.push((item_id, item_name));
                }
                // Generate the original expression we wanted with the new arguments in scope.
                let (branch, branch_dyn) = self.gen_expr(u, typ, max_depth, Flags::TOP)?;
                is_dyn |= branch_dyn;
                let case = MatchCase { constructor, arguments, branch };
                match_expr.cases.push(case);
                self.exit_scope();
                // We must not generate a default, or the compiler will panic.
                false
            }
            other => {
                unreachable!("unexpected type to generate match for: ${other}");
            }
        };

        // Optionally generate a default case.
        if gen_default {
            let (default_expr, default_dyn) = self.gen_expr(u, typ, max_depth, Flags::TOP)?;
            is_dyn |= default_dyn;
            match_expr.default_case = Some(Box::new(default_expr));
        }

        self.in_dynamic = was_in_dynamic;
        let match_expr = Expression::Match(match_expr);
        let expr = Expression::Block(vec![src_expr, match_expr]);

        Ok(Some((expr, is_dyn)))
    }

    /// Generate a random field that can be used in the match constructor of a numeric type.
    fn gen_num_field(
        &mut self,
        u: &mut Unstructured,
        typ: &Type,
    ) -> arbitrary::Result<SignedField> {
        let literal = self.gen_literal(u, typ)?;
        let Expression::Literal(Literal::Integer(field, _, _)) = literal else {
            unreachable!("expected Literal::Integer; got {literal:?}");
        };
        Ok(field)
    }

    /// Generate a match constructor for a numeric type.
    fn gen_num_match_constructor(
        &mut self,
        u: &mut Unstructured,
        typ: &Type,
    ) -> arbitrary::Result<Constructor> {
        // TODO: Currently the parser does not seem to support the `Constructor::Range` syntax.
        // When it does, we should generate either a field, or a range.
        let constructor = Constructor::Int(self.gen_num_field(u, typ)?);
        Ok(constructor)
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
            if let Some((call, is_dyn)) = self.gen_call(u, &typ, self.max_depth())? {
                return Ok(Some(self.let_var(false, typ, call, false, is_dyn, local_name)));
            }
        }
        Ok(None)
    }

    /// Find a global function matching a type signature.
    ///
    /// For local functions we use `gen_expr_from_vars`.
    fn find_global_function_with_signature(
        &mut self,
        u: &mut Unstructured,
        typ: &Type,
    ) -> arbitrary::Result<Expression> {
        let Type::Function(param_types, return_type, _, unconstrained) = typ else {
            unreachable!(
                "find_function_with_signature should only be called with Type::Function; got {typ}"
            );
        };

        let candidates = self
            .ctx
            .function_declarations
            .iter()
            .skip(1) // Can't call main.
            .filter_map(|(func_id, func)| {
                let matches = func.return_type == *return_type.as_ref()
                    && func.unconstrained == *unconstrained
                    && func.params.len() == param_types.len()
                    && func.params.iter().zip(param_types).all(|((_, _, _, a, _), b)| a == b);

                matches.then_some(*func_id)
            })
            .collect::<Vec<_>>();

        if candidates.is_empty() {
            panic!("No candidate found for function type: {typ}");
        }

        let callee_id = u.choose_iter(candidates)?;
        let callee_ident = self.func_ident(callee_id);

        Ok(Expression::Ident(callee_ident))
    }

    /// Expression to use in a `Call` for the function (pointer).
    ///
    /// If the ID is something like `f: &mut &mut fn(...) -> ...` then it will return `(*(*f))`.
    fn callable_expr(&mut self, callee_id: CallableId) -> Expression {
        match callee_id {
            CallableId::Global(id) => Expression::Ident(self.func_ident(id)),
            CallableId::Local(id) => {
                fn deref_function(typ: &Type, ident: Ident) -> Expression {
                    match typ {
                        Type::Function(_, _, _, _) => Expression::Ident(ident),
                        Type::Reference(typ, _) => {
                            let inner = deref_function(typ.as_ref(), ident);
                            expr::deref(inner, typ.as_ref().clone())
                        }
                        other => {
                            unreachable!("expected function or function reference; got {other}")
                        }
                    }
                }
                let ident = self.local_ident(id);
                let typ = self.local_type(id);
                deref_function(typ, ident)
            }
        }
    }

    /// Identifier for a global function.
    fn func_ident(&mut self, callee_id: FuncId) -> Ident {
        let callee = self.ctx.function_decl(callee_id).clone();
        let param_types = callee.params.iter().map(|p| p.3.clone()).collect::<Vec<_>>();

        Ident {
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
        }
    }

    /// Identifier for a local variable.
    fn local_ident(&mut self, id: LocalId) -> Ident {
        let (mutable, name, typ) = self.locals.current().get_variable(&id).clone();
        let ident_id = self.next_ident_id();
        expr::ident_inner(VariableId::Local(id), ident_id, mutable, name, typ.clone())
    }

    /// Type of a local variable.
    fn local_type(&self, id: LocalId) -> &Type {
        let (_, _, typ) = self.locals.current().get_variable(&id);
        typ
    }

    /// Create a loop index variable.
    fn next_loop_index(&mut self) -> (LocalId, String, Ident) {
        let idx_type = types::U32;
        let idx_local_id = self.next_local_id();
        let idx_id = self.next_ident_id();
        let idx_name = format!("idx_{}", make_name(idx_local_id.0 as usize, false));
        let idx_variable_id = VariableId::Local(idx_local_id);
        let idx_ident =
            expr::ident_inner(idx_variable_id, idx_id, true, idx_name.clone(), idx_type);
        (idx_local_id, idx_name, idx_ident)
    }

    /// Get the parameter types and return type of a callable function.
    fn callable_signature(&self, callee_id: CallableId) -> (Vec<Type>, Type, bool) {
        match callee_id {
            CallableId::Global(id) => {
                let decl = self.ctx.function_decl(id);
                let return_type = decl.return_type.clone();
                let param_types = decl.params.iter().map(|p| p.3.clone()).collect::<Vec<_>>();
                (param_types, return_type, decl.unconstrained)
            }
            CallableId::Local(id) => {
                let (_, _, typ) = self.locals.current().get_variable(&id);
                let Type::Function(param_types, return_type, _, unconstrained) = types::unref(typ)
                else {
                    unreachable!("function pointers should have function type; got {typ}")
                };
                (param_types.clone(), return_type.as_ref().clone(), *unconstrained)
            }
        }
    }

    /// Create a block with a let binding, then return a mutable reference to it.
    /// This is used as a workaround when we need a mutable reference over an immutable value.
    fn indirect_ref_mut(&mut self, (expr, is_dyn): TrackedExpression, typ: Type) -> Expression {
        let (let_expr, let_ident) =
            self.let_var_and_ident(true, typ.clone(), expr.clone(), false, is_dyn, local_name);
        let ref_expr = expr::ref_mut(Expression::Ident(let_ident), typ);
        Expression::Block(vec![let_expr, ref_expr])
    }

    /// Create a local let binding over a global variable.
    ///
    /// Returns the local ID and the `Let` expression.
    fn indirect_global(
        &mut self,
        id: GlobalId,
        mutable: bool,
        add_to_scope: bool,
    ) -> (LocalId, String, Expression) {
        let (_, name, typ) = self.globals.get_variable(&id).clone();
        let ident_id = self.next_ident_id();
        let ident = expr::ident(VariableId::Global(id), ident_id, false, name, typ.clone());
        let let_expr = self.let_var(mutable, typ, ident, add_to_scope, false, local_name);
        let Expression::Let(Let { id, name, .. }) = &let_expr else {
            unreachable!("expected Let; got {let_expr:?}");
        };
        (*id, name.clone(), let_expr)
    }

    /// Create a local let binding over a local variable.
    ///
    /// Returns the local ID and the `Let` expression.
    fn indirect_local(
        &mut self,
        id: LocalId,
        mutable: bool,
        add_to_scope: bool,
    ) -> (LocalId, String, Expression) {
        let ident = self.local_ident(id);
        let is_dynamic = self.is_dynamic(&id);
        let let_expr = self.let_var(
            mutable,
            ident.typ.clone(),
            Expression::Ident(ident),
            add_to_scope,
            is_dynamic,
            local_name,
        );
        let Expression::Let(Let { id, name, .. }) = &let_expr else {
            unreachable!("expected Let; got {let_expr:?}");
        };
        (*id, name.clone(), let_expr)
    }

    /// Construct a `Call` to the `array_len` builtin function, calling it with the
    /// identifier of a vector or an array.
    fn call_array_len(&mut self, array_or_vector: Expression, typ: Type) -> Expression {
        let func_ident = Ident {
            location: None,
            definition: Definition::Builtin("array_len".to_string()),
            mutable: false,
            name: "len".to_string(),
            typ: Type::Function(vec![typ], Box::new(types::U32), Box::new(Type::Unit), false),
            id: self.next_ident_id(),
        };
        Expression::Call(Call {
            func: Box::new(Expression::Ident(func_ident)),
            arguments: vec![array_or_vector],
            return_type: types::U32,
            location: Location::dummy(),
        })
    }

    /// Construct a `Call` to one of the `vector_*` builtin functions.
    fn call_vector_builtin(
        &mut self,
        name: &str,
        return_type: Type,
        arg_types: Vec<Type>,
        args: Vec<Expression>,
    ) -> Expression {
        let func_ident = Ident {
            location: None,
            definition: Definition::Builtin(format!("vector_{name}")),
            mutable: false,
            name: name.to_string(),
            typ: Type::Function(
                arg_types,
                Box::new(return_type.clone()),
                Box::new(Type::Unit),
                false,
            ),
            id: self.next_ident_id(),
        };
        Expression::Call(Call {
            func: Box::new(Expression::Ident(func_ident)),
            arguments: args,
            return_type,
            location: Location::dummy(),
        })
    }

    /// Construct a `Call` to the `vector_push_front` or `vector_push_back` builtin function.
    fn call_vector_push(
        &mut self,
        vector_type: Type,
        item_type: Type,
        vector: Expression,
        is_front: bool,
        item: Expression,
    ) -> Expression {
        self.call_vector_builtin(
            if is_front { "push_front" } else { "push_back" },
            vector_type.clone(),
            vec![vector_type, item_type],
            vec![vector, item],
        )
    }

    /// Construct a `Call` to the `vector_pop_front` or `vector_pop_back` builtin function.
    fn call_vector_pop(
        &mut self,
        vector_type: Type,
        item_type: Type,
        vector: Expression,
        is_front: bool,
    ) -> Expression {
        let return_fields = if is_front {
            vec![item_type, vector_type.clone()]
        } else {
            vec![vector_type.clone(), item_type]
        };
        self.call_vector_builtin(
            if is_front { "pop_front" } else { "pop_back" },
            Type::Tuple(return_fields),
            vec![vector_type],
            vec![vector],
        )
    }

    /// Construct a `Call` to the `vector_remove` builtin function.
    fn call_vector_remove(
        &mut self,
        vector_type: Type,
        item_type: Type,
        vector: Expression,
        idx: Expression,
    ) -> Expression {
        self.call_vector_builtin(
            "remove",
            Type::Tuple(vec![vector_type.clone(), item_type]),
            vec![vector_type, types::U32],
            vec![vector, idx],
        )
    }

    /// Construct a `Call` to the `vector_insert` builtin function.
    fn call_vector_insert(
        &mut self,
        vector_type: Type,
        item_type: Type,
        vector: Expression,
        idx: Expression,
        item: Expression,
    ) -> Expression {
        self.call_vector_builtin(
            "insert",
            Type::Tuple(vec![vector_type.clone()]),
            vec![vector_type, types::U32, item_type],
            vec![vector, idx, item],
        )
    }

    /// Random decision whether to allow "Index out of bounds" errors to happen
    /// on a specific array or vector access operation.
    ///
    /// If [Config::avoid_index_out_of_bounds] is turned on, then this is always `true`.
    ///
    /// It also returns `true` when `in_no_dynamic` mode is on, because an overflowing
    /// index might not be simplified out of the SSA in ACIR, and end up being considered
    /// a dynamic index, and leave reference allocations until ACIR gen, where they fail.
    fn avoid_index_out_of_bounds(&self, u: &mut Unstructured) -> arbitrary::Result<bool> {
        if self.config().avoid_index_out_of_bounds || self.in_no_dynamic {
            return Ok(true);
        }
        // Avoid OOB with 90% chance.
        u.ratio(9, 10)
    }
}

#[cfg(test)]
mod tests {
    use arbitrary::Unstructured;
    use noirc_frontend::monomorphization::ast::FuncId;

    use crate::program::{Context, FunctionContext};

    #[test]
    fn test_loop() {
        let mut u = Unstructured::new(&[0u8; 1]);
        let mut ctx = Context::default();
        ctx.config.max_loop_size = 10;
        ctx.config.vary_loop_size = false;
        ctx.gen_main_decl(&mut u);
        let mut function_ctx = FunctionContext::new(&mut ctx, FuncId(0));
        function_ctx.budget = 2;
        let loop_code = format!("{}", function_ctx.gen_loop(&mut u).unwrap()).replace(" ", "");

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
        let mut function_ctx = FunctionContext::new(&mut ctx, FuncId(0));
        function_ctx.budget = 2;
        let while_code = format!("{}", function_ctx.gen_while(&mut u).unwrap()).replace(" ", "");

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
}
