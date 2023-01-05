#pragma once

#include "init.hpp"

#include <aztec3/circuits/abis/private_kernel/private_inputs.hpp>
// #include <aztec3/circuits/abis/private_kernel/public_inputs.hpp>

namespace aztec3::circuits::kernel::private_kernel {

using aztec3::circuits::abis::private_kernel::PrivateInputs;
// using abis::private_kernel::PublicInputs;

// TODO: decide what to return.
void private_kernel_circuit(Composer& composer, OracleWrapper& oracle, PrivateInputs<NT> const& _private_inputs);

} // namespace aztec3::circuits::kernel::private_kernel