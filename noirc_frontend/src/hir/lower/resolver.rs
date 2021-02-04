
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
struct ResolverMeta{
    num_times_used : usize,
    id : IdentId
}
use std::collections::HashMap;

use noirc_errors::Spanned;

use crate::{BlockExpression, Expression, ExpressionKind, FunctionKind, Ident, Literal, NoirFunction, Statement, hir::{crate_def_map::CrateDefMap, crate_graph::CrateId, resolution::{PathResolver}}};

use crate::hir::{scope::{Scope as GenericScope, ScopeTree as GenericScopeTree, ScopeForest as GenericScopeForest}};
use super::{HirArrayLiteral, HirBinaryOp, HirBlockExpression, HirCallExpression, HirCastExpression, HirExpression, HirForExpression, HirIndexExpression, HirInfixExpression, HirLiteral, HirPrefixExpression, HirUnaryOp, function::{FuncMeta, HirFunction, Param}, node_interner::{NodeInterner, ExprId, FuncId, IdentId, StmtId}, stmt::{HirConstStatement, HirConstrainStatement, HirLetStatement, HirPrivateStatement, HirStatement}};

use super::errors::ResolverError;

type Scope = GenericScope<String, ResolverMeta>;
type ScopeTree = GenericScopeTree<String, ResolverMeta>;
type ScopeForest = GenericScopeForest<String, ResolverMeta>;

pub struct Resolver<'a> {
    scopes : ScopeForest,

    path_resolver : &'a dyn PathResolver,
    def_maps : &'a HashMap<CrateId, CrateDefMap>,

    interner : &'a mut NodeInterner,

    errors : Vec<ResolverError>,
}

impl<'a> Resolver<'a> {
    pub fn new(interner : &'a mut NodeInterner, path_resolver : &'a dyn PathResolver, def_maps : &'a HashMap<CrateId, CrateDefMap>) -> Resolver<'a> {
        Self {
            path_resolver,
            def_maps,
            scopes : ScopeForest::new(),
            interner,
            errors: Vec::new(),
        }
    }

    fn push_err(&mut self, err : ResolverError) {
        self.errors.push(err.into())
    }

    /// Resolving a function involves interning the metadata
    /// interning any statements inside of the function
    /// and interning the function itself
    /// We resolve and lower the function at the same time
    /// Since lowering would require scope data, unless we add an extra resolution field to the AST
    pub fn resolve_function(mut self, func : NoirFunction) -> Result<(HirFunction, FuncMeta), Vec<ResolverError>> {
        self.scopes.start_function();
        
        let (hir_func, func_meta) = self.intern_function(func);
        let func_scope_tree = self.scopes.end_function();

        self.check_for_unused_variables_in_scope_tree(func_scope_tree);
        
        if self.errors.is_empty() {
            return Ok((hir_func, func_meta))
        } else {
            return Err(self.errors)
        }       
    }
    fn resolve_expression(&mut self, expr : Expression) -> ExprId {
        self.intern_expr(expr)
    }

    fn check_for_unused_variables_in_scope_tree(&mut self, scope_decls : ScopeTree) {
        let mut unused_vars = Vec::new();
        for scope in scope_decls.0.into_iter(){
            Resolver::check_for_unused_variables_in_local_scope(scope, &mut unused_vars);
        };
        
        for unused_var in unused_vars.iter() {            
            self.push_err(ResolverError::UnusedVariable{ident_id : *unused_var});
        }
    }
    fn check_for_unused_variables_in_local_scope(decl_map : Scope, unused_vars : &mut Vec<IdentId>) {
        let unused_variables = decl_map.predicate(|kv :&(&String, &ResolverMeta)| -> bool {
            
            let variable_name = kv.0;
            let metadata = kv.1;
            
            let has_underscore_prefix = variable_name.starts_with("_"); // XXX: This is used for development mode, and will be removed

            if metadata.num_times_used == 0 && !has_underscore_prefix {
                return true
            }
            false
        });
        unused_vars.extend(unused_variables.into_iter().map(|(_, meta)|meta.id));

    }

}

impl<'a> Resolver<'a> {
    fn add_variable_decl(&mut self, name : Ident) -> IdentId {

        let id = self.interner.push_ident(name.clone());
        // Variable was defined here, so it's definition links to itself
        self.interner.linked_ident_to_def(id, id);

        let scope = self.scopes.get_mut_scope();
        let resolver_meta = ResolverMeta {num_times_used : 0, id};
        let old_value = scope.add_key_value(name.0.contents.clone(), resolver_meta);

        match old_value {
            None => {
                // New value, do nothing
            }, 
            Some(old_value) => {
                self.push_err(ResolverError::DuplicateDefinition{first_ident : old_value.id, second_ident : id});
            }
        }
        id
    }

    // Checks for a variable having been declared before
    // variable declaration and definition cannot be separate in Noir
    // Once the variable has been found, intern and link `name` to this definition
    // return the IdentId of `name`
    // 
    // If a variable is not found, then an error is logged and a dummy id 
    // is returned, for better error reporting UX
    fn find_variable(&mut self, name : &Ident) -> IdentId {
        
        // Give variable an IdentId. This is not a definition 
        let id = self.interner.push_ident(name.clone());

        // Find the definition for this Ident
        let scope_tree = self.scopes.current_scope_tree();
        let variable = scope_tree.find_key(&name.0.contents);
        
        if let Some(variable_found) = variable {
            variable_found.num_times_used = variable_found.num_times_used + 1;
            self.interner.linked_ident_to_def(id, variable_found.id);
            return id
        } 

        let err = ResolverError::VariableNotDeclared{name : name.0.contents.clone(), span : name.0.span()};
        self.push_err(err);

        return IdentId::dummy_id()
    }


    pub fn intern_function(&mut self, func : NoirFunction) -> (HirFunction, FuncMeta) {

        let func_meta = self.extract_meta(&func);
    
        let hir_func = match func.kind {
            FunctionKind::Builtin | FunctionKind::LowLevel => HirFunction::empty(),
            FunctionKind::Normal => {
                let expr_id = self.resolve_block(func.def.body);

                self.interner.push_expr_span(expr_id, func.def.span);

                HirFunction::unsafe_from_expr(expr_id)
            },
        };

        (hir_func, func_meta)
    }

    /// Extract metadata from a NoirFunction
    /// to be used in analysis and intern the function parameters
    fn extract_meta(&mut self, func: &NoirFunction) -> FuncMeta {
        let name = func.name().to_owned();
        let attributes = func.attribute().cloned();

        let mut parameters = Vec::new();
        for (ident, typ) in func.parameters().to_owned() {
            let ident_id = self.add_variable_decl(ident.clone());

            parameters.push(Param(ident_id, typ));
        }

        let return_type = func.return_type();

        let func_meta = FuncMeta {
            name,
            kind : func.kind, 
            attributes,
            parameters,
            return_type,
            has_body : func.def.body.len() > 0,
        };
        func_meta
    }

    pub fn intern_stmt(&mut self, stmt : Statement) -> StmtId {
        match stmt {
            Statement::Let(let_stmt) => {
                let id = self.add_variable_decl(let_stmt.identifier);

                let let_stmt = HirLetStatement {
                    identifier: id,
                    r#type: let_stmt.r#type,
                    expression: self.intern_expr(let_stmt.expression),
                };
        
                self.interner.push_stmt(HirStatement::Let(let_stmt))
            },
            Statement::Const(const_stmt) => {
                let id = self.add_variable_decl(const_stmt.identifier);

                let const_stmt = HirConstStatement {
                    identifier: id,
                    r#type: const_stmt.r#type,
                    expression: self.intern_expr(const_stmt.expression),
                };
        
                self.interner.push_stmt(HirStatement::Const(const_stmt))
            },
            Statement::Constrain(constrain_stmt) => {
                let lhs = self.resolve_expression(constrain_stmt.0.lhs);
                let operator : HirBinaryOp = constrain_stmt.0.operator.into();
                let rhs = self.resolve_expression(constrain_stmt.0.rhs);

                let stmt = HirConstrainStatement(HirInfixExpression {lhs, rhs, operator});

                self.interner.push_stmt(HirStatement::Constrain(stmt))
            },
            Statement::Public(_) => todo!(),
            Statement::Private(priv_stmt) => {
                
                let identifier = self.add_variable_decl(priv_stmt.identifier);
                let expression = self.resolve_expression(priv_stmt.expression);
                let stmt = HirPrivateStatement {
                    identifier, expression, r#type : priv_stmt.r#type
                };
                self.interner.push_stmt(HirStatement::Private(stmt))
            },
            Statement::Expression(expr) => {
                let stmt = HirStatement::Expression(self.resolve_expression(expr));
                self.interner.push_stmt(stmt)
            }
        }
    }

    pub fn intern_expr(&mut self, expr : Expression) -> ExprId {
        let expr_id = match expr.kind {
            ExpressionKind::Ident(string) => {
                let span = expr.span;
                let ident : Ident = Spanned::from(span, string).into();
                let ident_id = self.find_variable(&ident);                
                self.interner.push_expr(HirExpression::Ident(ident_id))
            },
            ExpressionKind::Literal(literal) => {
                let literal = match literal {
                    Literal::Bool(b) => HirLiteral::Bool(b),
                    Literal::Array(arr) => {
                        let mut interned_contents = Vec::new();
                        for content in arr.contents {
                            interned_contents.push(self.resolve_expression(content));
                        } 
                        HirLiteral::Array(HirArrayLiteral {
                            contents : interned_contents,
                            r#type : arr.r#type,
                            length : arr.length
                        })
                    },
                    Literal::Integer(integer) => HirLiteral::Integer(integer),
                    Literal::Str(str) => HirLiteral::Str(str)
                };

                self.interner.push_expr(HirExpression::Literal(literal))
                
            },
            ExpressionKind::Prefix(prefix) =>  {
                let operator : HirUnaryOp = prefix.operator.into();
                let rhs = self.resolve_expression(prefix.rhs);
                let expr = HirPrefixExpression{rhs, operator};
                self.interner.push_expr(HirExpression::Prefix(expr))
            },
            ExpressionKind::Infix(infix) | ExpressionKind::Predicate(infix) => {
                let lhs = self.intern_expr(infix.lhs);
                let rhs = self.intern_expr(infix.rhs);
                let expr = HirInfixExpression {
                    lhs,
                    operator: infix.operator.into(),
                    rhs,
                };
                self.interner.push_expr(HirExpression::Infix(expr))
            },
            ExpressionKind::Call(call_expr) => {
                // Get the span and name of path for error reporting
                let span = call_expr.func_name.span();
                let func_name = call_expr.func_name.as_string();

                let func_id = match self.path_resolver.resolve(self.def_maps, call_expr.func_name) {
                    None => {
                        // Could not resolve this symbol, log the error and return a dummy function id
                        let err = ResolverError::PathUnresolved{span, name : func_name};
                        self.push_err(err);

                        FuncId::dummy_id()
                    },
                    Some(def_id) => {
                        // A symbol was found. Check if it is a function
                        if let Some(func_id) = def_id.as_function() {
                            func_id
                        } else {
                            let err = ResolverError::Expected{expected : "function".to_owned(), got : def_id.as_str().to_owned(), span : span };
                            self.push_err(err);
                            FuncId::dummy_id()
                        }
                    },
                };
  
                let mut arguments = Vec::with_capacity(call_expr.arguments.len());
                for arg in call_expr.arguments {
                    arguments.push(self.resolve_expression(arg));
                }
            
                let expr = HirCallExpression {func_id,arguments};
                self.interner.push_expr(HirExpression::Call(expr))
            },
            ExpressionKind::Cast(cast_expr) => {
                
                let lhs = self.resolve_expression(cast_expr.lhs);
                let expr = HirCastExpression {
                    lhs,
                    r#type : cast_expr.r#type,
                };

                self.interner.push_expr(HirExpression::Cast(expr))
            },
            ExpressionKind::For(for_expr) => {
                
                let start_range = self.resolve_expression(for_expr.start_range);
                let end_range = self.resolve_expression(for_expr.end_range);
                
                self.scopes.start_for_loop();

                let identifier =  self.add_variable_decl(for_expr.identifier);
                
                let block_id = self.resolve_block(for_expr.block);
                let for_scope = self.scopes.end_for_loop();

                self.check_for_unused_variables_in_scope_tree(for_scope.into());

                let expr = HirForExpression {
                    start_range, 
                    end_range,
                    block : block_id,
                    identifier
                };
                self.interner.push_expr(HirExpression::For(expr))
            },
            ExpressionKind::If(_) => todo!("If statements are currently not supported"),
            ExpressionKind::Index(indexed_expr) => {

                let collection_name = self.find_variable(&indexed_expr.collection_name);
                let index = self.resolve_expression(indexed_expr.index);
                let expr = HirIndexExpression {
                    collection_name,
                    index
                };
                self.interner.push_expr(HirExpression::Index(expr))
            },
            ExpressionKind::Path(path) => {
                // If the Path is being used as an Expression, then it is referring to an Identifier
                //
                // This is currently not supported : const x = foo::bar::SOME_CONST + 10;
                let ident_id = match path.as_ident() {
                    None => {
                        self.push_err(ResolverError::PathIsNotIdent{span : path.span()});

                        IdentId::dummy_id()
                    },
                    Some(identifier) => self.find_variable(identifier),
                };

                self.interner.push_expr(HirExpression::Ident(ident_id))
            }
            ExpressionKind::Block(block_expr) => {
                self.resolve_block(block_expr)
            }
        };

        self.interner.push_expr_span(expr_id, expr.span);
        expr_id
    }

    fn resolve_block(&mut self, block_expr : BlockExpression) -> ExprId {
        let stmts : Vec<_> = block_expr.0.into_iter().map(|stmt|self.intern_stmt(stmt)).collect();

        let hir_block = HirBlockExpression(stmts);

        self.interner.push_expr(HirExpression::Block(hir_block))
    }
}

// XXX: These tests repeat a lot of code
// what we should do is have test cases which are passed to a test harness
// A test harness will allow for more expressive and readable tests
#[cfg(test)]
mod test {

    use std::{collections::HashMap};

    use errors::ResolverError;

    use crate::{Parser, Path, hir::{crate_def_map::{CrateDefMap, ModuleDefId}, crate_graph::CrateId, lower::{errors::{self}, function::HirFunction, node_interner::{NodeInterner, FuncId}}}};

    use super::{PathResolver, Resolver};

    // func_namespace is used to emulate the fact that functions can be imported
    // and functions can be forward declared
    fn resolve_src_code(src : &str, func_namespace : Vec<String>) -> (NodeInterner, Vec<ResolverError>){
        let mut parser = Parser::from_src(Default::default(), src);
        let program = parser.parse_program().unwrap();
        let mut interner = NodeInterner::default();
        
        let mut func_ids = Vec::new();
        for _ in 0..func_namespace.len() {
            func_ids.push(interner.push_fn(HirFunction::empty()));
        }

        let mut path_resolver = TestPathResolver(HashMap::new());
        for (name, id) in func_namespace.into_iter().zip(func_ids) {
            path_resolver.insert_func(name, id);
        }
        
        let def_maps : HashMap<CrateId, CrateDefMap>= HashMap::new();
        
        let mut errors = Vec::new();
        for func in program.functions {
            let resolver = Resolver::new(&mut interner, &path_resolver,&def_maps);
            match resolver.resolve_function(func) {
                Ok((_, _)) => {},
                Err(err) => errors.extend(err)
            }
        }

        (interner.clone(), errors)
    }

    #[test]
    fn resolve_empty_function() {
        let src = "
            fn main() {

            }
        ";

        let (_,errors) = resolve_src_code(src, vec![String::from("main")]);
        assert!(errors.is_empty());
    }
    #[test]
    fn resolve_basic_function() {
        let src = r#"
            fn main(x : Witness) {
                let y = x + x;
                constrain y == x
            }
        "#;

        let (_,errors) = resolve_src_code(src, vec![String::from("main")]);
        assert!(errors.is_empty());
    }
    #[test]
    fn resolve_unused_var() {
        let src = r#"
            fn main(x : Witness) {
                let y = x + x;
                constrain x == x
            }
        "#;

        let (interner,mut errors) = resolve_src_code(src, vec![String::from("main")]);
        
        // There should only be one error
        assert!(errors.len() == 1);
        let err = errors.pop().unwrap();
        // It should be regarding the unused variable
        match err {
            errors::ResolverError::UnusedVariable{ident_id} => {
                assert_eq!(interner.ident_name(&ident_id), "y".to_owned());
            },
            _=> unimplemented!("we should only have an unused var error")
        }
    }
    #[test]
    fn resolve_unresolved_var() {
        let src = r#"
            fn main(x : Witness) {
                let y = x + x;
                constrain y == z
            }
        "#;

        let (_,mut errors) = resolve_src_code(src, vec![String::from("main")]);
        
        // There should only be one error
        assert!(errors.len() == 1);
        let err = errors.pop().unwrap();
        
        // It should be regarding the unresolved var `z` (Maybe change to undeclared and special case)
        match err {
            errors::ResolverError::VariableNotDeclared{ name , span} => assert_eq!(name, "z"),
            _=> unimplemented!("we should only have an unresolved variable")
        }
    }

    #[test]
    fn unresolved_path() {
        let src = "
            fn main(x : Witness) {
                let _z = some::path::to::a::func(x);
            }
        ";

        let (_, mut errors) = resolve_src_code(src, vec![String::from("main"),String::from("foo")]);
        assert!(errors.len() == 1);
        let err = errors.pop().unwrap();

        path_unresolved_error(err, "some::path::to::a::func");
    }

    #[test]
    fn resolve_literal_expr() {
        let src = r#"
            fn main(x : Witness) {
                let y = 5;
                constrain y == x
            }
        "#;

        let (_,errors) = resolve_src_code(src, vec![String::from("main")]);
        assert!(errors.is_empty());
    }

    #[test]
    fn multiple_resolution_errors() {
        let src = r#"
            fn main(x : Witness) {
               let y = foo::bar(x);
               let z = y + a;
            }
        "#;

        let (interner,errors) = resolve_src_code(src, vec![String::from("main")]);
        assert!(errors.len() == 3);

        // Errors are:
        // `a` is undeclared
        // `z` is unused
        // `foo::bar` does not exist
        for err in errors {
            match &err {
                ResolverError::UnusedVariable { ident_id } => {
                    let name = interner.ident_name(ident_id);
                    assert_eq!(name, "z");
                }
                ResolverError::VariableNotDeclared { name, .. } => {
                    assert_eq!(name, "a");
                }
                ResolverError::PathUnresolved { .. } => path_unresolved_error(err, "foo::bar"),
                _=> unimplemented!()
            };
        }
    }
    #[test]
    fn resolve_prefix_expr() {

        let src = r#"
            fn main(x : Witness) {
                let _y = -x
            }
        "#;

        let (_,errors) = resolve_src_code(src, vec![String::from("main")]);
        assert!(errors.is_empty());
    }
    #[test]
    fn resolve_for_expr() {
        let src = r#"
            fn main(x : Witness) {
                for i in 1..20 {
                    priv _z = x + i;
                };
            }
        "#;

        let (_,errors) = resolve_src_code(src, vec![String::from("main")]);
        assert!(errors.is_empty());
    }
    #[test]
    fn resolve_call_expr() {
        let src = r#"
            fn main(x : Witness) {
                let _z = foo(x);
            }

            fn foo(x : Witness) -> Witness {
                x
            }
        "#;

        let (_,errors) = resolve_src_code(src, vec![String::from("main"),String::from("foo")]);
        assert!(errors.is_empty());
    }

    fn path_unresolved_error(err: ResolverError, expected_unresolved_path : &str) {
        match err {
            ResolverError::PathUnresolved{span, name} =>{
                assert_eq!(name, expected_unresolved_path)
            },
            _=> unimplemented!("expected an unresolved path")
        }
    }

    struct TestPathResolver(HashMap<String, ModuleDefId>);

    impl PathResolver for TestPathResolver {
        fn resolve(&self, _def_maps : &HashMap<CrateId, CrateDefMap>, path : Path) -> Option<ModuleDefId> {
            // Not here that foo::bar and hello::foo::bar would fetch the same thing
            let name = path.segments.last().unwrap();
            self.0.get(&name.0.contents).cloned()
        }
    } 

    impl TestPathResolver {
        pub fn insert_func(&mut self, name : String, func_id : FuncId) {
            self.0.insert(name, func_id.into());
        }
    }
}