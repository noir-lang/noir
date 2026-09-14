use ski_calculus_lib::{Arena, ARENA_SIZE, Node};

#[test]
fn test_identity() {
    let mut arena: Arena<ARENA_SIZE> = Arena::default();

    // I (I (.. I))
    // 1  2  .. 10
    let target_formula = arena.push_I_n(10);

    // normalize arena
    arena.assert_eval_steps(10, 10);

    // ensure it reduced to a single identity function
    assert_eq(arena.get(target_formula), Node::I);

    // check final length
    assert_eq(arena.inner.len(), 11);
}
