use std::collections::BTreeMap; // Using BTreeMap in case we need to deterministically enumerate items.

use acir::FieldElement;
use nargo::errors::Location;
use strum::IntoEnumIterator;

use arbitrary::{Arbitrary, Unstructured};
use noirc_frontend::{
    ast::IntegerBitSize,
    hir_def::{self, expr::HirIdent, stmt::HirPattern},
    monomorphization::ast::{
        ArrayLiteral, Expression, FuncId, Function, GlobalId, InlineType, Literal, LocalId,
        Parameters, Program, Type,
    },
    node_interner::DefinitionId,
    shared::{Signedness, Visibility},
    signed_field::SignedField,
};

use crate::Config;

/// Generate an arbitrary monomorphized AST and its ABI.
pub fn arb_program(u: &mut Unstructured, config: Config) -> arbitrary::Result<Program> {
    let mut ctx = Context::new(config);
    ctx.gen_globals(u)?;
    ctx.gen_function_decls(u)?;
    ctx.gen_functions(u)?;
    let program = ctx.finalize();
    Ok(program)
}

/// Signature of a functions we can call.
struct FunctionDeclaration {
    name: String,
    params: Parameters,
    param_visibilities: Vec<Visibility>,
    return_type: Type,
    return_visibility: Visibility,
    inline_type: InlineType,
    unconstrained: bool,
}

impl FunctionDeclaration {
    /// Generate a HIR function signature.
    fn signature(&self) -> hir_def::function::FunctionSignature {
        let param_types = self
            .params
            .iter()
            .zip(self.param_visibilities.iter())
            .map(|((_id, mutable, _name, typ), vis)| {
                // The pattern doesn't seem to be used in `ssa::create_program`,
                // apart from its location, so it shouldn't matter what we put into it.
                let mut pat = HirPattern::Identifier(HirIdent {
                    location: Location::dummy(),
                    id: DefinitionId::dummy_id(),
                    impl_kind: hir_def::expr::ImplKind::NotATraitMethod,
                });
                if *mutable {
                    pat = HirPattern::Mutable(Box::new(pat), Location::dummy());
                }

                let typ = to_hir_type(typ);

                (pat, typ, *vis)
            })
            .collect();

        let return_type = (self.return_type != Type::Unit).then(|| to_hir_type(&self.return_type));

        (param_types, return_type)
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
        self.function_declarations.get(&id).expect("function should exist")
    }

    fn main_decl(&self) -> &FunctionDeclaration {
        self.function_declarations.get(&Program::main_id()).expect("main should exist")
    }

    /// Generate random global definitions.
    fn gen_globals(&mut self, u: &mut Unstructured) -> arbitrary::Result<()> {
        let num_globals = u.int_in_range(0..=self.config.max_globals)?;
        for i in 0..num_globals {
            let g = self.gen_global(u, i)?;
            // TODO: We might want to generate a name for it, or just use its ID as name later.
            self.globals.insert(GlobalId(i as u32), g);
        }
        Ok(())
    }

    /// Generate the i-th global variable, which is allowed to use global variables `0..i`.
    fn gen_global(&self, u: &mut Unstructured, _i: usize) -> arbitrary::Result<Expression> {
        let typ = self.gen_type(u, self.config.max_depth)?;
        // TODO: Can we use binary expressions here? Trying it out on a few examples
        // resulted in the compiler already evaluating such expressions into literals.
        Ok(gen_expr_literal(u, &typ)?)
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
        &self,
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
            // TODO: Reuse existing types, so functions can be aligned.
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

    fn finalize(self) -> Program {
        let return_visibility = self.main_decl().return_visibility;
        let functions = self.functions.into_values().collect::<Vec<_>>();
        let function_signatures = functions.iter().map(|f| f.func_sig.clone()).collect::<Vec<_>>();
        let main_function_signature = function_signatures[0].clone();

        Program {
            functions,
            function_signatures,
            main_function_signature,
            return_location: None,
            return_visibility,
            globals: self.globals,
            debug_variables: Default::default(),
            debug_functions: Default::default(),
            debug_types: Default::default(),
        }
    }

    /// Generate a random [Type].
    ///
    /// With a `max_depth` of 0 only leaf types are created.
    fn gen_type(&self, u: &mut Unstructured, max_depth: usize) -> arbitrary::Result<Type> {
        let max_index = if max_depth == 0 { 4 } else { 6 };
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
            4 => {
                let size = u.int_in_range(1..=self.config.max_tuple_size)?;
                let types = (0..=size)
                    .map(|_| self.gen_type(u, max_depth - 1))
                    .collect::<Result<Vec<_>, _>>()?;
                Type::Tuple(types)
            }
            5 => {
                let size = u.int_in_range(0..=self.config.max_array_size)?;
                let typ = self.gen_type(u, max_depth - 1)?;
                Type::Array(size as u32, Box::new(typ))
            }
            _ => unreachable!(),
        };
        Ok(typ)
    }
}

/// Context used during the generation of a function body.
struct FunctionContext<'a> {
    /// Top level context, to access global variables and other functions.
    ctx: &'a Context,
    /// Declaration of this function.
    decl: &'a FunctionDeclaration,
    /// Self ID.
    id: FuncId,
    /// Every variable created in the function will have an increasing ID,
    /// which does not reset when variables go out of scope.
    next_local_id: u32,
    /// Variables accumulated during the generation of the function body,
    /// initially consisting of the function parameters, then extended
    /// by locally defined variables. Block scopes add and remove layers.
    variables: Vec<im::OrdMap<LocalId, Type>>,
}

impl<'a> FunctionContext<'a> {
    fn new(ctx: &'a Context, id: FuncId) -> Self {
        let decl = ctx.function_decl(id);

        let params = decl.params.iter().map(|(id, _, _, typ)| (*id, typ.clone())).collect();
        let next_local_id = decl.params.iter().map(|p| p.0.0).max().unwrap_or_default();

        Self { ctx, decl, id, next_local_id, variables: vec![params] }
    }

    /// Generate the function body.
    fn gen_body(self, u: &mut Unstructured) -> arbitrary::Result<Expression> {
        // TODO: Generate a random AST using the variables, and return the expected type.
        gen_expr_literal(u, &self.decl.return_type)
    }

    /// Local variables currently in scope.
    fn current_scope(&self) -> &im::OrdMap<LocalId, Type> {
        self.variables.last().expect("there is always the params layer")
    }

    /// Add a layer of block variables.
    fn enter_scope(&mut self) {
        // Instead of shallow cloning an immutable map, we could loop through layers when looking up variables.
        self.variables.push(self.current_scope().clone());
    }

    /// Remove the last layer of block variables.
    fn exit_scope(&mut self) {
        self.variables.pop();
        assert!(!self.variables.is_empty(), "never pop the params layer");
    }

    /// Look up a local variable.
    ///
    /// Panics if it doesn't exist.
    fn local_variable(&self, id: &LocalId) -> &Type {
        self.current_scope().get(id).expect("local variable doesn't exist")
    }

    /// Get and increment the next local ID.
    fn next_local_id(&mut self) -> LocalId {
        let id = LocalId(self.next_local_id);
        self.next_local_id += 1;
        id
    }
}

/// Generate a literal expression according to a type.
fn gen_expr_literal(u: &mut Unstructured, typ: &Type) -> arbitrary::Result<Expression> {
    let expr = match typ {
        Type::Unit => Expression::Literal(Literal::Unit),
        Type::Bool => Expression::Literal(Literal::Bool(bool::arbitrary(u)?)),
        Type::Field => {
            let field = SignedField {
                field: FieldElement::from(u128::arbitrary(u)?),
                is_negative: bool::arbitrary(u)?,
            };
            Expression::Literal(Literal::Integer(field, Type::Field, Location::dummy()))
        }
        Type::Integer(signedness, integer_bit_size) => {
            let field = match integer_bit_size {
                IntegerBitSize::One => FieldElement::from(bool::arbitrary(u)?),
                IntegerBitSize::Eight => FieldElement::from(u8::arbitrary(u)? as u32),
                IntegerBitSize::Sixteen => FieldElement::from(u16::arbitrary(u)? as u32),
                IntegerBitSize::ThirtyTwo => FieldElement::from(u32::arbitrary(u)?),
                IntegerBitSize::SixtyFour => FieldElement::from(u64::arbitrary(u)?),
                IntegerBitSize::HundredTwentyEight => FieldElement::from(u128::arbitrary(u)?),
            };

            let field =
                SignedField { field, is_negative: signedness.is_signed() && bool::arbitrary(u)? };

            Expression::Literal(Literal::Integer(
                field,
                Type::Integer(*signedness, *integer_bit_size),
                Location::dummy(),
            ))
        }
        Type::String(len) => {
            let mut s = String::new();
            for _ in 0..*len {
                s.push(char::arbitrary(u)?);
            }
            Expression::Literal(Literal::Str(s))
        }
        Type::Array(len, typ) => {
            let mut arr = ArrayLiteral { contents: Vec::new(), typ: typ.as_ref().clone() };
            for _ in 0..*len {
                arr.contents.push(gen_expr_literal(u, typ)?);
            }
            Expression::Literal(Literal::Array(arr))
        }
        Type::Tuple(items) => {
            let mut values = Vec::new();
            for typ in items {
                values.push(gen_expr_literal(u, typ)?);
            }
            Expression::Tuple(values)
        }
        _ => unreachable!("unexpected literal type: {typ}"),
    };
    Ok(expr)
}

fn to_hir_type(typ: &Type) -> hir_def::types::Type {
    use hir_def::types::{Kind as HirKind, Type as HirType};

    let size_const =
        |size: u32| Box::new(HirType::Constant(FieldElement::from(size), HirKind::Integer));

    match typ {
        Type::Unit => HirType::Unit,
        Type::Bool => HirType::Bool,
        Type::Field => HirType::FieldElement,
        Type::Integer(signedness, integer_bit_size) => {
            HirType::Integer(*signedness, *integer_bit_size)
        }
        Type::String(size) => HirType::String(size_const(*size)),
        Type::Array(size, typ) => HirType::Array(size_const(*size), Box::new(to_hir_type(typ))),
        Type::Tuple(items) => HirType::Tuple(items.iter().map(to_hir_type).collect()),
        Type::FmtString(_, _)
        | Type::Slice(_)
        | Type::MutableReference(_)
        | Type::Function(_, _, _, _) => {
            unreachable!("unexpected type converting to HIR: {}", typ)
        }
    }
}
