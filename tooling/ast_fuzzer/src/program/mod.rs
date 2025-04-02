//! Module responsible for generating arbitrary [Program] ASTs.
use std::collections::{BTreeMap, BTreeSet}; // Using BTree for deterministic enumeration, for repeatability.

use expr::gen_expr_literal;
use func::{FunctionContext, FunctionDeclaration};
use strum::IntoEnumIterator;

use arbitrary::{Arbitrary, Unstructured};
use noirc_frontend::{
    ast::IntegerBitSize,
    monomorphization::ast::{
        Expression, FuncId, Function, GlobalId, InlineType, LocalId, Program, Type,
    },
    shared::{Signedness, Visibility},
};

use crate::Config;

mod expr;
mod func;

/// Generate an arbitrary monomorphized AST.
pub fn arb_program(u: &mut Unstructured, config: Config) -> arbitrary::Result<Program> {
    let mut ctx = Context::new(config);
    ctx.gen_globals(u)?;
    ctx.gen_function_decls(u)?;
    ctx.gen_functions(u)?;
    let program = ctx.finalize();
    Ok(program)
}

/// Context to accumulate top level generated item, so we know what we can choose from.
struct Context {
    config: Config,
    /// Global variables, with higher IDs able to refer to previous ones.
    globals: BTreeMap<GlobalId, (Type, Expression)>,
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
            // The AST only uses globals' names' when they are accessed.
            // We can just project a name like `GLOBAL{id}` at the time.
            self.globals.insert(GlobalId(i as u32), g);
        }
        Ok(())
    }

    /// Generate the i-th global variable, which is allowed to use global variables `0..i`.
    fn gen_global(
        &mut self,
        u: &mut Unstructured,
        _i: usize,
    ) -> arbitrary::Result<(Type, Expression)> {
        let typ = self.gen_type(u, self.config.max_depth)?;
        // TODO(7878): Can we use e.g. binary expressions here? Based on a few examples
        // it looked like the compiler already evaluated such expressions into literals.
        let val = gen_expr_literal(u, &typ)?;
        Ok((typ, val))
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
            let name = format!("param{p}");
            let is_mutable = !is_main && bool::arbitrary(u)?;
            let typ = self.gen_type(u, self.config.max_depth)?;
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

        let decl = FunctionDeclaration {
            name: if is_main { "main".to_string() } else { format!("function{i}") },
            params,
            param_visibilities,
            return_type: self.gen_type(u, self.config.max_depth)?,
            return_visibility: if is_main {
                match u.choose_index(5)? {
                    0 | 1 => Visibility::Public,
                    2 | 3 => Visibility::Private,
                    _ => Visibility::ReturnData,
                }
            } else {
                Visibility::Private
            },
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
        for (id, decl) in &self.function_declarations {
            let body = FunctionContext::new(self, *id).gen_body(u)?;
            let function = Function {
                id: *id,
                name: decl.name.clone(),
                parameters: decl.params.clone(),
                body,
                return_type: decl.return_type.clone(),
                unconstrained: decl.unconstrained,
                inline_type: decl.inline_type,
                func_sig: decl.signature(),
            };
            self.functions.insert(*id, function);
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

        let globals = self.globals.into_iter().map(|(id, (_typ, val))| (id, val)).collect();

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
    fn gen_type(&mut self, u: &mut Unstructured, max_depth: usize) -> arbitrary::Result<Type> {
        // See if we can reuse an existing type without going over the maximum depth.
        if u.ratio(5, 10)? {
            let existing_types =
                self.types.iter().filter(|typ| type_depth(typ) <= max_depth).collect::<Vec<_>>();

            if !existing_types.is_empty() {
                return u.choose(&existing_types).map(|typ| (*typ).clone());
            }
        }

        // Once we hit the maximum depth, stop generating composite types.
        let max_index = if max_depth == 0 { 4 } else { 8 };

        let typ = match u.choose_index(max_index)? {
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
                    .map(|_| self.gen_type(u, max_depth - 1))
                    .collect::<Result<Vec<_>, _>>()?;
                Type::Tuple(types)
            }
            6 | 7 => {
                let size = u.int_in_range(0..=self.config.max_array_size)?;
                let typ = self.gen_type(u, max_depth - 1)?;
                Type::Array(size as u32, Box::new(typ))
            }
            _ => unreachable!("unexpected arbitrary type index"),
        };

        self.types.insert(typ.clone());

        Ok(typ)
    }
}

/// Calculate the depth of a type.
///
/// Leaf types have a depth of 0.
fn type_depth(typ: &Type) -> usize {
    match typ {
        Type::Field | Type::Bool | Type::String(_) | Type::Unit | Type::Integer(_, _) => 0,
        Type::Array(_, typ) => 1 + type_depth(typ),
        Type::Tuple(types) => 1 + types.iter().map(type_depth).max().unwrap_or_default(),
        _ => unreachable!("unexpected type: {typ}"),
    }
}
