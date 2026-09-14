use ski_calculus_lib::{Arena, ARENA_SIZE, Node};

#[test]
fn test_add() {
    let mut arena: Arena<ARENA_SIZE> = Arena::default();

    let target_formula = arena.push_add_example();

    // normalize arena
    arena.assert_eval_steps(10, 13);

    let SUCC = arena.push(Node::Succ);
    let CONST_0 = arena.push(Node::Const(0));
    let app_succ = arena.push(Node::App(target_formula, SUCC));
    let app_succ_0 = arena.push(Node::App(app_succ, CONST_0));

    // target_formula (+1) 0
    let target_formula = app_succ_0;

    // normalize arena
    arena.assert_eval_steps(20, 42);
    assert_eq(arena.get(target_formula), Node::Const(5));

    // check final length
    assert_eq(arena.inner.len(), 78);
}
