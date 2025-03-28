// Using `BTreeMap` for deterministic iteration.
use std::collections::BTreeMap;

use arbitrary::{Arbitrary, Unstructured};
use noirc_abi::Abi;
use noirc_frontend::{
    hir_def,
    monomorphization::ast::{
        Expression, FuncId, Function, GlobalId, InlineType, LocalId, Parameters, Program, Type,
    },
    shared::Visibility,
};

/// Generate an arbitrary monomorphized AST.
pub fn arb_program(u: &mut Unstructured, config: Config) -> arbitrary::Result<(Program, Abi)> {
    let mut ctx = Context::new(config);
    ctx.gen_globals(u)?;
    ctx.gen_function_decls(u)?;
    ctx.gen_functions(u)?;
    let (program, abi) = ctx.finalize(u)?;
    Ok((program, abi))
}

/// AST generation configuration.
#[derive(Debug, Clone)]
pub struct Config {
    /// Maximum number of global definitions.
    pub max_globals: usize,
    /// Maximum number of functions (other than main) to generate.
    pub max_functions: usize,
    /// Maximum number of arguments a function can have.
    pub max_function_args: usize,
}

/// Signature of a functions we can call.
struct FunctionDeclaration {
    name: String,
    params: Parameters,
    return_type: Type,
    inline_type: InlineType,
    unconstrained: bool,
}

impl FunctionDeclaration {
    fn signature(&self) -> hir_def::function::FunctionSignature {
        todo!()
    }
}

/// Context to accumulate top level generated item, so we know what we can choose from.
struct Context {
    config: Config,
    /// Global variables, with higher IDs able to refer to previous ones.
    globals: BTreeMap<GlobalId, Expression>,
    /// Function signatures generated up front, so we can call any of them,
    /// (except `main`), while generating function bodies.
    function_declarations: BTreeMap<FuncId, FunctionDeclaration>,
    /// Randomly generated functions that can access the globals and call
    /// other functions.
    functions: BTreeMap<FuncId, Function>,
}

impl Context {
    fn new(config: Config) -> Self {
        Self {
            config,
            globals: Default::default(),
            function_declarations: Default::default(),
            functions: Default::default(),
        }
    }

    /// Get a function declaration.
    fn function_decl(&self, id: FuncId) -> &FunctionDeclaration {
        &self.function_declarations.get(&id).expect("function should exist")
    }

    /// Generate random global definitions.
    fn gen_globals(&mut self, _u: &mut Unstructured) -> arbitrary::Result<()> {
        todo!()
    }

    /// Generate random function names and signatures.
    fn gen_function_decls(&mut self, _u: &mut Unstructured) -> arbitrary::Result<()> {
        todo!()
    }

    /// Generate random function bodies.
    fn gen_functions(&mut self, u: &mut Unstructured) -> arbitrary::Result<()> {
        for (id, decl) in &self.function_declarations {
            let body = FunctionContext::new(&self, *id).gen_body(u);
            let function = Function {
                id: *id,
                name: decl.name.clone(),
                parameters: decl.params.clone(),
                body,
                return_type: decl.return_type.clone(),
                unconstrained: decl.unconstrained,
                inline_type: decl.inline_type.clone(),
                func_sig: decl.signature(),
            };
            self.functions.insert(*id, function);
        }
        Ok(())
    }

    fn finalize(self, u: &mut Unstructured) -> arbitrary::Result<(Program, Abi)> {
        let main_function_signature = self.function_decl(FuncId(0)).signature();
        let return_visibility =
            if bool::arbitrary(u)? { Visibility::Public } else { Visibility::Private };

        let program = Program {
            functions: self.functions.into_iter().map(|(_, f)| f).collect(),
            function_signatures: self
                .function_declarations
                .values()
                .map(|d| d.signature())
                .collect(),
            main_function_signature,
            return_location: None,
            return_visibility,
            globals: self.globals,
            debug_variables: Default::default(),
            debug_functions: Default::default(),
            debug_types: Default::default(),
        };
        let abi = todo!();

        Ok((program, abi))
    }
}

/// Context used during the generation of a function body.
struct FunctionContext<'a> {
    /// Top level context, to access global variables and other functions.
    top: &'a Context,
    /// Self ID.
    id: FuncId,
    /// Variables accumulated during the generation of the function body,
    /// initially consisting of the function parameters, then extended
    /// by locally defined variables.
    variables: Vec<(LocalId, Type)>,
}

impl<'a> FunctionContext<'a> {
    fn new(top: &'a Context, id: FuncId) -> Self {
        let decl = top.function_decl(id);

        let variables = decl.params.iter().map(|(id, _, _, typ)| (*id, typ.clone())).collect();

        Self { top, id, variables }
    }

    /// Generate the function body.
    fn gen_body(self, _u: &mut Unstructured) -> Expression {
        todo!()
    }
}
