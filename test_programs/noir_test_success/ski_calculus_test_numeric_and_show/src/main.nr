use ski_calculus_lib::{Arena, ARENA_SIZE, Node, PRINT_SIZE};

#[test]
unconstrained fn test_numeric_and_show() {
    let mut arena: Arena<ARENA_SIZE> = Arena::default();

    // 3
    let target_formula = arena.push_numeric_example();

    // print AST
    arena.assert_show::<PRINT_SIZE, _>(
        target_formula,
        [
            "(", "(", "S", " ", "(", "(", "S", " ", "(", "K", " ", "S", ")", ")", " ", "K", ")",
            ")", " ", "(", "(", "S", " ", "(", "(", "S", " ", "(", "K", " ", "S", ")", ")", " ",
            "K", ")", ")", " ", "(", "(", "S", " ", "(", "(", "S", " ", "(", "K", " ", "S", ")",
            ")", " ", "K", ")", ")", " ", "(", "K", " ", "I", ")", ")", ")", ")",
        ],
    );

    // normalize arena
    arena.assert_eval_steps(1, 0);
    assert_eq(arena.get(target_formula), arena.get(target_formula));

    let SUCC = arena.push(Node::Succ);
    let CONST_0 = arena.push(Node::Const(0));
    let app_succ = arena.push(Node::App(target_formula, SUCC));
    let app_succ_0 = arena.push(Node::App(app_succ, CONST_0));

    // target_formula (+1) 0
    let target_formula = app_succ_0;

    // normalize arena
    arena.assert_eval_steps(20, 20);
    assert_eq(arena.get(target_formula), Node::Const(3));

    // check final length
    assert_eq(arena.inner.len(), 33);
}
