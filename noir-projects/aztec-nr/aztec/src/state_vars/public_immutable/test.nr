use crate::{context::PublicContext, state_vars::public_immutable::PublicImmutable};
use crate::test::{helpers::test_environment::TestEnvironment, mocks::mock_struct::MockStruct};
use dep::protocol_types::traits::Serialize;

global storage_slot = 7;

fn setup() -> TestEnvironment {
    TestEnvironment::new()
}

fn in_public(env: TestEnvironment) -> PublicImmutable<MockStruct, &mut PublicContext> {
    PublicImmutable::new(&mut env.public(), storage_slot)
}

#[test]
fn test_uninitialized_by_default() {
    let env = setup();
    let state_var = in_public(env);

    assert_eq(state_var.is_initialized(), false);
}

#[test]
fn test_initialize_uninitialized() {
    let env = setup();
    let state_var = in_public(env);

    let value = MockStruct::new(5, 6);

    state_var.initialize(value);

    assert(state_var.is_initialized());
    assert(state_var.read() == value);
}

#[test(should_fail_with="PublicImmutable already initialized")]
fn test_initialize_already_initialized() {
    let env = setup();
    let state_var = in_public(env);

    let value = MockStruct::new(5, 6);

    state_var.initialize(value);
    state_var.initialize(value);
}

#[test(should_fail_with="PublicImmutable not initialized")]
fn test_read_uninitialized() {
    let env = setup();
    let state_var = in_public(env);

    let _ = state_var.read();
}
