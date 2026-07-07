use ski_calculus_lib::{Arena, ARENA_SIZE, Node};

#[test]
unconstrained fn test_logic_and_show() {
    let mut arena: Arena<ARENA_SIZE> = Arena::default();

    // (T & F | (NOT F | F) & T) & T
    let target_formula = arena.push_logic_example();

    // print AST
    arena.assert_show::<ARENA_SIZE, _>(
        target_formula,
        [
            "(", "(", "(", "(", "S", " ", "S", ")", " ", "(", "K", " ", "(", "K", " ", "(", "S",
            " ", "K", ")", ")", ")", ")", " ", "(", "(", "(", "(", "S", " ", "I", ")", " ", "(",
            "K", " ", "K", ")", ")", " ", "(", "(", "(", "(", "S", " ", "S", ")", " ", "(", "K",
            " ", "(", "K", " ", "(", "S", " ", "K", ")", ")", ")", ")", " ", "K", ")", " ", "(",
            "S", " ", "K", ")", ")", ")", " ", "(", "(", "(", "(", "S", " ", "I", ")", " ", "(",
            "K", " ", "K", ")", ")", " ", "(", "(", "(", "S", " ", "(", "(", "S", " ", "I", ")",
            " ", "(", "K", " ", "(", "S", " ", "K", ")", ")", ")", ")", " ", "(", "K", " ", "K",
            ")", ")", " ", "(", "S", " ", "K", ")", ")", ")",
        ],
    );

    // normalize arena
    arena.assert_eval_steps(20, 29);

    // print AST
    arena.assert_show::<ARENA_SIZE, _>(target_formula, ["K"]);

    // ensure result is TRUE
    assert_eq(arena.get(target_formula), Node::K);

    // check final length
    assert_eq(arena.inner.len(), 45);
}
