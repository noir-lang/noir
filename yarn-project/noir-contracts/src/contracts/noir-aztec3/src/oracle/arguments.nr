#[oracle(packArguments)]
fn pack_arguments_oracle<N>(_args: [Field; N]) -> [Field; 1] {}

unconstrained fn pack_arguments<N>(args: [Field; N]) -> Field {
    pack_arguments_oracle(args)[0]
}
