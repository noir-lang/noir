use arbitrary::Unstructured;
use nargo::errors::Location;
use noirc_evaluator::{assert_ssa_snapshot, ssa::ssa_gen};
use noirc_frontend::{
    ast::IntegerBitSize,
    monomorphization::{
        Monomorphizer,
        ast::{
            Call, Definition, Expression, For, FuncId, Function, Ident, IdentId, InlineType,
            LocalId, Program, Type,
        },
    },
    shared::Visibility,
};

use crate::{Config, arb_program, program::FunctionDeclaration, types};

use super::{Context, DisplayAstAsNoir};

#[test]
fn test_make_name() {
    use crate::program::make_name;

    for (i, n) in
        [(0, "a"), (1, "b"), (24, "y"), (25, "z"), (26, "ba"), (27, "bb"), (26 * 2 + 3, "cd")]
    {
        assert_eq!(make_name(i, false), n, "{i} should be {n}");
    }
}

/// Put a body in a `fn main() { <body> }` and compile it into the initial SSA.
fn generate_ssa_from_body(body: Expression) -> ssa_gen::Ssa {
    let func = Function {
        id: FuncId(0),
        name: "main".to_string(),
        parameters: Vec::new(),
        body,
        return_type: Type::Unit,
        return_visibility: Visibility::Private,
        unconstrained: false,
        inline_type: InlineType::Inline,
        func_sig: (Vec::new(), None),
    };

    let sigs = vec![func.func_sig.clone()];

    let program = Program {
        functions: vec![func],
        main_function_signature: sigs[0].clone(),
        function_signatures: sigs,
        return_location: None,
        globals: Default::default(),
        debug_variables: Default::default(),
        debug_functions: Default::default(),
        debug_types: Default::default(),
    };

    ssa_gen::generate_ssa(program).unwrap()
}

/// Test the SSA we get when we use negative range literals with modulo.
#[test]
fn test_modulo_of_negative_literals_in_range() {
    use super::expr::{int_literal, range_modulo};

    let max_size = 5;
    let index_type =
        Type::Integer(noirc_frontend::shared::Signedness::Signed, IntegerBitSize::SixtyFour);

    let start_range =
        range_modulo(int_literal(9u64, true, index_type.clone()), index_type.clone(), max_size);
    let end_range =
        range_modulo(int_literal(1u64, true, index_type.clone()), index_type.clone(), max_size);

    let body = Expression::For(For {
        index_variable: LocalId(0),
        index_name: "idx".to_string(),
        index_type,
        start_range: Box::new(start_range),
        end_range: Box::new(end_range),
        block: Box::new(Expression::Break),
        start_range_location: Location::dummy(),
        end_range_location: Location::dummy(),
    });

    let ssa = generate_ssa_from_body(body);

    // The lower bound is -4 (-9 % 5), represented as a field by subtracting it from u64::MAX.
    assert_ssa_snapshot!(ssa, @r"
    acir(inline) fn main f0 {
      b0():
        jmp b1(i64 -4)
      b1(v0: i64):
        v3 = lt v0, i64 -1
        jmpif v3 then: b2, else: b3
      b2():
        jmp b3()
      b3():
        return
    }
    ");
}

/// Check that the AST we generate for recursive functions is as expected.
#[test]
fn test_recursion_limit_rewrite() {
    let mut ctx = Context::new(Config::default());
    let mut next_ident_id = 0;

    let mut add_func = |id: FuncId, name: &str, unconstrained: bool, calling: &[FuncId]| {
        let calls = calling
            .iter()
            .map(|callee_id| {
                let (callee_name, callee_unconstrained) = if *callee_id == id {
                    (name.to_string(), unconstrained)
                } else {
                    let callee = &ctx.functions[callee_id];
                    (callee.name.clone(), callee.unconstrained)
                };

                let ident_id = IdentId(next_ident_id);
                next_ident_id += 1;

                Expression::Call(Call {
                    func: Box::new(Expression::Ident(Ident {
                        location: None,
                        definition: Definition::Function(*callee_id),
                        mutable: false,
                        name: callee_name,
                        typ: Type::Function(
                            vec![],
                            Box::new(Type::Unit),
                            Box::new(Type::Unit),
                            callee_unconstrained,
                        ),
                        id: ident_id,
                    })),
                    arguments: vec![],
                    return_type: Type::Unit,
                    location: Location::dummy(),
                })
            })
            .collect();

        let func = Function {
            id,
            name: name.to_string(),
            parameters: vec![],
            body: Expression::Block(calls),
            return_type: Type::Unit,
            return_visibility: Visibility::Private,
            unconstrained,
            inline_type: InlineType::InlineAlways,
            func_sig: (vec![], None),
        };

        ctx.function_declarations.insert(
            id,
            FunctionDeclaration {
                name: name.to_string(),
                params: vec![],
                return_type: Type::Unit,
                return_visibility: Visibility::Private,
                inline_type: func.inline_type,
                unconstrained: func.unconstrained,
            },
        );

        ctx.functions.insert(id, func);
    };

    // Create functions:
    // - ACIR main, calling foo
    // - ACIR foo, calling bar
    // - Brillig bar, calling baz and qux
    // - Brillig baz, calling itself
    // - Brillig qux, not calling anything

    let main_id = FuncId(0);
    let foo_id = FuncId(1);
    let bar_id = FuncId(2);
    let baz_id = FuncId(3);
    let qux_id = FuncId(4);

    add_func(qux_id, "qux", true, &[]);
    add_func(baz_id, "baz", true, &[baz_id]);
    add_func(bar_id, "bar", true, &[baz_id, qux_id]);
    add_func(foo_id, "foo", false, &[bar_id]);
    add_func(main_id, "main", false, &[foo_id]);

    // We only generate `Unit` returns, so no randomness is expected,
    // but it would be deterministic anyway.
    let mut u = Unstructured::new(&[0u8; 1]);
    ctx.rewrite_functions(&mut u).unwrap();
    let program = ctx.finalize();

    // Check that:
    // - main passes the limit to foo by ref
    // - foo passes the limit to bar_proxy by value
    // - bar_proxy passes the limit to baz by ref
    // - bar passes the limit to qux, even though it's unused
    // - baz passes the limit to itself by ref

    let code = format!("{}", DisplayAstAsNoir(&program));

    insta::assert_snapshot!(code, @r"
    #[inline_always]
    fn main() -> () {
        let mut ctx_limit: u32 = 25_u32;
        foo((&mut ctx_limit))
    }
    #[inline_always]
    fn foo(ctx_limit: &mut u32) -> () {
        if ((*ctx_limit) == 0_u32) {
            ()
        } else {
            *ctx_limit = ((*ctx_limit) - 1_u32);
            unsafe { bar_proxy((*ctx_limit)) }
        }
    }
    #[inline_always]
    unconstrained fn bar(ctx_limit: &mut u32) -> () {
        if ((*ctx_limit) == 0_u32) {
            ()
        } else {
            *ctx_limit = ((*ctx_limit) - 1_u32);
            baz(ctx_limit);
            qux(ctx_limit)
        }
    }
    #[inline_always]
    unconstrained fn baz(ctx_limit: &mut u32) -> () {
        if ((*ctx_limit) == 0_u32) {
            ()
        } else {
            *ctx_limit = ((*ctx_limit) - 1_u32);
            baz(ctx_limit)
        }
    }
    #[inline_always]
    unconstrained fn qux(_ctx_limit: &mut u32) -> () {
    }
    #[inline_always]
    unconstrained fn bar_proxy(mut ctx_limit: u32) -> () {
        bar((&mut ctx_limit))
    }
    ");
}

/// Test that if we generate a random program, then all of the functions' HIR type signature
/// can be turned into an AST type and back and yield the same result.
///
/// This is not generally true for real Noir programs with e.g. `struct`s in them, but for
/// HIR types that were derived from AST types, the transformation should be idempotent.
#[test]
fn test_to_hir_type_roundtrip() {
    arbtest::arbtest(|u| {
        let config = Config::default();
        let program = arb_program(u, config)?;

        // `program.function_signatures` only contains the `main` function.
        for func in program.functions {
            let hir_types = func
                .func_sig
                .0
                .into_iter()
                .map(|(_, typ, _)| typ)
                .chain(func.func_sig.1.into_iter());

            for hir_type0 in hir_types {
                let mono_type0 =
                    Monomorphizer::convert_type(&hir_type0, Location::dummy()).unwrap();
                let hir_type1 = types::to_hir_type(&mono_type0);
                // Need a second pass to get rid of any inconsistency in the constrainedness of functions.
                let mono_type1 =
                    Monomorphizer::convert_type(&hir_type1, Location::dummy()).unwrap();
                let hir_type2 = types::to_hir_type(&mono_type1);
                assert_eq!(hir_type1, hir_type2);
            }
        }

        Ok(())
    })
    .run();
}
