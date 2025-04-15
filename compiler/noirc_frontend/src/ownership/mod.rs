//! This module implements the "ownership analysis" compiler pass on the
//! monomorphized AST. It is run after monomorphization and before SSA-gen.
//! At this point the monomorphized AST has no polymorphic types and all functions
//! are specialized into either constrained or unconstrained versions. This pass
//! only operates on unconstrained functions since only Noir's unconstrained runtime Brillig
//! has any notion of cloning a value - specifically arrays with their brillig-only reference counts.
//!
//! Note that this documentation may refer to cloning a value or incrementing an array's reference
//! count. These operations are equivalent on arrays. Cloning may be applied to any value and only
//! increments the reference counts of any arrays contained within (but not behind references or
//! inside nested arrays). This document also focuses on arrays but all reference count operations
//! on arrays are also performed on slices.
//!
//! Arrays in brillig have copy on write semantics which relies on us incrementing their
//! reference counts when they are shared in multiple places. Note that while Noir has references,
//! arrays can also be shared by value and we want to avoid clones when possible. This pass
//! clones arrays (increments their reference counts) in the following situations:
//! - Function parameters:
//!   - Any arrays behind a mutable reference `&mut [T; N]` will have their reference count
//!     incremented iff there was already a prior array of the same type passed into the same
//!     function. E.g. if there are two parameters of type `&mut [Field; 3]` we increment only
//!     the later. If there are 3 we increment the last two.
//!     - This applies within struct & tuple types as well. If a function only takes 1 struct
//!       parameter but that struct contains 2 or more mutable references to the same array
//!       type, we increment the reference count of each instance of the type after the first.
//!     - In the case of references to nested arrays, only the outer array has its reference count incremented.
//!   - Arrays taken by mutable value are always cloned, e.g. in `mut x: [u32; 3]`, `x` will
//!     have its reference count incremented.
//! - Let bindings (`let _ = <expression which returns an array>;`):
//!   - Binding an array to a let binding increments the reference count of the array unless
//!     the expression is an array literal in which case it is considered to be moved.
//! - Assignments (`x = <expression which returns an array>;`):
//!   - Similarly, assigning an array to an existing variable will also increment the reference
//!     count of the array unless it is an array literal.
//! - Array literals:
//!   - Arrays stored inside a nested array literal (e.g. both variables in `[array1, array2]`
//!     have their reference count incremented).
//!   - This does not apply to nested array literals since we know they are not referenced elsewhere.
//! - Extracting an array from another array (`let inner: [_; _] = array[0];`):
//!   - Extracting a nested array from its outer array will always increment the reference count
//!     of the nested array.
//!
//! Additionally we currently only decrement reference counts at the end of the function when
//! a parameter goes out of scope. These means reference counts likely trend upward over time
//! until the array is eventually mutated and it is reset back to 1.
//!
//! --------------------------------- EXPERIMENTAL OWNERSHIP RULES ---------------------------------
//!
//! This pass currently contains two sets of ownership rules. There is the current default set of
//! rules described above, and there is the set of rules enabled by `Context::experimental_ownership_feature`.
//! The experimental ownership rules aim to be less ad-hoc than the current rules with the goal of
//! making it easier for users to see where clones occur - and possibly forcing users to write
//! their own clones manually. These rules treat each variable roughly as a variable in Rust which
//! implements `Copy`:
//! - Variables are copied on each use, except for the last use where they are moved.
//!   - If a variable's last use is in a loop that it was not defined in, it is copied instead of moved.
//!   - The last use analysis isn't sophisticated on struct fields. It will count `a.b` and `a.c`
//!     both as uses of `a`. Even if both could conceptually be moved, only the last usage will be
//!     moved and the first (say `a.b`) will still be cloned.
//! - Dereferences always clone.
//! - Certain expressions will avoid cloning or delay where clones are performed:
//!   - Reference expressions `&e` will not clone `e` but may still clone variables used within.
//!     - E.g. `&foo.bar` will not clone but `&foo(bar)` may still clone `foo` or `bar`.
//!   - Dereferences will attempt to extract a field first if possible.
//!     - E.g. `(*self).b.c` is transformed to `*(self.b.c)` where the `*` operation also clones.
//!   - Ordinary member access will also delay clones.
//!     - E.g. `self.b.c` is compiled as `self.b.c.clone()` over `self.clone().b.c`
//!   - Array indexing `a[i]` will avoid cloning `a`. The extracted element is always cloned.
//!
//! Most of this logic is contained in this file except for the last use analysis which is in the
//! `last_uses` module. That module contains a separate pass run on each function before this pass
//! to find the last use of each local variable to identify where moves can occur.
use crate::{
    ast::UnaryOp,
    monomorphization::ast::{
        Definition, Expression, Function, Ident, IdentId, LValue, Let, Literal, LocalId,
        Parameters, Program, Type, Unary,
    },
};

use fxhash::FxHashMap as HashMap;
use fxhash::FxHashSet as HashSet;
use noirc_errors::Location;

mod last_uses;

impl Program {
    pub(crate) fn handle_ownership(
        mut self,
        next_local_id: u32,
        next_ident_id: u32,
        experimental_ownership_feature: bool,
    ) -> Self {
        let mut context = Context {
            experimental_ownership_feature,
            next_ident_id,
            next_local_id,
            variables_to_move: Default::default(),
        };

        for function in self.functions.iter_mut() {
            context.handle_ownership_in_function(function);
        }
        self
    }
}

struct Context {
    experimental_ownership_feature: bool,

    /// If `experimental_ownership_feature` is enabled, this contains each instance of a variable
    /// we should move instead of cloning.
    variables_to_move: HashMap<LocalId, Vec<IdentId>>,

    next_ident_id: u32,
    next_local_id: u32,
}

impl Context {
    fn next_ident_id(&mut self) -> IdentId {
        let id = self.next_ident_id;
        self.next_ident_id += 1;
        IdentId(id)
    }

    fn next_local_id(&mut self) -> LocalId {
        let id = self.next_local_id;
        self.next_local_id += 1;
        LocalId(id)
    }

    fn should_move(&self, definition: LocalId, variable: IdentId) -> bool {
        self.variables_to_move
            .get(&definition)
            .is_some_and(|instances_to_move| instances_to_move.contains(&variable))
    }

    fn handle_ownership_in_function(&mut self, function: &mut Function) {
        if !function.unconstrained {
            return;
        }

        if self.experimental_ownership_feature {
            self.variables_to_move = Self::find_last_uses_of_variables(function);
        }

        self.handle_expression(&mut function.body);

        if !self.experimental_ownership_feature {
            let new_bindings = self.collect_parameters_to_clone(&function.parameters);

            // Prepend new_bindings to the function body and insert drops for them at the end.
            if !new_bindings.is_empty() {
                let unit = Expression::Literal(Literal::Unit);
                let old_body = std::mem::replace(&mut function.body, unit);

                // Store anything we want to clone in let bindings first so when we later drop
                // them we know we're dropping the same instance rather than a fresh copy.
                let (mut new_body, new_idents) = self.create_let_bindings(new_bindings);

                // Now push the clones for each parameter
                for new_ident in &new_idents {
                    new_body.push(Expression::Clone(Box::new(new_ident.clone())));
                }

                // Insert a `let` for the returned value so we can insert drops after it
                let return_id = self.next_local_id();
                let return_let = Expression::Let(Let {
                    id: return_id,
                    mutable: false,
                    name: "return".to_string(),
                    expression: Box::new(old_body),
                });

                new_body.push(return_let);

                // Now drop each parameter we cloned
                for new_ident in new_idents {
                    new_body.push(Expression::Drop(Box::new(new_ident)));
                }

                // Finally, return the original return value we held on to
                new_body.push(Expression::Ident(Ident {
                    location: None,
                    definition: Definition::Local(return_id),
                    mutable: false,
                    name: "return".to_string(),
                    typ: function.return_type.clone(),
                    id: self.next_ident_id(),
                }));

                function.body = Expression::Block(new_body);
            }
        }
    }

    fn create_let_bindings(
        &mut self,
        bindings_to_create: Vec<(String, Type, Expression)>,
    ) -> (Vec<Expression>, Vec<Expression>) {
        let mut bindings = Vec::with_capacity(bindings_to_create.len());
        let mut idents = Vec::with_capacity(bindings_to_create.len());

        for (name, typ, expression) in bindings_to_create {
            let id = self.next_local_id();
            let expression = Box::new(expression);
            bindings.push(Expression::Let(Let {
                id,
                mutable: false,
                name: String::new(),
                expression,
            }));

            idents.push(Expression::Ident(Ident {
                location: None,
                definition: Definition::Local(id),
                mutable: false,
                name,
                typ,
                id: self.next_ident_id(),
            }));
        }

        (bindings, idents)
    }

    /// Returns a vector of new parameters to prepend clones to a function - if any.
    /// Note that these may be full expressions e.g. `*param.field` so they should
    /// be stored in a let binding before being cloned to ensure that a later drop
    /// would be to the same value.
    fn collect_parameters_to_clone(
        &mut self,
        parameters: &Parameters,
    ) -> Vec<(String, Type, Expression)> {
        let mut seen_array_types = HashSet::default();
        let mut new_bindings = Vec::new();

        for (parameter_id, mutable, name, parameter_type) in parameters {
            let parameter = Expression::Ident(Ident {
                location: None,
                definition: Definition::Local(*parameter_id),
                mutable: *mutable,
                name: name.clone(),
                typ: parameter_type.clone(),
                id: self.next_ident_id(),
            });

            // (by-value) Mutable parameters are always cloned. Otherwise, we have to recur on the type
            // to find a duplicate array types behind mutable references.
            let parameter = if *mutable {
                let name = name.clone();
                new_bindings.push((name, parameter_type.clone(), parameter));
                // disable cloning in recur_on_parameter, we already cloned
                None
            } else {
                Some(parameter)
            };

            recur_on_parameter(
                parameter,
                parameter_type,
                name,
                &mut seen_array_types,
                &mut new_bindings,
                false,
            );
        }

        new_bindings
    }
}

/// Recur on a parameter's type, digging into any struct fields, looking for references to arrays.
/// This will build up an Expression of the current parameter access we're doing, e.g. `*foo.bar`
/// would correspond to a parameter `foo` with struct field `bar` that is a reference to an array.
///
/// This function inserts a .clone() expression to any parameter arrays behind references with
/// repeated types since these may potentially be aliased by other parameters.
///
/// If `parameter` is `None` we'll still traverse the type to find any array types mentioned but we
/// will not issue any clones. This is required e.g. on a mutable by-value parameter like `mut x: ...`
/// since `x` may contain arrays internally that we'll need to remember in case there is another
/// parameter which uses them. E.g. `mut x: [Field; 2], y: &mut [Field; 2]`.
fn recur_on_parameter<'typ>(
    parameter: Option<Expression>,
    parameter_type: &'typ Type,
    parameter_name: &str,
    seen_array_types: &mut HashSet<&'typ Type>,
    new_bindings: &mut Vec<(String, Type, Expression)>,
    passed_reference: bool,
) {
    match parameter_type {
        // These types never contain arrays
        Type::Field | Type::Integer(..) | Type::Bool | Type::Unit | Type::Function(..) => (),

        Type::Array(..) | Type::Slice(_) | Type::String(_) | Type::FmtString(..) => {
            // If we've already seen this type and this is behind a reference
            if !seen_array_types.insert(parameter_type) && passed_reference {
                if let Some(parameter) = parameter {
                    new_bindings.push((
                        parameter_name.to_string(),
                        parameter_type.clone(),
                        parameter,
                    ));
                }
            }

            // Don't recur on the element type here, we rely on the reference count to already be
            // incremented in the nested array case when the nested array is created.
        }

        Type::Tuple(fields) => {
            for (i, field) in fields.iter().enumerate() {
                let expr = parameter.clone().map(|p| Expression::ExtractTupleField(Box::new(p), i));
                recur_on_parameter(
                    expr,
                    field,
                    parameter_name,
                    seen_array_types,
                    new_bindings,
                    passed_reference,
                );
            }
        }

        Type::Reference(element_type, _mutable) => {
            let expr = parameter.map(|parameter| {
                Expression::Unary(Unary {
                    operator: UnaryOp::Dereference { implicitly_added: true },
                    rhs: Box::new(parameter.clone()),
                    result_type: element_type.as_ref().clone(),
                    location: Location::dummy(), // TODO
                })
            });
            recur_on_parameter(
                expr,
                element_type,
                parameter_name,
                seen_array_types,
                new_bindings,
                true,
            );
        }
    }
}

impl Context {
    fn handle_expression(&mut self, expr: &mut Expression) {
        match expr {
            Expression::Ident(_) => self.handle_ident(expr),
            Expression::Literal(literal) => self.handle_literal(literal),
            Expression::Block(exprs) => {
                exprs.iter_mut().for_each(|expr| self.handle_expression(expr));
            }
            Expression::Unary(_) => self.handle_unary(expr),
            Expression::Binary(binary) => self.handle_binary(binary),
            Expression::Index(_) => self.handle_index(expr),
            Expression::Cast(cast) => self.handle_cast(cast),
            Expression::For(for_expr) => self.handle_for(for_expr),
            Expression::Loop(loop_expr) => self.handle_expression(loop_expr),
            Expression::While(while_expr) => self.handle_while(while_expr),
            Expression::If(if_expr) => self.handle_if(if_expr),
            Expression::Match(match_expr) => self.handle_match(match_expr),
            Expression::Tuple(elements) => self.handle_tuple(elements),
            Expression::ExtractTupleField(..) => self.handle_extract_expression(expr),
            Expression::Call(call) => self.handle_call(call),
            Expression::Let(let_expr) => self.handle_let(let_expr),
            Expression::Constrain(boolean, _location, msg) => self.handle_constrain(boolean, msg),
            Expression::Assign(assign) => self.handle_assign(assign),
            Expression::Semi(expr) => self.handle_expression(expr),
            // Clones & Drops are only inserted by this pass so we can assume any code they
            // contain is already handled
            Expression::Clone(_) => (),
            Expression::Drop(_) => (),
            Expression::Break => (),
            Expression::Continue => (),
        }
    }

    /// Handle the rhs of a `&expr` unary expression.
    /// When the experimental ownership flag is enabled variables and field accesses
    /// in these expressions are exempt from clones.
    fn handle_reference_expression(&mut self, expr: &mut Expression) {
        match expr {
            Expression::Ident(_) => (),
            Expression::Block(exprs) => {
                let len_minus_one = exprs.len().saturating_sub(1);
                for expr in exprs.iter_mut().take(len_minus_one) {
                    // In `&{ a; b; ...; z }` we're only taking the reference of `z`.
                    self.handle_expression(expr);
                }
                if let Some(expr) = exprs.last_mut() {
                    self.handle_reference_expression(expr);
                }
            }
            Expression::Unary(Unary { rhs, operator: UnaryOp::Dereference { .. }, .. }) => {
                self.handle_reference_expression(rhs);
            }
            Expression::ExtractTupleField(tuple, _index) => self.handle_reference_expression(tuple),

            // If we have something like `f(arg)` then we want to treat those variables normally
            // rather than avoid cloning them. So we shouldn't recur in `handle_reference_expression`.
            other => self.handle_expression(other),
        }
    }

    fn handle_extract_expression(&mut self, expr: &mut Expression) {
        let Expression::ExtractTupleField(tuple, index) = expr else {
            panic!("handle_extract_expression given non-extract expression {expr}");
        };

        if !self.experimental_ownership_feature {
            return self.handle_expression(tuple);
        }

        // When experimental ownership is enabled, we may clone identifiers. We want to avoid
        // cloning the entire object though if we're only accessing one field of it so we check
        // here to move the clone to the outermost extract expression instead.
        // E.g. we want to change `a.clone().b.c` to `a.b.c.clone()`.
        if let Some((should_clone, tuple_type)) = self.handle_extract_expression_rec(tuple) {
            if let Some(elements) = unwrap_tuple_type(tuple_type) {
                if should_clone && contains_array_or_str_type(&elements[*index]) {
                    clone_expr(expr);
                }
            }
        } else {
            self.handle_expression(tuple);
        }
    }

    /// Traverse an expression comprised of only identifiers, tuple field extractions, and
    /// dereferences returning whether we should clone the result and the type of that result.
    /// Returns None if a different expression variant was found.
    fn handle_extract_expression_rec(&mut self, expr: &mut Expression) -> Option<(bool, Type)> {
        match expr {
            Expression::Ident(ident) => {
                let should_clone = self.should_clone_ident(ident);
                Some((should_clone, ident.typ.clone()))
            }
            // Delay dereferences as well so we change `(*self).foo.bar` to `*(self.foo.bar)`
            Expression::Unary(Unary {
                rhs,
                operator: UnaryOp::Dereference { .. },
                result_type,
                ..
            }) => {
                self.handle_reference_expression(rhs);
                Some((true, result_type.clone()))
            }
            Expression::ExtractTupleField(tuple, index) => {
                let (should_clone, typ) = self.handle_extract_expression_rec(tuple)?;
                let mut elements = unwrap_tuple_type(typ)?;
                Some((should_clone, elements.swap_remove(*index)))
            }
            _ => None,
        }
    }

    /// Under the experimental alternate ownership scheme, whenever an ident is used it is
    /// always cloned unless it is the last use of the ident (not in a loop).
    fn should_clone_ident(&self, ident: &Ident) -> bool {
        if self.experimental_ownership_feature {
            if let Definition::Local(local_id) = &ident.definition {
                if contains_array_or_str_type(&ident.typ) && !self.should_move(*local_id, ident.id)
                {
                    return true;
                }
            }
        }
        false
    }

    fn handle_ident(&self, expr: &mut Expression) {
        let ident = match expr {
            Expression::Ident(ident) => ident,
            other => panic!("handle_ident given non-ident expr: {other}"),
        };

        if self.should_clone_ident(ident) {
            clone_expr(expr);
        }
    }

    /// - Array literals:
    ///   - Arrays stored inside a nested array literal (e.g. both variables in `[array1, array2]`
    ///     have their reference count incremented).
    ///   - This does not apply to nested array literals since we know they are not referenced elsewhere.
    fn handle_literal(&mut self, literal: &mut Literal) {
        match literal {
            Literal::Integer(..) | Literal::Bool(_) | Literal::Unit | Literal::Str(_) => (),

            Literal::FmtStr(_, _, captures) => self.handle_expression(captures),

            Literal::Array(array) | Literal::Slice(array) => {
                let element_type = array
                    .typ
                    .array_element_type()
                    .expect("Array literal should have an array type");

                if !self.experimental_ownership_feature && contains_array_or_str_type(element_type)
                {
                    // We have to clone nested arrays unless they are array literals
                    for element in array.contents.iter_mut() {
                        if !is_array_or_str_literal(element) {
                            clone_expr(element);
                        }
                    }
                }
            }
        }
    }

    fn handle_unary(&mut self, expr: &mut Expression) {
        let unary = match expr {
            Expression::Unary(unary) => unary,
            other => panic!("handle_unary given non-unary expression: {other}"),
        };

        // Don't clone `rhs` if this is a reference or dereference expression.
        // - If this is a reference expression `&rhs`, `rhs` by definition shouldn't be cloned
        // - If this is `*rhs` we're going to clone the extracted element instead
        if self.experimental_ownership_feature
            && matches!(unary.operator, UnaryOp::Reference { .. } | UnaryOp::Dereference { .. })
        {
            self.handle_reference_expression(&mut unary.rhs);
        } else {
            self.handle_expression(&mut unary.rhs);
        }

        if self.experimental_ownership_feature
            && matches!(unary.operator, UnaryOp::Dereference { .. })
            && contains_array_or_str_type(&unary.result_type)
        {
            clone_expr(expr);
        }
    }

    fn handle_binary(&mut self, binary: &mut crate::monomorphization::ast::Binary) {
        self.handle_expression(&mut binary.lhs);
        self.handle_expression(&mut binary.rhs);
    }

    /// - Extracting an array from another array (`let inner: [_; _] = array[0];`):
    ///   - Extracting a nested array from its outer array will always increment the reference count
    ///     of the nested array.
    fn handle_index(&mut self, index_expr: &mut Expression) {
        let crate::monomorphization::ast::Expression::Index(index) = index_expr else {
            panic!("handle_index should only be called with Index nodes");
        };

        if self.experimental_ownership_feature {
            // Don't clone the collection, cloning only the resulting element is cheaper
            self.handle_reference_expression(&mut index.collection);
        } else {
            self.handle_expression(&mut index.collection);
        }

        self.handle_expression(&mut index.index);

        if contains_array_or_str_type(&index.element_type) {
            clone_expr(index_expr);
        }
    }

    fn handle_cast(&mut self, cast: &mut crate::monomorphization::ast::Cast) {
        self.handle_expression(&mut cast.lhs);
    }

    fn handle_for(&mut self, for_expr: &mut crate::monomorphization::ast::For) {
        self.handle_expression(&mut for_expr.start_range);
        self.handle_expression(&mut for_expr.end_range);
        self.handle_expression(&mut for_expr.block);
    }

    fn handle_while(&mut self, while_expr: &mut crate::monomorphization::ast::While) {
        self.handle_expression(&mut while_expr.condition);
        self.handle_expression(&mut while_expr.body);
    }

    fn handle_if(&mut self, if_expr: &mut crate::monomorphization::ast::If) {
        self.handle_expression(&mut if_expr.condition);
        self.handle_expression(&mut if_expr.consequence);
        if let Some(alt) = &mut if_expr.alternative {
            self.handle_expression(alt);
        }
    }

    fn handle_match(&mut self, match_expr: &mut crate::monomorphization::ast::Match) {
        for case in &mut match_expr.cases {
            self.handle_expression(&mut case.branch);
        }

        if let Some(default_case) = &mut match_expr.default_case {
            self.handle_expression(default_case);
        }
    }

    fn handle_tuple(&mut self, elements: &mut [Expression]) {
        for elem in elements {
            self.handle_expression(elem);
        }
    }

    fn handle_call(&mut self, call: &mut crate::monomorphization::ast::Call) {
        self.handle_expression(&mut call.func);
        for arg in &mut call.arguments {
            self.handle_expression(arg);
        }
    }

    /// - Let bindings (`let _ = <expression which returns an array>;`):
    ///   - Binding an array to a let binding increments the reference count of the array unless
    ///     the expression is an array literal in which case it is considered to be moved.
    fn handle_let(&mut self, let_expr: &mut crate::monomorphization::ast::Let) {
        self.handle_expression(&mut let_expr.expression);

        if !self.experimental_ownership_feature && !is_array_or_str_literal(&let_expr.expression) {
            clone_expr(&mut let_expr.expression);
        }
    }

    fn handle_constrain(
        &mut self,
        boolean: &mut Expression,
        msg: &mut Option<Box<(Expression, crate::hir_def::types::Type)>>,
    ) {
        self.handle_expression(boolean);

        if let Some(msg) = msg {
            self.handle_expression(&mut msg.0);
        }
    }

    /// - Assignments (`x = <expression which returns an array>;`):
    ///   - Assigning an array to an existing variable will also increment the reference
    ///     count of the array unless it is an array literal.
    fn handle_assign(&mut self, assign: &mut crate::monomorphization::ast::Assign) {
        self.handle_lvalue(&mut assign.lvalue);
        self.handle_expression(&mut assign.expression);

        if !self.experimental_ownership_feature && !is_array_or_str_literal(&assign.expression) {
            clone_expr(&mut assign.expression);
        }
    }

    fn handle_lvalue(&mut self, lvalue: &mut LValue) {
        match lvalue {
            LValue::Ident(_) => (),
            LValue::Index { array, index, element_type: _, location: _ } => {
                self.handle_expression(index);
                self.handle_lvalue(array);
            }
            LValue::MemberAccess { object, field_index: _ } => {
                self.handle_lvalue(object);
            }
            LValue::Dereference { reference, element_type: _ } => {
                self.handle_lvalue(reference);
            }
        }
    }
}

/// Adds a `.clone()` to the given expression.
/// Note that this method should be careful not to actually duplicate the given expression
/// so that we do not duplicate any side-effects.
fn clone_expr(expr: &mut Expression) {
    let unit = Expression::Literal(Literal::Unit);
    let old_expr = std::mem::replace(expr, unit);
    *expr = Expression::Clone(Box::new(old_expr));
}

fn is_array_or_str_literal(expr: &Expression) -> bool {
    match expr {
        Expression::Literal(literal) => match literal {
            Literal::Integer(..) | Literal::Bool(_) | Literal::Unit => false,

            Literal::Array(_) | Literal::Slice(_) | Literal::Str(_) | Literal::FmtStr(..) => true,
        },
        Expression::Block(exprs) => {
            if let Some(expr) = exprs.last() {
                is_array_or_str_literal(expr)
            } else {
                false
            }
        }

        Expression::Unary(_)
        | Expression::Ident(_)
        | Expression::Binary(_)
        | Expression::Index(_)
        | Expression::Cast(_)
        | Expression::For(_)
        | Expression::Loop(_)
        | Expression::While(_)
        | Expression::If(_)
        | Expression::Match(_)
        | Expression::Tuple(_)
        | Expression::ExtractTupleField(_, _)
        | Expression::Call(_)
        | Expression::Let(_)
        | Expression::Constrain(..)
        | Expression::Assign(_)
        | Expression::Semi(_)
        | Expression::Clone(_)
        | Expression::Drop(_)
        | Expression::Break
        | Expression::Continue => false,
    }
}

fn contains_array_or_str_type(typ: &Type) -> bool {
    match typ {
        Type::Field
        | Type::Integer(..)
        | Type::Bool
        | Type::Unit
        | Type::Function(..)
        | Type::Reference(..) => false,

        Type::Array(_, _) | Type::String(_) | Type::FmtString(_, _) | Type::Slice(_) => true,

        Type::Tuple(elems) => elems.iter().any(contains_array_or_str_type),
    }
}

fn unwrap_tuple_type(typ: Type) -> Option<Vec<Type>> {
    match typ {
        Type::Tuple(elements) => Some(elements),
        // array accesses will automatically dereference so we do too
        Type::Reference(element, _) => unwrap_tuple_type(*element),
        _ => None,
    }
}
