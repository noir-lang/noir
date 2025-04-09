//! Module responsible for generating arbitrary [Program] ASTs.
use std::collections::{BTreeMap, BTreeSet}; // Using BTree for deterministic enumeration, for repeatability.

use func::{FunctionContext, FunctionDeclaration};
use strum::IntoEnumIterator;

use arbitrary::{Arbitrary, Unstructured};
use noirc_frontend::{
    ast::IntegerBitSize,
    monomorphization::{
        ast::{Expression, FuncId, Function, GlobalId, InlineType, LocalId, Program, Type},
        printer::AstPrinter,
    },
    shared::{Signedness, Visibility},
};

use crate::Config;

mod expr;
pub(crate) mod freq;
mod func;
mod scope;
mod types;

/// Generate an arbitrary monomorphized AST.
pub fn arb_program(u: &mut Unstructured, config: Config) -> arbitrary::Result<Program> {
    let mut ctx = Context::new(config);
    ctx.gen_globals(u)?;
    ctx.gen_function_decls(u)?;
    ctx.gen_functions(u)?;
    let program = ctx.finalize();
    Ok(program)
}

/// ID of variables in scope.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
enum VariableId {
    Local(LocalId),
    Global(GlobalId),
}

/// Name of a variable.
type Name = String;

/// Context to accumulate top level generated item, so we know what we can choose from.
struct Context {
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
        let typ = self.gen_type(u, self.config.max_depth, true)?;
        // By the time we get to the monomorphized AST the compiler will have already turned
        // complex global expressions into literals.
        let val = expr::gen_literal(u, &typ)?;
        let name = make_name(i, true);
        Ok((name, typ, val))
    }

    /// Generate random function names and signatures.
    fn gen_function_decls(&mut self, u: &mut Unstructured) -> arbitrary::Result<()> {
        let num_non_main_fns = u.int_in_range(0..=self.config.max_functions)?;
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
            let typ = self.gen_type(u, self.config.max_depth, false)?;
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

        let return_type = self.gen_type(u, self.config.max_depth, false)?;
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
            unconstrained: bool::arbitrary(u)?,
        };

        Ok(decl)
    }

    /// Generate random function bodies.
    fn gen_functions(&mut self, u: &mut Unstructured) -> arbitrary::Result<()> {
        let ids = self.function_declarations.keys().cloned().collect::<Vec<_>>();
        for id in ids {
            let body = FunctionContext::new(self, id).gen_body(u)?;
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
        }
        Ok(())
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

        Program {
            functions,
            function_signatures,
            main_function_signature,
            return_location: None,
            return_visibility,
            globals,
            debug_variables: Default::default(),
            debug_functions: Default::default(),
            debug_types: Default::default(),
        }
    }

    /// Generate a random [Type].
    ///
    /// Keeps track of types already created, so that we can reuse types instead of always
    /// creating new ones, to increase the chance of being able to pass variables between
    /// functions.
    ///
    /// With a `max_depth` of 0 only leaf types are created.
    fn gen_type(
        &mut self,
        u: &mut Unstructured,
        max_depth: usize,
        is_global: bool,
    ) -> arbitrary::Result<Type> {
        // See if we can reuse an existing type without going over the maximum depth.
        if u.ratio(5, 10)? {
            let existing_types = self
                .types
                .iter()
                .filter(|typ| !is_global || types::can_be_global(typ))
                .filter(|typ| types::type_depth(typ) <= max_depth)
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
                2 => Type::Integer(
                    *u.choose(&[Signedness::Signed, Signedness::Unsigned])?,
                    u.choose_iter(IntegerBitSize::iter())?,
                ),
                3 => Type::String(u.int_in_range(0..=self.config.max_array_size)? as u32),
                // 2 composite types
                4 | 5 => {
                    // 1-size tuples look strange, so let's make it minimum 2 fields.
                    let size = u.int_in_range(2..=self.config.max_tuple_size)?;
                    let types = (0..size)
                        .map(|_| self.gen_type(u, max_depth - 1, is_global))
                        .collect::<Result<Vec<_>, _>>()?;
                    Type::Tuple(types)
                }
                6 | 7 => {
                    let size = u.int_in_range(0..=self.config.max_array_size)?;
                    let typ = self.gen_type(u, max_depth - 1, is_global)?;
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
        printer.print_program(self.0, f)
    }
}

#[cfg(test)]
mod tests {
    use crate::program::make_name;

    #[test]
    fn test_make_name() {
        for (i, n) in
            [(0, "a"), (1, "b"), (24, "y"), (25, "z"), (26, "ba"), (27, "bb"), (26 * 2 + 3, "cd")]
        {
            assert_eq!(make_name(i, false), n, "{i} should be {n}");
        }
    }
}
