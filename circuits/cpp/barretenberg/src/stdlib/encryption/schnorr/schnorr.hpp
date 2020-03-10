#pragma once
#include <stdlib/types/turbo.hpp>
#include <crypto/schnorr/schnorr.hpp>

namespace plonk {
namespace stdlib {
namespace schnorr {

using namespace plonk::stdlib::types::turbo;

struct signature_bits {
    bit_array_ct s;
    bit_array_ct e;
};

point variable_base_mul(const point& pub_key, const bit_array_ct& scalar);

bool verify_signature(const bit_array_ct& message, const point& pub_key, const signature_bits& sig);

signature_bits convert_signature(Composer* context, const crypto::schnorr::signature& sig);
bit_array_ct convert_message(Composer* context, const std::string& message_string);

} // namespace schnorr
} // namespace stdlib
} // namespace plonk
