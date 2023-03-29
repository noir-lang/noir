#pragma once
#include "index.hpp"
#include "init.hpp"

namespace {
using NT = aztec3::utils::types::NativeTypes;
using aztec3::circuits::abis::private_kernel::PreviousKernelData;
using namespace plonk::stdlib::types;
} // namespace

namespace aztec3::circuits::kernel::private_kernel::utils {

// TODO rename dummy
PreviousKernelData<NT> default_previous_kernel();

} // namespace aztec3::circuits::kernel::private_kernel::utils