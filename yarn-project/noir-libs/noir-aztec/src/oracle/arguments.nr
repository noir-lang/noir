#[oracle(packArguments)]
fn pack_arguments_oracle<N>(_args: [Field; N]) -> Field {}

// TODO: explain what this does.
unconstrained fn pack_arguments<N>(args: [Field; N]) -> Field {
    pack_arguments_oracle(args)
}
