#pragma once

#include "../../../composer/turbo_composer.hpp"
#include "../../field/field.hpp"
#include "../../uint/uint.hpp"
#include "../../uint32/uint32.hpp"
#include "../crypto.hpp"

#include "../../../../curves/grumpkin/grumpkin.hpp"
#include "../../../../misc_crypto/pedersen/pedersen.hpp"

namespace plonk {
namespace stdlib {
namespace pedersen_note {

struct public_note {
    point ciphertext;
};

struct private_note {
    point owner;
    uint32<waffle::TurboComposer> value;
    field_t<waffle::TurboComposer> secret;
};

struct note_triple {
    point base;
    field_t<waffle::TurboComposer> scalar;
};

template <size_t num_bits>
note_triple fixed_base_scalar_mul(const field_t<waffle::TurboComposer>& in, const size_t generator_index);

public_note encrypt_note(const private_note& plaintext);

extern template note_triple fixed_base_scalar_mul<32>(const field_t<waffle::TurboComposer>& in,
                                                      const size_t generator_index);
extern template note_triple fixed_base_scalar_mul<250>(const field_t<waffle::TurboComposer>& in,
                                                       const size_t generator_index);

} // namespace pedersen_note
} // namespace stdlib
} // namespace plonk