//! Module responsible for generating arbitrary [Program] ASTs.
use std::collections::{BTreeMap, BTreeSet}; // Using BTree for deterministic enumeration, for repeatability.

use func::{FunctionContext, FunctionDeclaration, can_call};
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

/// Length of generated random constraint messages.
pub(crate) const CONSTRAIN_MSG_LENGTH: u32 = 3;

pub mod expr;
pub(crate) mod freq;
mod func;
pub mod rewrite;
pub mod scope;
pub mod types;

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

    // Generate the first (main-wrapped) function declaration
    let decl_inner = ctx.gen_function_decl(u, 1, true)?;
    ctx.set_function_decl(FuncId(1), decl_inner.clone());

    // Generate the rest of the declarations
    let num_extra_fns = u.int_in_range(ctx.config.min_functions..=ctx.config.max_functions)?;

    for i in 0..num_extra_fns {
        let d = ctx.gen_function_decl(u, i + 2, false)?;
        ctx.set_function_decl(FuncId((i + 2) as u32), d);
    }

    // Parameterless main declaration wrapping the inner "main"
    // function call
    let decl_main = FunctionDeclaration {
        name: "main".into(),
        params: vec![],
        return_type: decl_inner.return_type.clone(),
        return_visibility: Visibility::Public,
        inline_type: InlineType::default(),
        unconstrained: false,
    };

    ctx.set_function_decl(FuncId(0), decl_main);

    // Generating functions in this way (after the main wrapper has been
    // declared) will disallow them from calling the wrapper but not the
    // main inner function
    ctx.gen_functions(u)?;

    ctx.gen_function_with_body(u, FuncId(0), |u, function_ctx| {
        function_ctx.gen_body_with_lit_call(u, FuncId(1))
    })?;
    ctx.rewrite_functions(u)?;

    let program = ctx.finalize();
    Ok(program)
}

/// Build a program with the single `main` function returning
/// the result of a given expression (used for conversion of the
/// comptime interpreter execution results for comparison)
pub fn program_wrap_expression(expr: Expression) -> Program {
    let mut ctx = Context::new(Config::default());

    let decl_main = FunctionDeclaration {
        name: "main".into(),
        params: vec![],
        return_type: expr.return_type().unwrap().into_owned(),
        return_visibility: Visibility::Public,
        inline_type: InlineType::default(),
        unconstrained: true,
    };

    ctx.set_function_decl(FuncId(0), decl_main);
    ctx.gen_function_with_body(&mut Unstructured::new(&[]), FuncId(0), |_u, _function_ctx| {
        Ok(expr)
    })
    .expect("shouldn't access any randomness");

    ctx.finalize()
}

/// ID of variables in scope.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub(crate) enum VariableId {
    Local(LocalId),
    Global(GlobalId),
}

/// ID of a function we can call, either as a pointer in a local variable,
/// or directly as a global function.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub(crate) enum CallableId {
    Local(LocalId),
    Global(FuncId),
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
        let typ = self.gen_type(
            u,
            self.config.max_depth,
            true,
            false,
            self.config.comptime_friendly,
            true,
        )?;
        // By the time we get to the monomorphized AST the compiler will have already turned
        // complex global expressions into literals.
        let val = expr::gen_literal(u, &typ, &self.config)?;
        let name = make_name(i, true);
        Ok((name, typ, val))
    }

    /// Generate random function names and signatures.
    fn gen_function_decls(&mut self, u: &mut Unstructured) -> arbitrary::Result<()> {
        let num_non_main_fns =
            u.int_in_range(self.config.min_functions..=self.config.max_functions)?;

        for i in 0..(1 + num_non_main_fns) {
            let d = self.gen_function_decl(u, i, i == 0)?;
            self.function_declarations.insert(FuncId(i as u32), d);
        }
        Ok(())
    }

    /// Generate a random function declaration.
    ///
    /// The `is_abi` parameter tells the generator to only use parameters which are ABI compatible.
    fn gen_function_decl(
        &mut self,
        u: &mut Unstructured,
        i: usize,
        is_abi: bool,
    ) -> arbitrary::Result<FunctionDeclaration> {
        let id = FuncId(i as u32);
        let is_main = id == Program::main_id();
        let num_params = u.int_in_range(0..=self.config.max_function_args)?;

        // If `main` is unconstrained, it won't call ACIR, so no point generating ACIR functions.
        let unconstrained = self.config.force_brillig
            || (!is_main
                && self
                    .functions
                    .get(&Program::main_id())
                    .map(|func| func.unconstrained)
                    .unwrap_or_default())
            || bool::arbitrary(u)?;

        // We could return a function as well.
        let return_type = self.gen_type(
            u,
            self.config.max_depth,
            false,
            is_main || is_abi,
            self.config.comptime_friendly,
            true,
        )?;

        // Which existing functions we could receive as parameters.
        let func_param_candidates: Vec<FuncId> = if is_main || self.config.avoid_lambdas {
            // Main cannot receive function parameters from outside.
            vec![]
        } else {
            self.function_declarations
                .iter()
                .filter_map(|(callee_id, callee)| {
                    can_call(
                        id,
                        unconstrained,
                        types::contains_reference(&return_type),
                        *callee_id,
                        callee,
                    )
                    .then_some(*callee_id)
                })
                .collect()
        };

        // Choose parameter types.
        let mut params = Vec::new();
        for p in 0..num_params {
            let id = LocalId(p as u32);
            let name = make_name(p, false);
            let is_mutable = bool::arbitrary(u)?;

            let typ = if func_param_candidates.is_empty() || u.ratio(7, 10)? {
                // Take some kind of data type.
                self.gen_type(
                    u,
                    self.config.max_depth,
                    false,
                    is_main || is_abi,
                    self.config.comptime_friendly,
                    true,
                )?
            } else {
                // Take a function type.
                let callee_id = u.choose_iter(&func_param_candidates)?;
                let callee = &self.function_declarations[callee_id];
                let param_types = callee.params.iter().map(|p| p.3.clone()).collect::<Vec<_>>();
                let typ = Type::Function(
                    param_types,
                    Box::new(callee.return_type.clone()),
                    Box::new(Type::Unit),
                    callee.unconstrained,
                );
                if u.ratio(2, 5)? { types::ref_mut(typ) } else { typ }
            };

            let visibility = if is_main {
                match u.choose_index(5)? {
                    0 | 1 => Visibility::Public,
                    2 | 3 => Visibility::Private,
                    _ => Visibility::CallData(p as u32),
                }
            } else {
                Visibility::Private
            };

            params.push((id, is_mutable, name, typ, visibility));
        }

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
            return_type,
            return_visibility,
            inline_type: if is_main {
                InlineType::default()
            } else {
                *u.choose(&[InlineType::Inline, InlineType::InlineAlways])?
            },
            unconstrained,
        };

        Ok(decl)
    }

    /// Generate and add main (for testing)
    #[cfg(test)]
    fn gen_main_decl(&mut self, u: &mut Unstructured) {
        let d = self.gen_function_decl(u, 0, true).unwrap();
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
        self.gen_function_with_body(u, id, |u, function_ctx| function_ctx.gen_body(u))
    }

    /// Generate function with a specified body generator.
    fn gen_function_with_body(
        &mut self,
        u: &mut Unstructured,
        id: FuncId,
        f: impl FnOnce(&mut Unstructured, FunctionContext) -> arbitrary::Result<Expression>,
    ) -> arbitrary::Result<()> {
        let function_ctx = FunctionContext::new(self, id);
        let body = f(u, function_ctx)?;
        let decl = self.function_decl(id);
        let func = Function {
            id,
            name: decl.name.clone(),
            parameters: decl.params.clone(),
            body,
            return_type: decl.return_type.clone(),
            return_visibility: decl.return_visibility,
            unconstrained: decl.unconstrained,
            inline_type: decl.inline_type,
            func_sig: decl.signature(),
        };
        self.functions.insert(id, func);
        Ok(())
    }

    /// Post-processing steps that change functions.
    fn rewrite_functions(&mut self, u: &mut Unstructured) -> arbitrary::Result<()> {
        rewrite::remove_unreachable_functions(self);
        rewrite::add_recursion_limit(self, u)?;
        Ok(())
    }

    /// Return the generated [Program].
    fn finalize(self) -> Program {
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
    #[allow(clippy::too_many_arguments)]
    fn gen_type(
        &mut self,
        u: &mut Unstructured,
        max_depth: usize,
        is_global: bool,
        is_main: bool,
        is_comptime_friendly: bool,
        is_vector_allowed: bool,
    ) -> arbitrary::Result<Type> {
        // See if we can reuse an existing type without going over the maximum depth.
        if u.ratio(5, 10)? {
            let existing_types = self
                .types
                .iter()
                .filter(|typ| !is_global || types::can_be_global(typ))
                .filter(|typ| !is_main || types::can_be_main(typ))
                .filter(|typ| types::type_depth(typ) <= max_depth)
                .filter(|typ| is_vector_allowed || !types::contains_vector(typ))
                .collect::<Vec<_>>();

            if !existing_types.is_empty() {
                return u.choose(&existing_types).map(|typ| (*typ).clone());
            }
        }

        // Once we hit the maximum depth, stop generating composite types.
        let max_index = if max_depth == 0 { 4 } else { 8 };

        // Generate the inner type for composite types with reduced maximum depth.
        let gen_inner_type = |this: &mut Self, u: &mut Unstructured, is_vector_allowed: bool| {
            this.gen_type(
                u,
                max_depth - 1,
                is_global,
                is_main,
                is_comptime_friendly,
                is_vector_allowed,
            )
        };

        let mut typ: Type;
        loop {
            typ = match u.choose_index(max_index)? {
                // 4 leaf types
                0 => Type::Bool,
                1 => Type::Field,
                2 => {
                    // i1 is deprecated, and i128 does not exist yet
                    let sign = *u.choose(&[Signedness::Signed, Signedness::Unsigned])?;
                    let sizes = IntegerBitSize::iter()
                        .filter(|bs| {
                            // i1 and i128 are rejected by the frontend
                            (!sign.is_signed() || (bs.bit_size() != 1 && bs.bit_size() != 128)) &&
                            // Comptime doesn't allow for u1 either
                            (!is_comptime_friendly || bs.bit_size() != 1)
                        })
                        .collect::<Vec<_>>();
                    Type::Integer(sign, u.choose_iter(sizes)?)
                }
                3 => Type::String(u.int_in_range(0..=self.config.max_array_size)? as u32),
                // 3 composite types
                4 | 5 => {
                    // 1-size tuples look strange, so let's make it minimum 2 fields.
                    let size = u.int_in_range(2..=self.config.max_tuple_size)?;
                    let types = (0..size)
                        .map(|_| gen_inner_type(self, u, is_vector_allowed))
                        .collect::<Result<Vec<_>, _>>()?;
                    Type::Tuple(types)
                }
                6 if is_vector_allowed && !self.config.avoid_vectors => {
                    let typ = gen_inner_type(self, u, false)?;
                    Type::Vector(Box::new(typ))
                }
                6 | 7 => {
                    let min_size = 0;
                    let size = u.int_in_range(min_size..=self.config.max_array_size)?;
                    let typ = gen_inner_type(self, u, false)?;
                    Type::Array(size as u32, Box::new(typ))
                }
                _ => unreachable!("unexpected arbitrary type index"),
            };
            // Looping is kinda dangerous, we could get stuck if we run out of randomness,
            // so we have to make sure the first type on the list is acceptable.
            if is_global && !types::can_be_global(&typ) || is_main && !types::can_be_main(&typ) {
                continue;
            } else {
                break;
            }
        }

        if !is_main && !is_global && u.ratio(1, 5)? {
            // Read-only references require the experimental "ownership" feature.
            typ = types::ref_mut(typ);
        }

        self.types.insert(typ.clone());

        Ok(typ)
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
    let mut name = name.into_iter().collect::<String>();
    if matches!(name.as_str(), "as" | "if" | "in" | "fn" | "for" | "loop") {
        name = format!("{name}_");
    }
    if is_global { format!("G_{name}") } else { name }
}

/// Wrapper around `Program` that prints the AST as close to being able to
/// copy-paste it as a Noir program as we can get.
pub struct DisplayAstAsNoir<'a>(pub &'a Program);

impl std::fmt::Display for DisplayAstAsNoir<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut printer = AstPrinter::default();
        printer.show_id = false;
        printer.show_clone_and_drop = false;
        printer.show_specials_as_std = true;
        printer.show_type_in_let = true;
        // Most of the time it doesn't affect testing, except the comptime tests where
        // we parse back the code. For that we use `DisplayAstAsNoirComptime`.
        // But printing ints with their type makes it much easier to replicate errors in integration tests,
        // otherwise the frontend rejects negative or large numbers in many contexts.
        printer.show_type_of_int_literal = true;
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
        printer.show_specials_as_std = true;
        // Declare the type in `let` so that when we parse snippets we can match the types which
        // the AST had, otherwise a literal which was a `u32` in the AST might be inferred as `Field`.
        printer.show_type_in_let = true;
        // Also annotate literals with their type, so we don't have subtle differences in expressions,
        // for example `for i in (5 / 10) as u32 .. 2` is `0..2` or `1..2` depending on whether 5 and 10
        // were some number in the AST or `Field` when parsed by the test.
        printer.show_type_of_int_literal = true;

        for function in &self.0.functions {
            if function.id == Program::main_id() {
                let mut function = function.clone();
                function.return_visibility = Visibility::Public;
                let fpo = FunctionPrintOptions { comptime_wrap_body: true, ..Default::default() };
                printer.print_function(&function, f, fpo)?;
            } else {
                printer.print_function(function, f, FunctionPrintOptions::default())?;
            }
        }

        Ok(())
    }
}
