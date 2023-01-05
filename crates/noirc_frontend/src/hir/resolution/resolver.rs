// Fix usage of intern and resolve
// In some places, we do intern, however in others we are resolving and interning
// Ideally, I want to separate the interning and resolving abstractly
// so separate functions, but combine them naturally
// This could be possible, if lowering, is given a mutable map/scope as a parameter.
// So that it can match Idents to Ids. This is close to what the Scope map looks like
// Except for the num_times_used parameter.
// We can instead have a map from Ident to Into<IdentId> and implement that trait on ResolverMeta
//
//
// XXX: Change mentions of intern to resolve. In regards to the above comment
//
// XXX: Resolver does not check for unused functions
#[derive(Debug, PartialEq, Eq)]
struct ResolverMeta {
    num_times_used: usize,
    ident: HirIdent,
}

use crate::hir_def::expr::{
    HirBinaryOp, HirBlockExpression, HirCallExpression, HirCastExpression,
    HirConstructorExpression, HirExpression, HirForExpression, HirIdent, HirIfExpression,
    HirIndexExpression, HirInfixExpression, HirLiteral, HirMemberAccess, HirMethodCallExpression,
    HirPrefixExpression,
};
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::rc::Rc;

use crate::graph::CrateId;
use crate::hir::def_map::{ModuleDefId, TryFromModuleDefId};
use crate::hir_def::stmt::{HirAssignStatement, HirLValue, HirPattern};
use crate::node_interner::{DefinitionId, ExprId, FuncId, NodeInterner, StmtId, StructId};
use crate::{
    hir::{def_map::CrateDefMap, resolution::path_resolver::PathResolver},
    BlockExpression, Expression, ExpressionKind, FunctionKind, Ident, Literal, NoirFunction,
    Statement,
};
use crate::{
    ArrayLiteral, Generics, LValue, NoirStruct, Path, Pattern, Shared, StructType, Type,
    TypeBinding, TypeExpression, TypeVariable, UnresolvedType, ERROR_IDENT,
};
use fm::FileId;
use iter_extended::vecmap;
use noirc_errors::{Location, Span, Spanned};

use crate::hir::scope::{
    Scope as GenericScope, ScopeForest as GenericScopeForest, ScopeTree as GenericScopeTree,
};
use crate::hir_def::{
    function::{FuncMeta, HirFunction, Param},
    stmt::{HirConstrainStatement, HirLetStatement, HirStatement},
};

use super::errors::ResolverError;

type Scope = GenericScope<String, ResolverMeta>;
type ScopeTree = GenericScopeTree<String, ResolverMeta>;
type ScopeForest = GenericScopeForest<String, ResolverMeta>;

pub struct Resolver<'a> {
    scopes: ScopeForest,
    path_resolver: &'a dyn PathResolver,
    def_maps: &'a HashMap<CrateId, CrateDefMap>,
    interner: &'a mut NodeInterner,
    errors: Vec<ResolverError>,
    file: FileId,

    /// Set to the current type if we're resolving an impl
    self_type: Option<StructId>,

    /// Contains a mapping of the current struct's generics to
    /// unique type variables if we're resolving a struct. Empty otherwise.
    generics: HashMap<Rc<String>, (TypeVariable, Span)>,
}

impl<'a> Resolver<'a> {
    pub fn new(
        interner: &'a mut NodeInterner,
        path_resolver: &'a dyn PathResolver,
        def_maps: &'a HashMap<CrateId, CrateDefMap>,
        file: FileId,
    ) -> Resolver<'a> {
        Self {
            path_resolver,
            def_maps,
            scopes: ScopeForest::new(),
            interner,
            self_type: None,
            generics: HashMap::new(),
            errors: Vec::new(),
            file,
        }
    }

    pub fn set_self_type(&mut self, self_type: Option<StructId>) {
        self.self_type = self_type;
    }

    fn push_err(&mut self, err: ResolverError) {
        self.errors.push(err)
    }

    /// Resolving a function involves interning the metadata
    /// interning any statements inside of the function
    /// and interning the function itself
    /// We resolve and lower the function at the same time
    /// Since lowering would require scope data, unless we add an extra resolution field to the AST
    pub fn resolve_function(
        mut self,
        func: NoirFunction,
    ) -> (HirFunction, FuncMeta, Vec<ResolverError>) {
        self.scopes.start_function();

        // Check whether the function has globals in the local module and add them to the scope
        self.resolve_local_globals();

        self.add_generics(func.def.generics.clone());

        let (hir_func, func_meta) = self.intern_function(func);
        let func_scope_tree = self.scopes.end_function();

        self.check_for_unused_variables_in_scope_tree(func_scope_tree);

        (hir_func, func_meta, self.errors)
    }

    fn check_for_unused_variables_in_scope_tree(&mut self, scope_decls: ScopeTree) {
        let mut unused_vars = Vec::new();
        for scope in scope_decls.0.into_iter() {
            Resolver::check_for_unused_variables_in_local_scope(scope, &mut unused_vars);
        }

        for unused_var in unused_vars.iter() {
            let definition_info = self.interner.definition(unused_var.id);
            let name = &definition_info.name;
            if name != ERROR_IDENT && !definition_info.is_global {
                let ident = Ident(Spanned::from(unused_var.location.span, name.to_owned()));
                self.push_err(ResolverError::UnusedVariable { ident });
            }
        }
    }

    fn check_for_unused_variables_in_local_scope(decl_map: Scope, unused_vars: &mut Vec<HirIdent>) {
        let unused_variables = decl_map.filter(|(variable_name, metadata)| {
            let has_underscore_prefix = variable_name.starts_with('_'); // XXX: This is used for development mode, and will be removed

            if metadata.num_times_used == 0 && !has_underscore_prefix {
                return true;
            }
            false
        });
        unused_vars.extend(unused_variables.map(|(_, meta)| meta.ident));
    }

    /// Run the given function in a new scope.
    fn in_new_scope<T, F: FnOnce(&mut Self) -> T>(&mut self, f: F) -> T {
        self.scopes.start_scope();
        let ret = f(self);
        let scope = self.scopes.end_scope();
        self.check_for_unused_variables_in_scope_tree(scope.into());
        ret
    }

    fn add_variable_decl(
        &mut self,
        name: Ident,
        mutable: bool,
        is_global: bool,
        rhs: Option<ExprId>,
    ) -> HirIdent {
        if is_global {
            return self.add_global_variable_decl(name, rhs);
        }

        let id = self.interner.push_definition(name.0.contents.clone(), mutable, is_global, rhs);
        let location = Location::new(name.span(), self.file);
        let ident = HirIdent { location, id };
        let resolver_meta = ResolverMeta { num_times_used: 0, ident };

        let scope = self.scopes.get_mut_scope();
        let old_value = scope.add_key_value(name.0.contents.clone(), resolver_meta);
        if let Some(old_value) = old_value {
            self.push_err(ResolverError::DuplicateDefinition {
                name: name.0.contents,
                first_span: old_value.ident.location.span,
                second_span: location.span,
            });
        }
        ident
    }

    fn add_global_variable_decl(&mut self, name: Ident, rhs: Option<ExprId>) -> HirIdent {
        let scope = self.scopes.get_mut_scope();
        let ident;
        let resolver_meta;

        // This check is necessary to maintain the same definition ids in the interner. Currently, each function uses a new resolver that has its own ScopeForest and thus global scope.
        // We must first check whether an existing definition ID has been inserted as otherwise there will be multiple definitions for the same global statement.
        // This leads to an error in evaluation where the wrong definition ID is selected when evaluating a statement using the global. The check below prevents this error.
        let mut stmt_id = None;
        let global = self.interner.get_all_globals();
        for (global_stmt_id, global_info) in global {
            if global_info.ident == name
                && global_info.local_id == self.path_resolver.local_module_id()
            {
                stmt_id = Some(global_stmt_id);
            }
        }

        if let Some(id) = stmt_id {
            let hir_let_stmt = self.interner.let_statement(&id);
            ident = hir_let_stmt.ident();
            resolver_meta = ResolverMeta { num_times_used: 0, ident };
        } else {
            let id = self.interner.push_definition(name.0.contents.clone(), false, true, rhs); // The rhs expr for a global is already interned in a separate map and scope
            let location = Location::new(name.span(), self.file);
            ident = HirIdent { location, id };
            resolver_meta = ResolverMeta { num_times_used: 0, ident };
        }

        let old_global_value = scope.add_key_value(name.0.contents.clone(), resolver_meta);
        if let Some(old_global_value) = old_global_value {
            self.push_err(ResolverError::DuplicateDefinition {
                name: name.0.contents.clone(),
                first_span: old_global_value.ident.location.span,
                second_span: name.span(),
            });
        }
        ident
    }

    // Checks for a variable having been declared before
    // variable declaration and definition cannot be separate in Noir
    // Once the variable has been found, intern and link `name` to this definition
    // return the IdentId of `name`
    //
    // If a variable is not found, then an error is logged and a dummy id
    // is returned, for better error reporting UX
    fn find_variable(&mut self, name: &Ident) -> HirIdent {
        // Find the definition for this Ident
        let scope_tree = self.scopes.current_scope_tree();
        let variable = scope_tree.find(&name.0.contents);

        let location = Location::new(name.span(), self.file);
        let id = if let Some(variable_found) = variable {
            variable_found.num_times_used += 1;
            variable_found.ident.id
        } else {
            self.push_err(ResolverError::VariableNotDeclared {
                name: name.0.contents.clone(),
                span: name.0.span(),
            });
            DefinitionId::dummy_id()
        };

        HirIdent { location, id }
    }

    pub fn intern_function(&mut self, func: NoirFunction) -> (HirFunction, FuncMeta) {
        let func_meta = self.extract_meta(&func);

        let hir_func = match func.kind {
            FunctionKind::Builtin | FunctionKind::LowLevel => HirFunction::empty(),
            FunctionKind::Normal => {
                let expr_id = self.intern_block(func.def.body);
                self.interner.push_expr_location(expr_id, func.def.span, self.file);
                HirFunction::unsafe_from_expr(expr_id)
            }
        };

        (hir_func, func_meta)
    }

    /// Translates an UnresolvedType into a Type and appends any
    /// freshly created TypeVariables created to new_variables.
    fn resolve_type_inner(&mut self, typ: UnresolvedType, new_variables: &mut Generics) -> Type {
        match typ {
            UnresolvedType::FieldElement(comptime) => Type::FieldElement(comptime),
            UnresolvedType::Array(size, elem) => {
                let resolved_size = match &size {
                    None => {
                        let id = self.interner.next_type_variable_id();
                        let typevar = Shared::new(TypeBinding::Unbound(id));
                        new_variables.push((id, typevar.clone()));

                        // 'Named'Generic is a bit of a misnomer here, we want a type variable that
                        // wont be bound over but this one has no name since we do not currently
                        // require users to explicitly be generic over array lengths.
                        Type::NamedGeneric(typevar, Rc::new("".into()))
                    }
                    Some(expr) => {
                        let len = self.eval_array_length(expr);
                        Type::Expression(TypeExpression::Constant(len))
                    }
                };
                let elem = Box::new(self.resolve_type_inner(*elem, new_variables));
                Type::Array(Box::new(resolved_size), elem)
            }
            UnresolvedType::Integer(comptime, sign, bits) => Type::Integer(comptime, sign, bits),
            UnresolvedType::Bool(comptime) => Type::Bool(comptime),
            UnresolvedType::Unit => Type::Unit,
            UnresolvedType::Unspecified => Type::Error,
            UnresolvedType::Error => Type::Error,
            UnresolvedType::Named(path, args) => {
                // Check if the path is a type variable first. We currently disallow generics on type
                // variables since this is what rust does.
                if args.is_empty() && path.segments.len() == 1 {
                    let name = &path.last_segment().0.contents;
                    if let Some((name, (var, _))) = self.generics.get_key_value(name) {
                        return Type::NamedGeneric(var.clone(), name.clone());
                    }
                }

                match self.lookup_struct(path) {
                    Some(definition) => {
                        let args = vecmap(args, |arg| self.resolve_type_inner(arg, new_variables));
                        Type::Struct(definition, args)
                    }
                    None => Type::Error,
                }
            }
            UnresolvedType::Tuple(fields) => {
                Type::Tuple(vecmap(fields, |field| self.resolve_type_inner(field, new_variables)))
            }
        }
    }

    fn get_ident_from_path(&mut self, path: Path) -> HirIdent {
        if path.segments.len() == 1 {
            match path.as_ident() {
                Some(identifier) => self.find_variable(identifier),
                None => {
                    self.push_err(ResolverError::PathIsNotIdent { span: path.span() });
                    let id = DefinitionId::dummy_id();
                    let location = Location::new(path.span(), self.file);
                    HirIdent { id, location }
                }
            }
        } else {
            let stmt_id = self.lookup_global(path.clone());
            let hir_let_stmt = self.interner.let_statement(&stmt_id);
            hir_let_stmt.ident()
        }
    }

    /// Translates an UnresolvedType to a Type
    fn resolve_type(&mut self, typ: UnresolvedType) -> Type {
        self.resolve_type_inner(typ, &mut vec![])
    }

    fn add_generics(&mut self, generics: Vec<Ident>) -> Generics {
        vecmap(generics, |generic| {
            // Map the generic to a fresh type variable
            let id = self.interner.next_type_variable_id();
            let typevar = Shared::new(TypeBinding::Unbound(id));
            let span = generic.0.span();

            // Check for name collisions of this generic
            let name = Rc::new(generic.0.contents.clone());
            if let Some(old) = self.generics.insert(name, (typevar.clone(), span)) {
                let span = generic.0.span();
                self.errors.push(ResolverError::DuplicateDefinition {
                    name: generic.0.contents,
                    first_span: old.1,
                    second_span: span,
                })
            }
            (id, typevar)
        })
    }

    pub fn resolve_struct_fields(
        mut self,
        unresolved: NoirStruct,
    ) -> (Generics, BTreeMap<Ident, Type>, Vec<ResolverError>) {
        let generics = self.add_generics(unresolved.generics);

        // Check whether the struct definition has globals in the local module and add them to the scope
        self.resolve_local_globals();

        let fields = unresolved
            .fields
            .into_iter()
            .map(|(ident, typ)| (ident, self.resolve_type(typ)))
            .collect();

        (generics, fields, self.errors)
    }

    fn resolve_local_globals(&mut self) {
        for (stmt_id, global_info) in self.interner.get_all_globals() {
            if global_info.local_id == self.path_resolver.local_module_id() {
                let global_stmt = self.interner.let_statement(&stmt_id);
                self.add_global_variable_decl(global_info.ident, Some(global_stmt.expression));
            }
        }
    }

    /// Extract metadata from a NoirFunction
    /// to be used in analysis and intern the function parameters
    fn extract_meta(&mut self, func: &NoirFunction) -> FuncMeta {
        let name = func.name().to_owned();

        let location = Location::new(func.name_ident().span(), self.file);
        let id = self.interner.push_definition(name, false, false, None);
        let name_ident = HirIdent { id, location };

        let attributes = func.attribute().cloned();

        assert_eq!(self.generics.len(), func.def.generics.len());
        let mut generics = vecmap(&func.def.generics, |generic| {
            // Always expect self.generics to contain all the generics of this function
            let typevar = self.generics.get(&generic.0.contents).unwrap();
            match &*typevar.0.borrow() {
                TypeBinding::Unbound(id) => (*id, typevar.0.clone()),
                TypeBinding::Bound(binding) => unreachable!(
                    "Expected {} to be unbound, but it is bound to {}",
                    generic, binding
                ),
            }
        });

        let mut parameters = vec![];
        let mut parameter_types = vec![];

        for (pattern, typ, visibility) in func.parameters().iter().cloned() {
            if func.name() != "main" && visibility == noirc_abi::AbiVisibility::Public {
                self.push_err(ResolverError::UnnecessaryPub { ident: func.name_ident().clone() })
            }

            let pattern = self.resolve_pattern(pattern, false, None);
            let typ = self.resolve_type_inner(typ, &mut generics);
            parameters.push(Param(pattern, typ.clone(), visibility));
            parameter_types.push(typ);
        }

        let return_type = Box::new(self.resolve_type(func.return_type()));

        if func.name() == "main"
            && *return_type != Type::Unit
            && func.def.return_visibility != noirc_abi::AbiVisibility::Public
        {
            self.push_err(ResolverError::NecessaryPub { ident: func.name_ident().clone() })
        }

        let mut typ = Type::Function(parameter_types, return_type);

        if !generics.is_empty() {
            typ = Type::Forall(generics, Box::new(typ));
        }

        FuncMeta {
            name: name_ident,
            kind: func.kind,
            attributes,
            location,
            typ,
            parameters: parameters.into(),
            return_visibility: func.def.return_visibility,
            has_body: !func.def.body.is_empty(),
        }
    }

    pub fn resolve_stmt(&mut self, stmt: Statement, is_global: bool) -> HirStatement {
        match stmt {
            Statement::Let(let_stmt) => {
                let expression = self.resolve_expression(let_stmt.expression);
                HirStatement::Let(HirLetStatement {
                    pattern: self.resolve_pattern(let_stmt.pattern, is_global, Some(expression)),
                    r#type: self.resolve_type(let_stmt.r#type),
                    expression,
                })
            }
            Statement::Constrain(constrain_stmt) => {
                let expr_id = self.resolve_expression(constrain_stmt.0);
                HirStatement::Constrain(HirConstrainStatement(expr_id, self.file))
            }
            Statement::Expression(expr) => HirStatement::Expression(self.resolve_expression(expr)),
            Statement::Semi(expr) => HirStatement::Semi(self.resolve_expression(expr)),
            Statement::Assign(assign_stmt) => {
                let identifier = self.resolve_lvalue(assign_stmt.lvalue);
                let expression = self.resolve_expression(assign_stmt.expression);
                let stmt = HirAssignStatement { lvalue: identifier, expression };
                HirStatement::Assign(stmt)
            }
            Statement::Error => HirStatement::Error,
        }
    }

    pub fn intern_stmt(&mut self, stmt: Statement, is_global: bool) -> StmtId {
        let hir_stmt = self.resolve_stmt(stmt, is_global);
        self.interner.push_stmt(hir_stmt)
    }

    fn resolve_lvalue(&mut self, lvalue: LValue) -> HirLValue {
        match lvalue {
            LValue::Ident(ident) => HirLValue::Ident(self.find_variable(&ident)),
            LValue::MemberAccess { object, field_name } => {
                let object = Box::new(self.resolve_lvalue(*object));
                HirLValue::MemberAccess { object, field_name, field_index: None }
            }
            LValue::Index { array, index } => {
                let array = Box::new(self.resolve_lvalue(*array));
                let index = self.resolve_expression(index);
                HirLValue::Index { array, index }
            }
        }
    }

    pub fn resolve_expression(&mut self, expr: Expression) -> ExprId {
        let hir_expr = match expr.kind {
            ExpressionKind::Ident(string) => {
                let span = expr.span;
                let ident: Ident = Spanned::from(span, string).into();
                let ident_id = self.find_variable(&ident);
                HirExpression::Ident(ident_id)
            }
            ExpressionKind::Literal(literal) => HirExpression::Literal(match literal {
                Literal::Bool(b) => HirLiteral::Bool(b),
                Literal::Array(ArrayLiteral::Standard(elems)) => {
                    HirLiteral::Array(vecmap(elems, |elem| self.resolve_expression(elem)))
                }
                Literal::Array(ArrayLiteral::Repeated { repeated_element, length }) => {
                    let len = self.eval_array_length(&length);
                    let elem = self.resolve_expression(*repeated_element);
                    HirLiteral::Array(vec![elem; len.try_into().unwrap()])
                }
                Literal::Integer(integer) => HirLiteral::Integer(integer),
                Literal::Str(str) => HirLiteral::Str(str),
            }),
            ExpressionKind::Prefix(prefix) => {
                let operator = prefix.operator;
                let rhs = self.resolve_expression(prefix.rhs);
                HirExpression::Prefix(HirPrefixExpression { operator, rhs })
            }
            ExpressionKind::Infix(infix) => {
                let lhs = self.resolve_expression(infix.lhs);
                let rhs = self.resolve_expression(infix.rhs);

                HirExpression::Infix(HirInfixExpression {
                    lhs,
                    operator: HirBinaryOp::new(infix.operator, self.file),
                    rhs,
                })
            }
            ExpressionKind::Call(call_expr) => {
                // Get the span and name of path for error reporting
                let func_id = self.lookup_function(call_expr.func_name);
                let arguments = vecmap(call_expr.arguments, |arg| self.resolve_expression(arg));
                HirExpression::Call(HirCallExpression { func_id, arguments })
            }
            ExpressionKind::MethodCall(call_expr) => {
                let method = call_expr.method_name;
                let object = self.resolve_expression(call_expr.object);
                let arguments = vecmap(call_expr.arguments, |arg| self.resolve_expression(arg));
                HirExpression::MethodCall(HirMethodCallExpression { arguments, method, object })
            }
            ExpressionKind::Cast(cast_expr) => HirExpression::Cast(HirCastExpression {
                lhs: self.resolve_expression(cast_expr.lhs),
                r#type: self.resolve_type(cast_expr.r#type),
            }),
            ExpressionKind::For(for_expr) => {
                let start_range = self.resolve_expression(for_expr.start_range);
                let end_range = self.resolve_expression(for_expr.end_range);
                let (identifier, block) = (for_expr.identifier, for_expr.block);

                // TODO: For loop variables are currently mutable by default since we haven't
                //       yet implemented syntax for them to be optionally mutable.
                let (identifier, block_id) = self.in_new_scope(|this| {
                    (
                        this.add_variable_decl(identifier, true, false, None),
                        this.resolve_expression(block),
                    )
                });

                HirExpression::For(HirForExpression {
                    start_range,
                    end_range,
                    block: block_id,
                    identifier,
                })
            }
            ExpressionKind::If(if_expr) => HirExpression::If(HirIfExpression {
                condition: self.resolve_expression(if_expr.condition),
                consequence: self.resolve_expression(if_expr.consequence),
                alternative: if_expr.alternative.map(|e| self.resolve_expression(e)),
            }),
            ExpressionKind::Index(indexed_expr) => HirExpression::Index(HirIndexExpression {
                collection: self.resolve_expression(indexed_expr.collection),
                index: self.resolve_expression(indexed_expr.index),
            }),
            ExpressionKind::Path(path) => {
                // If the Path is being used as an Expression, then it is referring to a global from a separate module
                // Otherwise, then it is referring to an Identifier
                // This lookup allows support of such statements: let x = foo::bar::SOME_GLOBAL + 10;
                // If the expression is a singular indent, we search the resolver's current scope as normal.
                let hir_ident = self.get_ident_from_path(path);
                HirExpression::Ident(hir_ident)
            }
            ExpressionKind::Block(block_expr) => self.resolve_block(block_expr),
            ExpressionKind::Constructor(constructor) => {
                let span = constructor.type_name.span();

                if let Some(typ) = self.lookup_struct(constructor.type_name) {
                    let type_id = typ.borrow().id;

                    HirExpression::Constructor(HirConstructorExpression {
                        type_id,
                        fields: self.resolve_constructor_fields(
                            type_id,
                            constructor.fields,
                            span,
                            Resolver::resolve_expression,
                        ),
                        r#type: typ,
                    })
                } else {
                    HirExpression::Error
                }
            }
            ExpressionKind::MemberAccess(access) => {
                // Validating whether the lhs actually has the rhs as a field
                // needs to wait until type checking when we know the type of the lhs
                HirExpression::MemberAccess(HirMemberAccess {
                    lhs: self.resolve_expression(access.lhs),
                    rhs: access.rhs,
                })
            }
            ExpressionKind::Error => HirExpression::Error,
            ExpressionKind::Tuple(elements) => {
                let elements = vecmap(elements, |elem| self.resolve_expression(elem));
                HirExpression::Tuple(elements)
            }
        };

        let expr_id = self.interner.push_expr(hir_expr);
        self.interner.push_expr_location(expr_id, expr.span, self.file);
        expr_id
    }

    fn resolve_pattern(
        &mut self,
        pattern: Pattern,
        is_global: bool,
        rhs: Option<ExprId>,
    ) -> HirPattern {
        self.resolve_pattern_mutable(pattern, None, is_global, rhs)
    }

    fn resolve_pattern_mutable(
        &mut self,
        pattern: Pattern,
        mutable: Option<Span>,
        is_global: bool,
        rhs: Option<ExprId>,
    ) -> HirPattern {
        match pattern {
            Pattern::Identifier(name) => {
                let id = self.add_variable_decl(name, mutable.is_some(), is_global, rhs);
                HirPattern::Identifier(id)
            }
            Pattern::Mutable(pattern, span) => {
                if let Some(first_mut) = mutable {
                    self.push_err(ResolverError::UnnecessaryMut { first_mut, second_mut: span })
                }

                let pattern = self.resolve_pattern_mutable(*pattern, Some(span), is_global, rhs);
                HirPattern::Mutable(Box::new(pattern), span)
            }
            Pattern::Tuple(fields, span) => {
                let fields = vecmap(fields, |field| {
                    self.resolve_pattern_mutable(field, mutable, is_global, rhs)
                });
                HirPattern::Tuple(fields, span)
            }
            Pattern::Struct(name, fields, span) => {
                let struct_id = self.lookup_type(name);
                let struct_type = self.get_struct(struct_id);
                let resolve_field = |this: &mut Self, pattern| {
                    this.resolve_pattern_mutable(pattern, mutable, is_global, rhs)
                };
                let fields =
                    self.resolve_constructor_fields(struct_id, fields, span, resolve_field);
                HirPattern::Struct(struct_type, fields, span)
            }
        }
    }

    /// Resolve all the fields of a struct constructor expression.
    /// Ensures all fields are present, none are repeated, and all
    /// are part of the struct.
    ///
    /// This is generic to allow it to work for constructor expressions
    /// and constructor patterns.
    fn resolve_constructor_fields<T, U>(
        &mut self,
        type_id: StructId,
        fields: Vec<(Ident, T)>,
        span: Span,
        mut resolve_function: impl FnMut(&mut Self, T) -> U,
    ) -> Vec<(Ident, U)> {
        let mut ret = Vec::with_capacity(fields.len());
        let mut seen_fields = HashSet::new();
        let mut unseen_fields = self.get_field_names_of_type(type_id);

        for (field, expr) in fields {
            let resolved = resolve_function(self, expr);

            if unseen_fields.contains(&field) {
                unseen_fields.remove(&field);
                seen_fields.insert(field.clone());
            } else if seen_fields.contains(&field) {
                // duplicate field
                self.push_err(ResolverError::DuplicateField { field: field.clone() });
            } else {
                // field not required by struct
                self.push_err(ResolverError::NoSuchField {
                    field: field.clone(),
                    struct_definition: self.get_struct(type_id).borrow().name.clone(),
                });
            }

            ret.push((field, resolved));
        }

        if !unseen_fields.is_empty() {
            self.push_err(ResolverError::MissingFields {
                span,
                missing_fields: unseen_fields.into_iter().map(|field| field.to_string()).collect(),
                struct_definition: self.get_struct(type_id).borrow().name.clone(),
            });
        }

        ret
    }

    pub fn get_struct(&self, type_id: StructId) -> Shared<StructType> {
        self.interner.get_struct(type_id)
    }

    fn get_field_names_of_type(&self, type_id: StructId) -> BTreeSet<Ident> {
        let typ = self.get_struct(type_id);
        let typ = typ.borrow();
        typ.field_names()
    }

    fn lookup<T: TryFromModuleDefId>(&mut self, path: Path) -> T {
        let span = path.span();
        match self.resolve_path(path) {
            // Could not resolve this symbol, the error is already logged, return a dummy function id
            None => T::dummy_id(),
            Some(def_id) => T::try_from(def_id).unwrap_or_else(|| {
                self.push_err(ResolverError::Expected {
                    expected: T::description(),
                    got: def_id.as_str().to_owned(),
                    span,
                });
                T::dummy_id()
            }),
        }
    }

    fn lookup_function(&mut self, path: Path) -> FuncId {
        self.lookup(path)
    }

    fn lookup_global(&mut self, path: Path) -> StmtId {
        self.lookup(path)
    }

    fn lookup_type(&mut self, path: Path) -> StructId {
        let ident = path.as_ident();
        if ident.map_or(false, |i| i == "Self") {
            if let Some(id) = &self.self_type {
                return *id;
            }
        }

        self.lookup(path)
    }

    pub fn lookup_struct(&mut self, path: Path) -> Option<Shared<StructType>> {
        let id = self.lookup_type(path);
        (id != StructId::dummy_id()).then(|| self.get_struct(id))
    }

    pub fn lookup_type_for_impl(mut self, path: Path) -> (StructId, Vec<ResolverError>) {
        (self.lookup_type(path), self.errors)
    }

    fn resolve_path(&mut self, path: Path) -> Option<ModuleDefId> {
        let span = path.span();
        let name = path.as_string();
        self.path_resolver.resolve(self.def_maps, path).unwrap_or_else(|segment| {
            self.push_err(ResolverError::PathUnresolved { name, span, segment });
            None
        })
    }

    fn resolve_block(&mut self, block_expr: BlockExpression) -> HirExpression {
        let statements =
            self.in_new_scope(|this| vecmap(block_expr.0, |stmt| this.intern_stmt(stmt, false)));
        HirExpression::Block(HirBlockExpression(statements))
    }

    pub fn intern_block(&mut self, block: BlockExpression) -> ExprId {
        let hir_block = self.resolve_block(block);
        self.interner.push_expr(hir_block)
    }

    fn eval_array_length(&mut self, length: &Expression) -> u64 {
        match self.try_eval_array_length(length).map(|length| length.try_into()) {
            Ok(Ok(length_value)) => return length_value,
            Ok(Err(_cast_err)) => {
                self.push_err(ResolverError::IntegerTooLarge { span: length.span })
            }
            Err(Some(error)) => self.push_err(error),
            Err(None) => (),
        }
        0
    }

    /// This function is a mini interpreter inside name resolution.
    /// We should eventually get rid of it and only have 1 evaluator - the existing
    /// one inside the ssa pass. Doing this would mean ssa would need to handle these
    /// sugared array forms but would let users use any comptime expressions, including functions,
    /// inside array lengths.
    fn try_eval_array_length(
        &mut self,
        length: &Expression,
    ) -> Result<u128, Option<ResolverError>> {
        let span = length.span;
        match &length.kind {
            ExpressionKind::Literal(Literal::Integer(int)) => {
                int.try_into_u128().ok_or(Some(ResolverError::IntegerTooLarge { span }))
            }
            ExpressionKind::Ident(ident) => {
                let ident: Ident = Spanned::from(span, ident.to_owned()).into();
                let ident = self.find_variable(&ident);
                self.try_eval_array_length_ident(ident.id, span)
            }
            ExpressionKind::Path(path) => {
                let ident = self.get_ident_from_path(path.clone());
                self.try_eval_array_length_ident(ident.id, span)
            }
            ExpressionKind::Prefix(operator) => {
                let value = self.try_eval_array_length(&operator.rhs)?;
                match operator.operator {
                    crate::UnaryOp::Minus => Ok(0 - value),
                    crate::UnaryOp::Not => Ok(!value),
                }
            }
            ExpressionKind::Infix(operator) => {
                let lhs = self.try_eval_array_length(&operator.lhs)?;
                let rhs = self.try_eval_array_length(&operator.rhs)?;
                match operator.operator.contents {
                    crate::BinaryOpKind::Add => Ok(lhs + rhs),
                    crate::BinaryOpKind::Subtract => Ok(lhs - rhs),
                    crate::BinaryOpKind::Multiply => Ok(lhs * rhs),
                    crate::BinaryOpKind::Divide => Ok(lhs / rhs),
                    crate::BinaryOpKind::ShiftRight => Ok(lhs >> rhs),
                    crate::BinaryOpKind::ShiftLeft => Ok(lhs << rhs),
                    crate::BinaryOpKind::Modulo => Ok(lhs % rhs),
                    crate::BinaryOpKind::And => Ok(lhs & rhs),
                    crate::BinaryOpKind::Or => Ok(lhs | rhs),
                    crate::BinaryOpKind::Xor => Ok(lhs ^ rhs),

                    crate::BinaryOpKind::Equal
                    | crate::BinaryOpKind::NotEqual
                    | crate::BinaryOpKind::Less
                    | crate::BinaryOpKind::LessEqual
                    | crate::BinaryOpKind::Greater
                    | crate::BinaryOpKind::GreaterEqual => {
                        Err(Some(ResolverError::InvalidArrayLengthExpr { span }))
                    }
                }
            }

            ExpressionKind::Literal(_)
            | ExpressionKind::Block(_)
            | ExpressionKind::Index(_)
            | ExpressionKind::Call(_)
            | ExpressionKind::MethodCall(_)
            | ExpressionKind::Constructor(_)
            | ExpressionKind::MemberAccess(_)
            | ExpressionKind::Cast(_)
            | ExpressionKind::For(_)
            | ExpressionKind::If(_)
            | ExpressionKind::Tuple(_) => Err(Some(ResolverError::InvalidArrayLengthExpr { span })),

            ExpressionKind::Error => Err(None),
        }
    }

    fn try_eval_array_length_ident(
        &mut self,
        id: DefinitionId,
        span: Span,
    ) -> Result<u128, Option<ResolverError>> {
        if id == DefinitionId::dummy_id() {
            return Err(None); // error already reported
        }

        let definition = self.interner.definition(id);
        match definition.rhs {
            Some(rhs) if definition.is_global || !definition.mutable => {
                self.try_eval_array_length_id(rhs, span)
            }
            _ => Err(Some(ResolverError::InvalidArrayLengthExpr { span })),
        }
    }

    fn try_eval_array_length_id(
        &self,
        rhs: ExprId,
        span: Span,
    ) -> Result<u128, Option<ResolverError>> {
        match self.interner.expression(&rhs) {
            HirExpression::Literal(HirLiteral::Integer(int)) => {
                int.try_into_u128().ok_or(Some(ResolverError::IntegerTooLarge { span }))
            }
            _other => Err(Some(ResolverError::InvalidArrayLengthExpr { span })),
        }
    }
}

// XXX: These tests repeat a lot of code
// what we should do is have test cases which are passed to a test harness
// A test harness will allow for more expressive and readable tests
#[cfg(test)]
mod test {

    use std::collections::HashMap;

    use fm::FileId;

    use crate::{hir::resolution::errors::ResolverError, Ident};

    use crate::graph::CrateId;
    use crate::hir_def::function::HirFunction;
    use crate::node_interner::{FuncId, NodeInterner};
    use crate::{
        hir::def_map::{CrateDefMap, LocalModuleId, ModuleDefId},
        parse_program, Path,
    };

    use super::{PathResolver, Resolver};

    // func_namespace is used to emulate the fact that functions can be imported
    // and functions can be forward declared
    fn resolve_src_code(src: &str, func_namespace: Vec<&str>) -> Vec<ResolverError> {
        let (program, errors) = parse_program(src);
        assert!(errors.is_empty());

        let mut interner = NodeInterner::default();

        let mut func_ids = Vec::new();
        for _ in 0..func_namespace.len() {
            func_ids.push(interner.push_fn(HirFunction::empty()));
        }

        let mut path_resolver = TestPathResolver(HashMap::new());
        for (name, id) in func_namespace.into_iter().zip(func_ids) {
            path_resolver.insert_func(name.to_owned(), id);
        }

        let def_maps: HashMap<CrateId, CrateDefMap> = HashMap::new();
        let file = FileId::default();

        let mut errors = Vec::new();
        for func in program.functions {
            let resolver = Resolver::new(&mut interner, &path_resolver, &def_maps, file);
            let (_, _, err) = resolver.resolve_function(func);
            errors.extend(err);
        }

        errors
    }

    #[test]
    fn resolve_empty_function() {
        let src = "
            fn main() {

            }
        ";

        let errors = resolve_src_code(src, vec!["main"]);
        assert!(errors.is_empty());
    }
    #[test]
    fn resolve_basic_function() {
        let src = r#"
            fn main(x : Field) {
                let y = x + x;
                constrain y == x;
            }
        "#;

        let errors = resolve_src_code(src, vec!["main"]);
        assert!(errors.is_empty());
    }
    #[test]
    fn resolve_unused_var() {
        let src = r#"
            fn main(x : Field) {
                let y = x + x;
                constrain x == x;
            }
        "#;

        let errors = resolve_src_code(src, vec!["main"]);

        // There should only be one error
        assert!(errors.len() == 1, "Expected 1 error, got: {:?}", errors);

        // It should be regarding the unused variable
        match &errors[0] {
            ResolverError::UnusedVariable { ident } => {
                assert_eq!(&ident.0.contents, "y");
            }
            _ => unreachable!("we should only have an unused var error"),
        }
    }

    #[test]
    fn resolve_unresolved_var() {
        let src = r#"
            fn main(x : Field) {
                let y = x + x;
                constrain y == z;
            }
        "#;

        let errors = resolve_src_code(src, vec!["main"]);

        // There should only be one error
        assert!(errors.len() == 1);

        // It should be regarding the unresolved var `z` (Maybe change to undeclared and special case)
        match &errors[0] {
            ResolverError::VariableNotDeclared { name, span: _ } => assert_eq!(name, "z"),
            _ => unimplemented!("we should only have an unresolved variable"),
        }
    }

    #[test]
    fn unresolved_path() {
        let src = "
            fn main(x : Field) {
                let _z = some::path::to::a::func(x);
            }
        ";

        let mut errors = resolve_src_code(src, vec!["main", "foo"]);
        assert_eq!(errors.len(), 1);
        let err = errors.pop().unwrap();

        path_unresolved_error(err, "some::path::to::a::func");
    }

    #[test]
    fn resolve_literal_expr() {
        let src = r#"
            fn main(x : Field) {
                let y = 5;
                constrain y == x;
            }
        "#;

        let errors = resolve_src_code(src, vec!["main"]);
        assert!(errors.is_empty());
    }

    #[test]
    fn multiple_resolution_errors() {
        let src = r#"
            fn main(x : Field) {
               let y = foo::bar(x);
               let z = y + a;
            }
        "#;

        let errors = resolve_src_code(src, vec!["main"]);
        assert!(errors.len() == 3, "Expected 3 errors, got: {:?}", errors);

        // Errors are:
        // `a` is undeclared
        // `z` is unused
        // `foo::bar` does not exist
        for err in errors {
            match &err {
                ResolverError::UnusedVariable { ident } => {
                    assert_eq!(&ident.0.contents, "z");
                }
                ResolverError::VariableNotDeclared { name, .. } => {
                    assert_eq!(name, "a");
                }
                ResolverError::PathUnresolved { .. } => path_unresolved_error(err, "foo::bar"),
                _ => unimplemented!(),
            };
        }
    }
    #[test]
    fn resolve_prefix_expr() {
        let src = r#"
            fn main(x : Field) {
                let _y = -x;
            }
        "#;

        let errors = resolve_src_code(src, vec!["main"]);
        assert!(errors.is_empty());
    }
    #[test]
    fn resolve_for_expr() {
        let src = r#"
            fn main(x : Field) {
                for i in 1..20 {
                    let _z = x + i;
                };
            }
        "#;

        let errors = resolve_src_code(src, vec!["main"]);
        assert!(errors.is_empty());
    }
    #[test]
    fn resolve_call_expr() {
        let src = r#"
            fn main(x : Field) {
                let _z = foo(x);
            }

            fn foo(x : Field) -> Field {
                x
            }
        "#;

        let errors = resolve_src_code(src, vec!["main", "foo"]);
        assert!(errors.is_empty());
    }

    fn path_unresolved_error(err: ResolverError, expected_unresolved_path: &str) {
        match err {
            ResolverError::PathUnresolved { span: _, name, segment: _ } => {
                assert_eq!(name, expected_unresolved_path)
            }
            _ => unimplemented!("expected an unresolved path"),
        }
    }

    struct TestPathResolver(HashMap<String, ModuleDefId>);

    impl PathResolver for TestPathResolver {
        fn resolve(
            &self,
            _def_maps: &HashMap<CrateId, CrateDefMap>,
            path: Path,
        ) -> Result<Option<ModuleDefId>, Ident> {
            // Not here that foo::bar and hello::foo::bar would fetch the same thing
            let name = path.segments.last().unwrap();
            let mod_def = self.0.get(&name.0.contents).cloned();
            match mod_def {
                None => Err(name.clone()),
                Some(_) => Ok(mod_def),
            }
        }

        fn local_module_id(&self) -> LocalModuleId {
            LocalModuleId::dummy_id()
        }
    }

    impl TestPathResolver {
        pub fn insert_func(&mut self, name: String, func_id: FuncId) {
            self.0.insert(name, func_id.into());
        }
    }
}
