#pragma once
#include <ecc/curves/grumpkin/grumpkin.hpp>

namespace rollup {
namespace tx {

struct tx_note {
    grumpkin::g1::affine_element owner;
    uint32_t value;
    barretenberg::fr secret;
};

grumpkin::g1::affine_element encrypt_note(const tx_note& plaintext);

} // namespace pedersen_note
} // namespace stdlib
