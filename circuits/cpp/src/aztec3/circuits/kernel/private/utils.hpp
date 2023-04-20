#pragma once
#include "index.hpp"
#include "init.hpp"

namespace {
using NT = aztec3::utils::types::NativeTypes;
using aztec3::circuits::abis::PreviousKernelData;
} // namespace

namespace aztec3::circuits::kernel::private_kernel::utils {

PreviousKernelData<NT> dummy_previous_kernel(bool real_vk_proof = false);

} // namespace aztec3::circuits::kernel::private_kernel::utils