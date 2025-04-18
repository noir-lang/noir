use nargo::errors::Location;
use noirc_evaluator::{assert_ssa_snapshot, ssa::ssa_gen};
use noirc_frontend::{
    ast::IntegerBitSize,
    monomorphization::ast::{
        Expression, For, FuncId, Function, InlineType, LocalId, Program, Type,
    },
    shared::Visibility,
};

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
        return_visibility: Visibility::Private,
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

    let start_range = range_modulo(
        int_literal(51675949543456665u64, true, index_type.clone()),
        index_type.clone(),
        max_size,
    );
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

    // The upper bound is -1, represented as a field by subtracting it from 1<<64.
    assert_ssa_snapshot!(ssa, @r"
    acir(inline) fn main f0 {
      b0():
        jmp b1(i64 0)
      b1(v0: i64):
        v3 = lt v0, i64 18446744073709551615
        jmpif v3 then: b2, else: b3
      b2():
        v5 = unchecked_add v0, i64 1
        jmp b3()
      b3():
        return
    }
    ");
}
