#pragma once
#include "index.hpp"
#include "init.hpp"

namespace {
using NT = aztec3::utils::types::NativeTypes;
using aztec3::circuits::abis::PreviousKernelData;
}  // namespace

namespace aztec3::circuits::kernel::private_kernel::utils {

std::vector<uint8_t> read_buffer_from_file(const std::string&);
NT::Proof get_proof_from_file();
std::shared_ptr<NT::VK> get_verification_key_from_file();
std::shared_ptr<NT::VK> fake_vk();
PreviousKernelData<NT> dummy_previous_kernel(bool real_vk_proof = false);

}  // namespace aztec3::circuits::kernel::private_kernel::utils