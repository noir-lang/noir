
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
struct ResolverMeta{
    num_times_used : usize,
    id : IdentId
}
use std::collections::HashMap;

use noirc_errors::Spanned;

use crate::{Expression, ExpressionKind, FunctionKind, Ident, Literal, NoirFunction, Statement, hir::{crate_def_map::{self, CrateDefMap, ModuleDefId, ModuleId}, crate_graph::CrateId, resolution::{PathResolver, import::{ImportDirective, resolve_imports}}}};

use crate::hir::{scope::{Scope as GenericScope, ScopeTree as GenericScopeTree, ScopeForest as GenericScopeForest}};
use super::{HirArrayLiteral, HirBinaryOp, HirCallExpression, HirCastExpression, HirExpression, HirForExpression, HirIndexExpression, HirInfixExpression, HirLiteral, HirPrefixExpression, HirUnaryOp, node_interner::{NodeInterner, ExprId, FuncId, IdentId, StmtId}, function::{FuncMeta, HirFunction, Param}, stmt::{HirBlockStatement, HirConstStatement, HirConstrainStatement, HirLetStatement, HirPrivateStatement, HirStatement}};

type Scope = GenericScope<String, ResolverMeta>;
type ScopeTree = GenericScopeTree<String, ResolverMeta>;
type ScopeForest = GenericScopeForest<String, ResolverMeta>;

pub struct Resolver<'a> {
    scopes : ScopeForest,

    path_resolver : &'a dyn PathResolver,
    def_maps : &'a HashMap<CrateId, CrateDefMap>,

    interner : &'a mut NodeInterner,
}

impl<'a> Resolver<'a> {
    pub fn new(interner : &'a mut NodeInterner, path_resolver : &'a dyn PathResolver, def_maps : &'a HashMap<CrateId, CrateDefMap>) -> Resolver<'a> {
        Self {
            path_resolver,
            def_maps,
            scopes : ScopeForest::new(),
            interner
        }
    }


    /// Resolving a function involves interning the metadata
    /// interning any statements inside of the function
    /// and interning the function itself
    /// We resolve and lower the function at the same time
    /// Since lowering would require scope data, unless we add an extra resolution field to the AST
    pub fn resolve_function(&mut self, func : NoirFunction) -> (HirFunction, FuncMeta) {
        self.scopes.start_function();
        
        let (hir_func, func_meta) = self.intern_function(func);
        let func_scope_tree = self.scopes.end_function();

        Resolver::check_for_unused_variables_in_scope_tree(func_scope_tree);
 

        (hir_func, func_meta)
    }
    fn resolve_expression(&mut self, expr : Expression) -> ExprId {
        self.intern_expr(expr)
    }

    fn check_for_unused_variables_in_scope_tree(scope_decls : ScopeTree) {
        let mut unused_vars = Vec::new();
        for scope in scope_decls.0.into_iter(){
            Resolver::check_for_unused_variables_in_local_scope(scope, &mut unused_vars)
        }
        
        if !unused_vars.is_empty() {
            println!("unused variables : {:?}", unused_vars);
            panic!("XXX: error reporting has been rolled back. unused variables in the program")
        }

    }
    fn check_for_unused_variables_in_local_scope(decl_map : Scope, unused_vars : &mut Vec<String>) {
        let unused_variables = decl_map.predicate(|kv :&(&String, &ResolverMeta)| -> bool {
            
            let variable_name = kv.0;
            let metadata = kv.1;
            
            let has_underscore_prefix = variable_name.starts_with("_"); // XXX: This is used for development mode, and will be removed

            if metadata.num_times_used == 0 && !has_underscore_prefix {
                return true
            }
            false
        });

        unused_vars.extend(unused_variables.into_iter().map(|(name, _)|name).cloned());

    }

}

impl<'a> Resolver<'a> {
    fn add_variable_decl(&mut self, name : Ident) -> IdentId {

        let id = self.interner.push_ident(name.clone());

        let scope = self.scopes.get_mut_scope();
        let resolver_meta = ResolverMeta {num_times_used : 0, id};
        let is_new_entry = scope.add_key_value(name.0.contents.clone(), resolver_meta);

        if !is_new_entry {
            let _first_decl = scope.find(&name.0.contents).unwrap();
            println!("{:?}", &name);
            panic!("duplicate def: span is currently not being stored at the moment, we will collect ids, then collect their spans only when we need to report errors")
        }
        id
    }

    // Checks for a variable having been declared before
    // variable declaration and definition cannot be separate in Noir
    // Once the variable has been found, intern and link `name` to this definition
    // return the IdentId of name
    fn find_variable(&mut self, name : &Ident) -> Option<IdentId> {
        
        // Give variable an IdentId. This is not a definition 
        let id = self.interner.push_ident(name.clone());

        // Find the definition for this Ident
        let scope_tree = self.scopes.current_scope_tree();
        let variable = scope_tree.find_key(&name.0.contents);
        
        if let Some(variable_found) = variable {
            variable_found.num_times_used = variable_found.num_times_used + 1;
            self.interner.linked_id_to_def(id, variable_found.id);
            return Some(id)
        } 
        return None
    }


    pub fn intern_function(&mut self, func : NoirFunction) -> (HirFunction, FuncMeta) {

        let func_meta = self.extract_meta(&func);
    
        let hir_func = match func.kind {
            FunctionKind::Builtin | FunctionKind::LowLevel => {
                HirFunction::empty()
            },
            FunctionKind::Normal => {
                let mut hir_func = HirFunction::empty();
                let body = func.def.body;
                for stmt in body.0 {
                    hir_func.push_stmt(self.intern_stmt(stmt));
                }
                hir_func
            }
    
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
                let operator : HirBinaryOp = constrain_stmt.0.operator.contents.into();
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
            Statement::Block(_) => todo!(),
            Statement::Expression(expr) => {
                let stmt = HirStatement::Expression(self.resolve_expression(expr));
                self.interner.push_stmt(stmt)
            }
        }
    }

    pub fn intern_expr(&mut self, expr : Expression) -> ExprId {
        let kind = expr.kind;
        match kind {
            ExpressionKind::Ident(string) => {
                let span = expr.span;
                let ident : Ident = Spanned::from(span, string).into();
                let ident_id = self.find_variable(&ident).expect(&format!("XXX: error reporting has been rolled back while lowering, cannot find variable: {}", ident.0.contents));                
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
                    operator: infix.operator.contents.into(),
                    rhs,
                };
                self.interner.push_expr(HirExpression::Infix(expr))
            },
            ExpressionKind::Call(call_expr) => {
                let module_def_id = self.path_resolver.resolve(self.def_maps, call_expr.func_name).expect("XXX: error reporting. Could not resolve function name");
                let func_id = match module_def_id {
                    ModuleDefId::FunctionId(func_id) => func_id,
                    _=> panic!("XXX: error reporting has been reverted during lowering. Expected a function")
                };
                
                let mut arguments = Vec::new();
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
                
                let mut stmts = Vec::new(); 
                for stmt in for_expr.block.0 {
                    stmts.push(self.intern_stmt(stmt));
                }
                let block = HirBlockStatement(stmts);
                let block_id = self.interner.push_stmt(HirStatement::Block(block));
                let for_scope = self.scopes.end_for_loop();

                Resolver::check_for_unused_variables_in_scope_tree(for_scope.into());

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

                let collection_name = self.find_variable(&indexed_expr.collection_name).expect("XXX: error reporting has been reverted while lowering. expected an ident for the array ");
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
                let ident = match path.as_ident() {
                    Some(ident) => ident,
                    None => panic!("path : {:?} cannot be used as an identifier", path)
                };

                let ident_id = self.find_variable(&ident).expect(&format!("XXX: error reporting has been rolled back while lowering, cannot find variable: {}", ident.0.contents));                
                self.interner.push_expr(HirExpression::Ident(ident_id))
            }
        }
    }
}


// XXX: These tests repeat a lot of code
// what we should do is have test cases which are passed to a test harness
// At the moment, we are testing that they are resolved and that they are lowered without errors
// We are not testing for equality
#[cfg(test)]
mod test {

    use std::collections::HashMap;

    use crate::{Parser, Path, hir::{crate_def_map::{CrateDefMap, ModuleDefId}, crate_graph::CrateId, lower::{node_interner::{NodeInterner, FuncId}, function::HirFunction}}};

    use super::{PathResolver, Resolver};

    // func_namespace is used to emulate the fact that functions can be imported
    // and functions can be forward declared
    fn resolve_src_code(src : &str, func_namespace : Vec<String>) {
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
        let mut resolver = Resolver::new(&mut interner, &path_resolver,&def_maps);

        for func in program.functions {
            let _ = resolver.resolve_function(func);
        }
    }

    #[test]
    fn resolve_empty_function() {
        let src = "
            fn main() {

            }
        ";

        resolve_src_code(src, vec![String::from("main")]);
    }
    #[test]
    fn resolve_basic_function() {
        let src = r#"
            fn main(x : Witness) {
                let _y = x + x;
                constrain x == x
            }
        "#;

        resolve_src_code(src, vec![String::from("main")]);
    }
    #[test]
    fn resolve_literal_expr() {
        let src = r#"
            fn main() {
                let _y = 5
            }
        "#;

        resolve_src_code(src, vec![String::from("main")]);
    }
    #[test]
    fn resolve_prefix_expr() {

        let src = r#"
            fn main(x : Witness) {
                let _y = -x
            }
        "#;

        resolve_src_code(src, vec![String::from("main")]);
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

        resolve_src_code(src, vec![String::from("main")]);
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

        resolve_src_code(src, vec![String::from("main"),String::from("foo")]);
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