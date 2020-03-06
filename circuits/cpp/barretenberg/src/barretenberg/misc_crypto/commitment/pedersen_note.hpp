#pragma once

#include "../../curves/grumpkin/grumpkin.hpp"
#include "../pedersen/pedersen.hpp"

namespace crypto {
namespace pedersen_note {

struct private_note {
    grumpkin::g1::affine_element owner;
    uint32_t value;
    barretenberg::fr secret;
};

grumpkin::g1::affine_element encrypt_note(const private_note& plaintext);
} // namespace pedersen_note
} // namespace stdlib
