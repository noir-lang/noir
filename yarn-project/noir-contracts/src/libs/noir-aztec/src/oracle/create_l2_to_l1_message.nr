
#[oracle(createL2ToL1Message)]
fn create_l2_to_l1_message_oracle(_message: Field) -> Field {}

unconstrained fn create_l2_to_l1_message(message: Field) {
    assert(create_l2_to_l1_message_oracle(message) == 0);
}
