
mod stmt;
mod expr;

// Type checking at the moment is very simple due to what is supported in the grammar.
// If polymorphism is never need, then Wands algorithm should be powerful enough to accommodate 
// all foreseeable types, if it is needed then we would need to switch to Hindley-Milner type or maybe bidirectional

use super::lower::{def_interner::{DefInterner, FuncId}};

// XXX: Note that although it seems like we may not need the TypeChecker struct
// Once errors are added back in fully, we will need it to store multiple errors
pub struct TypeChecker<'a>{
    interner : &'a mut DefInterner
}

impl<'a> TypeChecker<'a> {

    pub fn new(interner : &mut DefInterner) -> TypeChecker {
        TypeChecker{interner}
    }

    /// Type checks a function and assigns the 
    /// appropriate types to expressions in a side table
    pub fn check_func(&mut self, func_id : FuncId){
        
        // First fetch the metadata and add the types for parameters
        // Note that we do not look for the defining Identifier for a parameter,
        // since we know that it is the parameter itself
        let meta = self.interner.function_meta(func_id);
        for param in meta.parameters{
            self.interner.push_ident_type(param.0, param.1);
        }

        // Fetch the HirFunction and iterate all of it's statements
        let hir_func = self.interner.function(func_id);
        for stmt in hir_func.statements() {
            stmt::type_check(&mut self.interner, stmt)
        }
    }
}

// XXX: These tests are all manual currently. 
/// We can either build a test apparatus or pass raw code through the resolver
#[cfg(test)]
mod test {
    use std::collections::HashMap;

    use noirc_errors::{Span, Spanned};

    use crate::{FunctionKind, Parser, Path, Type, hir::{crate_def_map::{CrateDefMap, ModuleDefId}, crate_graph::CrateId, lower::{HirExpression, HirInfixExpression, def_interner::{DefInterner, FuncId}, function::{FuncMeta, HirFunction, Param}, resolver::Resolver, stmt::{HirPrivateStatement, HirStatement}}, resolution::PathResolver}};

    use super::TypeChecker;
    
    #[test]
    fn basic_priv() {
        let mut interner = DefInterner::default();    
    
        // Add a simple Priv Statement into the interner
        // let z = x + y;
        //
        // Push x variable
        let x_id = interner.push_ident(Spanned::from(Span::default(), String::from("x")).into());
        interner.linked_id_to_def(x_id, x_id);
        // Push y variable
        let y_id = interner.push_ident(Spanned::from(Span::default(), String::from("y")).into());
        interner.linked_id_to_def(y_id, y_id);
        // Push z variable
        let z_id = interner.push_ident(Spanned::from(Span::default(), String::from("z")).into());
        interner.linked_id_to_def(z_id, z_id);
        
        // Push x and y as expressions
        let x_expr_id = interner.push_expr(HirExpression::Ident(x_id));
        let y_expr_id = interner.push_expr(HirExpression::Ident(y_id));

        // Create Infix
        let expr = HirInfixExpression {
            lhs : x_expr_id,
            operator : crate::hir::lower::HirBinaryOp::Add,
            rhs : y_expr_id,
        };
        let expr_id = interner.push_expr(HirExpression::Infix(expr));

        // Create priv statement
        let priv_stmt = HirPrivateStatement {
            identifier : z_id,
            r#type : crate::Type::Unspecified,
            expression : expr_id,
        };
        let stmt_id = interner.push_stmt(HirStatement::Private(priv_stmt));

        // Create function to enclose the private statement
        let mut func = HirFunction::empty();
        func.push_stmt(stmt_id);
        let func_id = interner.push_fn(func);
        
        // Add function meta
        let func_meta = FuncMeta {
            name : String::from("test_func"),
            kind : FunctionKind::Normal,
            attributes : None,
            parameters : vec![Param(x_id, Type::Witness), Param(y_id, Type::Witness)], 
            return_type : Type::Unit,
            has_body : true,
        };
        interner.push_fn_meta(func_meta, func_id);

        let mut type_checker = TypeChecker::new(&mut interner);
        type_checker.check_func(func_id);
    }
    #[test]
    fn basic_priv2() {
        let src = r#"

            fn main(x : Witness) {
                priv k = x;
                priv _z = x + k 
            }

        "#;

        type_check_src_code(src, vec![String::from("main")]);
    }
    #[test]
    fn basic_let_stmt() {

        let src = r#"
            fn main(x : Witness) {
                let k = [x,x];
                priv _z = x + k
            }
        "#;

        type_check_src_code(src, vec![String::from("main")]);
    }
    #[test]
    fn basic_index_expr() {

        let src = r#"
            fn main(x : Witness) {
                let k = [x,x];
                priv _z = x + k[0]
            }
        "#;
   
        type_check_src_code(src, vec![String::from("main")]);
    }
    #[test]
    fn basic_call_expr() {

        let src = r#"
            fn main(x : Witness) {
                priv _z = x + foo(x)
            }

            fn foo(x : Witness) -> Witness {
                x
            }
        "#;
   
        type_check_src_code(src, vec![String::from("main"),String::from("foo")]);
    }
    #[test]
    fn basic_for_expr() {

        let src = r#"
            fn main(_x : Witness) {
                let _j = for _i in 0..10 {
                    for _k in 0..100 {

                    }
                }
            }

        "#;
   
        type_check_src_code(src, vec![String::from("main"),String::from("foo"), ]);
    }

    // This is the same Stub that is in the resolver, maybe we can pull this out into a test module and re-use?
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

    // This function assumes that there is only one function and this is the 
    // func id that is returned
    fn type_check_src_code(src : &str, func_namespace : Vec<String>) {
        let mut parser = Parser::from_src(Default::default(), src);
        let program = parser.parse_program().unwrap();
        let mut interner = DefInterner::default();
        
        let mut func_ids = Vec::new();
        for _ in 0..func_namespace.len() {
            func_ids.push(interner.push_fn(HirFunction::empty()));
        }

        let mut path_resolver = TestPathResolver(HashMap::new());
        for (name, id) in func_namespace.into_iter().zip(func_ids.clone()) {
            path_resolver.insert_func(name, id);
        }
        
        let def_maps : HashMap<CrateId, CrateDefMap>= HashMap::new();
        let mut resolver = Resolver::new(&mut interner, &path_resolver,&def_maps);

        let func_meta : Vec<_>= program.functions.into_iter().map(|nf| resolver.resolve_function(nf)).collect();

        for ((hir_func, meta), func_id) in func_meta.into_iter().zip(func_ids.clone()) {
            interner.update_fn(func_id, hir_func);
            interner.push_fn_meta(meta, func_id)
        }   

        // Type check section
        let mut typechecker = TypeChecker::new(&mut interner);
        typechecker.check_func(func_ids.first().cloned().unwrap());

    }
}