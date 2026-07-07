use ski_calculus_lib::{Arena, ARENA_SIZE, Node};

#[test]
fn test_logic() {
    let mut arena: Arena<ARENA_SIZE> = Arena::default();

    // (T & F | (NOT F | F) & T) & T
    let target_formula = arena.push_logic_example();

    // normalize arena
    arena.assert_eval_steps(20, 29);

    // ensure result is TRUE
    assert_eq(arena.get(target_formula), Node::K);

    // check final length
    assert_eq(arena.inner.len(), 45);
}
