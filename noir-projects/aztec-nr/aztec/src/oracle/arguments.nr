#[oracle(packArgumentsArray)]
fn pack_arguments_array_oracle<N>(_args: [Field; N]) -> Field {}

#[oracle(packArguments)]
fn pack_arguments_oracle(_args: [Field]) -> Field {}

/// - Pack arguments (array version) will notify the simulator that these arguments will be used later at
///   some point in the call. 
/// - When the external call is made later, the simulator will know what the values unpack to.
/// - This oracle will not be required in public vm functions, as the vm will keep track of arguments 
///   itself.
unconstrained pub fn pack_arguments_array<N>(args: [Field; N]) -> Field {
    pack_arguments_array_oracle(args)
}

/// - Pack arguments (slice version) will notify the simulator that these arguments will be used later at
///   some point in the call. 
/// - When the external call is made later, the simulator will know what the values unpack to.
/// - This oracle will not be required in public vm functions, as the vm will keep track of arguments 
///   itself.
unconstrained pub fn pack_arguments(args: [Field]) -> Field {
    pack_arguments_oracle(args)
}

