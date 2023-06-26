
#[oracle(createL2ToL1Message)]
fn create_l2_to_l1_message_oracle(_message: Field) -> [Field; 1] {}

unconstrained fn create_l2_to_l1_message(message: Field) {
    constrain create_l2_to_l1_message_oracle(message)[0] == 0;
}
