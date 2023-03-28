#pragma once

#include "init.hpp"

#include <aztec3/circuits/abis/private_kernel/private_inputs.hpp>
#include <aztec3/circuits/abis/private_kernel/public_inputs.hpp>

namespace aztec3::circuits::kernel::private_kernel {

using aztec3::circuits::abis::private_kernel::PrivateInputs;
using aztec3::circuits::abis::private_kernel::PublicInputs;

PublicInputs<NT> private_kernel_circuit(Composer& composer, PrivateInputs<NT> const& _private_inputs);
PublicInputs<NT> private_kernel_native(PrivateInputs<NT> const& private_inputs);

} // namespace aztec3::circuits::kernel::private_kernel