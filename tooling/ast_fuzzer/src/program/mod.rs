//! Module responsible for generating arbitrary [Program] ASTs.
use std::collections::{BTreeMap, BTreeSet}; // Using BTree for deterministic enumeration, for repeatability.

use func::{FunctionContext, FunctionDeclaration};
use strum::IntoEnumIterator;

use arbitrary::{Arbitrary, Unstructured};
use noirc_frontend::{
    ast::IntegerBitSize,
    monomorphization::{
        ast::{Expression, FuncId, Function, GlobalId, InlineType, LocalId, Program, Type},
        printer::{AstPrinter, FunctionPrintOptions},
    },
    shared::{Signedness, Visibility},
};

use crate::Config;

pub mod expr;
pub(crate) mod freq;
mod func;
pub mod rewrite;
mod scope;
mod types;
pub mod visitor;

#[cfg(test)]
mod tests;

/// Generate an arbitrary monomorphized AST.
pub fn arb_program(u: &mut Unstructured, config: Config) -> arbitrary::Result<Program> {
    let mut ctx = Context::new(config);
    ctx.gen_globals(u)?;
    ctx.gen_function_decls(u)?;
    ctx.gen_functions(u)?;
    ctx.rewrite_functions(u)?;
    let program = ctx.finalize();
    Ok(program)
}

/// Generate an arbitrary monomorphized AST to be reversed into a valid comptime
/// Noir, with a single comptime function called from main with literal arguments.
pub fn arb_program_comptime(u: &mut Unstructured, config: Config) -> arbitrary::Result<Program> {
    let mut config = config.clone();
    // Comptime should use Brillig feature set
    config.force_brillig = true;

    let mut ctx = Context::new(config);

    let decl_inner = ctx.gen_function_decl(u, 1)?;
    ctx.set_function_decl(FuncId(1), decl_inner.clone());
    ctx.gen_function(u, FuncId(1))?;

    // Parameterless main declaration wrapping the inner "main"
    // function call
    let decl_main = FunctionDeclaration {
        name: "main".into(),
        params: vec![],
        return_type: decl_inner.return_type.clone(),
        param_visibilities: vec![],
        return_visibility: Visibility::Public,
        inline_type: InlineType::default(),
        unconstrained: false,
    };

    ctx.set_function_decl(FuncId(0), decl_main);
    ctx.gen_function_with_body(u, FuncId(0), |u, fctx| fctx.gen_body_with_lit_call(u, FuncId(1)))?;
    ctx.rewrite_functions(u)?;

    let program = ctx.finalize();
    Ok(program)
}

/// ID of variables in scope.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub(crate) enum VariableId {
    Local(LocalId),
    Global(GlobalId),
}

/// Name of a variable.
type Name = String;

#[derive(Default)]
/// Context to accumulate top level generated item, so we know what we can choose from.
pub(crate) struct Context {
    config: Config,
    /// Global variables.
    globals: BTreeMap<GlobalId, (Name, Type, Expression)>,
    /// Function signatures generated up front, so we can call any of them,
    /// (except `main`), while generating function bodies.
    function_declarations: BTreeMap<FuncId, FunctionDeclaration>,
    /// Randomly generated functions that can access the globals and call
    /// other functions.
    functions: BTreeMap<FuncId, Function>,
    /// Random types generated for functions.
    types: BTreeSet<Type>,
}

impl Context {
    fn new(config: Config) -> Self {
        Self {
            config,
            globals: Default::default(),
            function_declarations: Default::default(),
            functions: Default::default(),
            types: Default::default(),
        }
    }

    /// Get a function declaration.
    fn function_decl(&self, id: FuncId) -> &FunctionDeclaration {
        self.function_declarations.get(&id).expect("function should exist")
    }

    /// Get a function declaration.
    fn set_function_decl(&mut self, id: FuncId, decl: FunctionDeclaration) {
        self.function_declarations.insert(id, decl);
    }

    /// Get the main function declaration.
    fn main_decl(&self) -> &FunctionDeclaration {
        self.function_declarations.get(&Program::main_id()).expect("main should exist")
    }

    /// Generate random global definitions.
    fn gen_globals(&mut self, u: &mut Unstructured) -> arbitrary::Result<()> {
        let num_globals = u.int_in_range(0..=self.config.max_globals)?;
        for i in 0..num_globals {
            let g = self.gen_global(u, i)?;
            self.globals.insert(GlobalId(i as u32), g);
        }
        Ok(())
    }

    /// Generate the i-th global variable, which is allowed to use global variables `0..i`.
    fn gen_global(
        &mut self,
        u: &mut Unstructured,
        i: usize,
    ) -> arbitrary::Result<(Name, Type, Expression)> {
        let typ =
            self.gen_type(u, self.config.max_depth, true, false, self.config.comptime_friendly)?;
        // By the time we get to the monomorphized AST the compiler will have already turned
        // complex global expressions into literals.
        let val = expr::gen_literal(u, &typ)?;
        let name = make_name(i, true);
        Ok((name, typ, val))
    }

    /// Generate random function names and signatures.
    fn gen_function_decls(&mut self, u: &mut Unstructured) -> arbitrary::Result<()> {
        let num_non_main_fns =
            u.int_in_range(self.config.min_functions..=self.config.max_functions)?;

        for i in 0..(1 + num_non_main_fns) {
            let d = self.gen_function_decl(u, i)?;
            self.function_declarations.insert(FuncId(i as u32), d);
        }
        Ok(())
    }

    /// Generate a random function declaration.
    fn gen_function_decl(
        &mut self,
        u: &mut Unstructured,
        i: usize,
    ) -> arbitrary::Result<FunctionDeclaration> {
        let is_main = i == 0;
        let num_params = u.int_in_range(0..=self.config.max_function_args)?;

        let mut params = Vec::new();
        let mut param_visibilities = Vec::new();
        for p in 0..num_params {
            let id = LocalId(p as u32);
            let name = make_name(p, false);
            let is_mutable = !is_main && bool::arbitrary(u)?;
            let typ = self.gen_type(
                u,
                self.config.max_depth,
                false,
                false,
                self.config.comptime_friendly,
            )?;
            params.push((id, is_mutable, name, typ));

            param_visibilities.push(if is_main {
                match u.choose_index(5)? {
                    0 | 1 => Visibility::Public,
                    2 | 3 => Visibility::Private,
                    _ => Visibility::CallData(p as u32),
                }
            } else {
                Visibility::Private
            });
        }

        let return_type =
            self.gen_type(u, self.config.max_depth, false, false, self.config.comptime_friendly)?;
        let return_visibility = if is_main {
            if types::is_unit(&return_type) {
                Visibility::Private
            } else if u.ratio(4, 5)? {
                Visibility::Public
            } else {
                Visibility::ReturnData
            }
        } else {
            Visibility::Private
        };

        let decl = FunctionDeclaration {
            name: if is_main { "main".to_string() } else { format!("func_{i}") },
            params,
            param_visibilities,
            return_type,
            return_visibility,
            inline_type: if is_main {
                InlineType::default()
            } else {
                *u.choose(&[InlineType::Inline, InlineType::InlineAlways])?
            },
            unconstrained: self.config.force_brillig || bool::arbitrary(u)?,
        };

        Ok(decl)
    }

    /// Generate and add main (for testing)
    #[cfg(test)]
    fn gen_main_decl(&mut self, u: &mut Unstructured) {
        let d = self.gen_function_decl(u, 0).unwrap();
        self.function_declarations.insert(FuncId(0u32), d);
    }

    /// Generate random function bodies.
    fn gen_functions(&mut self, u: &mut Unstructured) -> arbitrary::Result<()> {
        let ids = self.function_declarations.keys().cloned().collect::<Vec<_>>();
        for id in ids {
            self.gen_function(u, id)?;
        }
        Ok(())
    }

    /// Generate random function body.
    fn gen_function(&mut self, u: &mut Unstructured, id: FuncId) -> arbitrary::Result<()> {
        println!("gen_function");
        self.gen_function_with_body(u, id, |u, fctx| fctx.gen_body(u))
    }

    /// Generate function with a specified body generator.
    fn gen_function_with_body(
        &mut self,
        u: &mut Unstructured,
        id: FuncId,
        f: impl Fn(&mut Unstructured, FunctionContext) -> arbitrary::Result<Expression>,
    ) -> arbitrary::Result<()> {
        let fctx = FunctionContext::new(self, id);
        let body = f(u, fctx)?;
        let decl = self.function_decl(id);
        let func = Function {
            id,
            name: decl.name.clone(),
            parameters: decl.params.clone(),
            body,
            return_type: decl.return_type.clone(),
            unconstrained: decl.unconstrained,
            inline_type: decl.inline_type,
            func_sig: decl.signature(),
        };
        self.functions.insert(id, func);
        Ok(())
    }

    /// As a post-processing step, identify recursive functions and add a call depth parameter to them.
    fn rewrite_functions(&mut self, u: &mut Unstructured) -> arbitrary::Result<()> {
        rewrite::add_recursion_limit(self, u)
    }

    /// Return the generated [Program].
    fn finalize(self) -> Program {
        let return_visibility = self.main_decl().return_visibility;
        let functions = self.functions.into_values().collect::<Vec<_>>();

        // The signatures should only contain entry functions. Currently that's just `main`.
        let function_signatures =
            functions.iter().take(1).map(|f| f.func_sig.clone()).collect::<Vec<_>>();

        let main_function_signature = function_signatures[0].clone();

        let globals = self.globals.into_iter().collect();

        let program = Program {
            functions,
            function_signatures,
            main_function_signature,
            return_location: None,
            return_visibility,
            globals,
            debug_variables: Default::default(),
            debug_functions: Default::default(),
            debug_types: Default::default(),
        };

        // Carry out the "ownership analysis" here, so the returned program is ready to be turned into SSA.
        // If we carry out changes that need that analysis to be performed again, we have to make sure
        // we only run it on functions that haven't been through it before, or it might panic. We could
        // instead delay the execution to the just before execution, but it's more consistent this way:
        // for example `CompareSsa` can print the final version without more changes being done to it
        // while the are being converted to SSA.
        program.handle_ownership()
    }

    /// Generate a random [Type].
    ///
    /// Keeps track of types already created, so that we can reuse types instead of always
    /// creating new ones, to increase the chance of being able to pass variables between
    /// functions.
    ///
    /// With a `max_depth` of 0 only leaf types are created.
    ///
    /// With `is_frontend_friendly` we try to only consider types which are less likely to result
    /// in literals that the frontend does not like when it has to infer their types. For example
    /// without further constraints on the type, the frontend expects integer literals to be `u32`.
    /// It also cannot infer the type of empty array literals, e.g. `let x = [];` would not compile.
    /// When we generate types for e.g. function parameters, where the type is going to be declared
    /// along with the variable name, this is not a concern.
    fn gen_type(
        &mut self,
        u: &mut Unstructured,
        max_depth: usize,
        is_global: bool,
        is_frontend_friendly: bool,
        is_comptime_friendly: bool,
    ) -> arbitrary::Result<Type> {
        // See if we can reuse an existing type without going over the maximum depth.
        if u.ratio(5, 10)? {
            let existing_types = self
                .types
                .iter()
                .filter(|typ| !is_global || types::can_be_global(typ))
                .filter(|typ| types::type_depth(typ) <= max_depth)
                .filter(|typ| !is_frontend_friendly || !self.should_avoid_literals(typ))
                .collect::<Vec<_>>();

            if !existing_types.is_empty() {
                return u.choose(&existing_types).map(|typ| (*typ).clone());
            }
        }

        // Once we hit the maximum depth, stop generating composite types.
        let max_index = if max_depth == 0 { 4 } else { 8 };

        let mut typ: Type;
        loop {
            typ = match u.choose_index(max_index)? {
                // 4 leaf types
                0 => Type::Bool,
                1 => Type::Field,
                2 => {
                    // i1 is deprecated, and i128 does not exist yet
                    let sign = if is_frontend_friendly {
                        Signedness::Unsigned
                    } else {
                        *u.choose(&[Signedness::Signed, Signedness::Unsigned])?
                    };
                    let sizes = IntegerBitSize::iter()
                        .filter(|bs| {
                            // i1 and i128 are rejected by the frontend
                            (!sign.is_signed() || (bs.bit_size() != 1 && bs.bit_size() != 128)) &&
                            // The frontend doesn't like non-u32 literals
                            (!is_frontend_friendly || bs.bit_size() <= 32) &&
                            // Comptime doesn't allow for u1 either
                            (!is_comptime_friendly || bs.bit_size() != 1)
                        })
                        .collect::<Vec<_>>();
                    Type::Integer(sign, u.choose_iter(sizes)?)
                }
                3 => Type::String(u.int_in_range(0..=self.config.max_array_size)? as u32),
                // 2 composite types
                4 | 5 => {
                    // 1-size tuples look strange, so let's make it minimum 2 fields.
                    let size = u.int_in_range(2..=self.config.max_tuple_size)?;
                    let types = (0..size)
                        .map(|_| {
                            self.gen_type(
                                u,
                                max_depth - 1,
                                is_global,
                                is_frontend_friendly,
                                is_comptime_friendly,
                            )
                        })
                        .collect::<Result<Vec<_>, _>>()?;
                    Type::Tuple(types)
                }
                6 | 7 => {
                    let min_size = if is_frontend_friendly { 1 } else { 0 };
                    let size = u.int_in_range(min_size..=self.config.max_array_size)?;
                    let typ = self.gen_type(
                        u,
                        max_depth - 1,
                        is_global,
                        is_frontend_friendly,
                        is_comptime_friendly,
                    )?;
                    Type::Array(size as u32, Box::new(typ))
                }
                _ => unreachable!("unexpected arbitrary type index"),
            };
            if !is_global || types::can_be_global(&typ) {
                break;
            }
        }
        self.types.insert(typ.clone());

        Ok(typ)
    }

    /// Is a type likely to cause type inference problems in the frontend when standing alone.
    fn should_avoid_literals(&self, typ: &Type) -> bool {
        match typ {
            Type::Integer(sign, size) => {
                // The frontend expects u32 literals.
                sign.is_signed() && self.config.avoid_negative_int_literals
                    || size.bit_size() > 32 && self.config.avoid_large_int_literals
            }
            Type::Array(0, _) => {
                // With 0 length arrays we run the risk of ending up with `let x = [];`,
                // or similar expressions returning `[]`, the type fo which the fronted could not infer.
                true
            }
            _ => false,
        }
    }
}

/// Derive a variable name from the ID.
///
/// Start with `a`, `b`, continuing with `aa`, `ab` if we run out of the alphabet.
fn make_name(mut id: usize, is_global: bool) -> String {
    let mut name = Vec::new();
    let start = if is_global { 65 } else { 97 };
    loop {
        let i = id % 26;
        name.push(char::from(start + i as u8));
        id -= i;
        if id == 0 {
            break;
        }
        id /= 26;
    }
    name.reverse();
    let name = name.into_iter().collect::<String>();
    if is_global { format!("G_{}", name) } else { name }
}

/// Wrapper around `Program` that prints the AST as close to being able to
/// copy-paste it as a Noir program as we can get.
pub struct DisplayAstAsNoir<'a>(pub &'a Program);

impl std::fmt::Display for DisplayAstAsNoir<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut printer = AstPrinter::default();
        printer.show_id = false;
        printer.show_clone_and_drop = false;
        printer.print_program(self.0, f)
    }
}

/// Wrapper around `Program` that prints its AST as close to
/// Noir syntax as we can get, making it `comptime`. The AST must
/// be specifically prepared to include a main function consisting
/// of a `comptime` wrapped call to a `comptime` (or `unconstrained`)
/// marked function.
pub struct DisplayAstAsNoirComptime<'a>(pub &'a Program);

impl std::fmt::Display for DisplayAstAsNoirComptime<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut printer = AstPrinter::default();
        printer.show_id = false;
        printer.show_clone_and_drop = false;
        for function in &self.0.functions {
            let mut fpo = FunctionPrintOptions::default();
            if function.id == Program::main_id() {
                fpo.comptime_wrap_body = true;
                fpo.return_visibility = Some(Visibility::Public);
            }
            printer.print_function(function, f, fpo)?;
        }
        Ok(())
    }
}
