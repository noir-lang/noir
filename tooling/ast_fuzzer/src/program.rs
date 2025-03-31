// Using `BTreeMap` for deterministic iteration.
use acir::FieldElement;
use nargo::errors::Location;
use std::collections::BTreeMap;
use strum::IntoEnumIterator;

use arbitrary::{Arbitrary, Unstructured};
use noirc_abi::Abi;
use noirc_frontend::{
    ast::IntegerBitSize,
    hir_def,
    monomorphization::ast::{
        ArrayLiteral, Expression, FuncId, Function, GlobalId, InlineType, Literal, LocalId,
        Parameters, Program, Type,
    },
    shared::{Signedness, Visibility},
    signed_field::SignedField,
};

use crate::{Config, abi::gen_abi};

/// Generate an arbitrary monomorphized AST and its ABI.
pub fn arb_program(u: &mut Unstructured, config: Config) -> arbitrary::Result<(Program, Abi)> {
    let mut ctx = Context::new(config);
    ctx.gen_globals(u)?;
    ctx.gen_function_decls(u)?;
    ctx.gen_functions(u)?;
    let program = ctx.finalize(u)?;
    let abi = gen_abi(u, &program)?;
    Ok((program, abi))
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
        self.function_declarations.get(&id).expect("function should exist")
    }

    /// Generate random global definitions.
    fn gen_globals(&mut self, u: &mut Unstructured) -> arbitrary::Result<()> {
        for i in 0..u.int_in_range(0..=self.config.max_globals)? {
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

    /// Generate random function names and signatures.
    fn gen_function_decls(&mut self, _u: &mut Unstructured) -> arbitrary::Result<()> {
        Ok(())
    }

    /// Generate random function bodies.
    fn gen_functions(&mut self, u: &mut Unstructured) -> arbitrary::Result<()> {
        for (id, decl) in &self.function_declarations {
            let body = FunctionContext::new(self, *id).gen_body(u);
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

    fn finalize(self, u: &mut Unstructured) -> arbitrary::Result<Program> {
        let main_function_signature = self.function_decl(FuncId(0)).signature();
        let return_visibility =
            if bool::arbitrary(u)? { Visibility::Public } else { Visibility::Private };

        let program = Program {
            functions: self.functions.into_values().collect(),
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

        Ok(program)
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
        _ => panic!("unexpected literal type: {typ}"),
    };
    Ok(expr)
}
