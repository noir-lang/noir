#include "./mimc.hpp"

#include <memory.h>

#include "../../assert.hpp"
#include "../../keccak/keccak.h"

#include "../composer/mimc_composer.hpp"
#include "../composer/standard_composer.hpp"

#include "./field/field.hpp"

namespace plonk {
namespace stdlib {
namespace {
// mimc.cpp contains an implementation of the 'MiMC7' hash algorithm.
// This uses the MiMC block cipher (with a permutation of x^7), and applies
// the Miyaguchi-Preneel compression function to create a 1-way hash function.

// For MiMC, number of rounds = ceil((security parameter) / log2(mimc exponent))
// for a 254 bit security parameter, and x^7, num rounds = 91.

// References:
// https://eprint.iacr.org/2016/492.pdf
// https://eprint.iacr.org/2005/210.pdf

constexpr size_t num_mimc_rounds = 91;

barretenberg::fr mimc_round_constants[num_mimc_rounds];

#pragma GCC diagnostic ignored "-Wunused-variable"
const auto init_var = []() {
    // clang-format off
    uint8_t inputs[32]{
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        static_cast<uint8_t>(atoi("m")),
        static_cast<uint8_t>(atoi("i")),
        static_cast<uint8_t>(atoi("m")),
        static_cast<uint8_t>(atoi("c"))
    };
    // clang-format on
    for (size_t i = 0; i < num_mimc_rounds; ++i) {
        keccak256 keccak256_hash = ethash_keccak256(&inputs[0], 32);
        memcpy((void*)&inputs[0], (void*)&keccak256_hash.word64s[0], 32);
        mimc_round_constants[i] = barretenberg::fr{ keccak256_hash.word64s[0],
                                                    keccak256_hash.word64s[1],
                                                    keccak256_hash.word64s[2],
                                                    keccak256_hash.word64s[3] }
                                      .to_montgomery_form();
    }
    return true;
}();
} // namespace

field_t<waffle::MiMCComposer> mimc_block_cipher(field_t<waffle::MiMCComposer> message,
                                                field_t<waffle::MiMCComposer> key)
{
    // TODO: Hmm, this should really be a std::shared_ptr
    waffle::MiMCComposer* context = message.context;
    ASSERT(context != nullptr);

    if (!(message.additive_constant == barretenberg::fr::zero()) ||
        !(message.multiplicative_constant == barretenberg::fr::one())) {
        message = message.normalize();
    };
    if (!(key.additive_constant == barretenberg::fr::zero()) ||
        !(key.multiplicative_constant == barretenberg::fr::one())) {
        key = key.normalize();
    }

    // for now assume we have a mimc gate at our disposal

    // each mimc round is (x_in + k + c[i])^7
    barretenberg::fr x_in = message.get_value();
    barretenberg::fr x_out;
    barretenberg::fr k = key.get_value();
    uint32_t k_idx = key.witness_index;
    uint32_t x_in_idx = message.witness_index;
    uint32_t x_out_idx;
    ASSERT(k_idx != static_cast<uint32_t>(-1));
    ASSERT(message.witness_index != static_cast<uint32_t>(-1));
    for (size_t i = 0; i < num_mimc_rounds; ++i) {
        barretenberg::fr T0;
        barretenberg::fr x_cubed;
        T0 = x_in + k;
        T0 += mimc_round_constants[i];
        x_cubed = T0.sqr();
        x_cubed *= T0;
        x_out = x_cubed.sqr();
        x_out *= T0;

        uint32_t x_cubed_idx = context->add_variable(x_cubed);
        x_out_idx = context->add_variable(x_out);
        context->create_mimc_gate({ x_in_idx, x_cubed_idx, k_idx, x_out_idx, mimc_round_constants[i] });
        x_in_idx = x_out_idx;
        barretenberg::fr::__copy(x_out, x_in);
    }
    field_t<waffle::MiMCComposer> result(context, x_out);
    result.witness_index = x_out_idx;
    return result;
}

field_t<waffle::StandardComposer> mimc_block_cipher(field_t<waffle::StandardComposer> message,
                                                    field_t<waffle::StandardComposer> key)
{
    ASSERT(message.context == key.context);
    ASSERT(message.context != nullptr);

    field_t<waffle::StandardComposer> x_in = message;
    field_t<waffle::StandardComposer> x_out(message.context);
    for (size_t i = 0; i < num_mimc_rounds; ++i) {
        x_out = x_in + key + field_t<waffle::StandardComposer>(message.context, mimc_round_constants[i]);
        field_t<waffle::StandardComposer> x_squared = x_out * x_out;
        field_t<waffle::StandardComposer> x_pow_four = x_squared * x_squared;
        x_out = x_pow_four * x_squared * x_out;
        x_in = x_out;
    }
    return x_out;
}

template <typename Composer> field_t<Composer> mimc7(std::vector<field_t<Composer>> const& inputs)
{
    if (inputs.size() == 0) {
        field_t<Composer> out(static_cast<uint64_t>(0));
        return out;
    }
    Composer* context = inputs[0].context;

    // begin with a key schedule of 0
    // TODO: should be constant, should be able to handle this with our custom gate :/
    field_t<Composer> key(witness_t<Composer>(context, 0U));
    field_t<Composer> x_in;
    field_t<Composer> x_out;
    for (size_t i = 0; i < inputs.size(); ++i) {
        field_t<Composer> message = inputs[i];
        x_out = mimc_block_cipher(message, key);
        // combine key with the cipher output and the message
        key = key + x_out + message;
    }
    return key;
}

template field_t<waffle::StandardComposer> mimc7(std::vector<field_t<waffle::StandardComposer>> const& inputs);
template field_t<waffle::MiMCComposer> mimc7(std::vector<field_t<waffle::MiMCComposer>> const& inputs);

} // namespace stdlib
} // namespace plonk
