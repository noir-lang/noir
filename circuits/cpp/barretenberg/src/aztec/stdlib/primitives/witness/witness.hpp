#pragma once
#include <ecc/curves/bn254/fr.hpp>

namespace plonk {
namespace stdlib {

/*
inline barretenberg::fr set_bit(const barretenberg::fr& scalar, const uint64_t bit_position)
{
    barretenberg::fr result = scalar;
    uint64_t limb_idx = bit_position / 64ULL;
    uint64_t limb_bit_position = bit_position - (limb_idx * 64ULL);
    result.data[limb_idx] = result.data[limb_idx] + (1ULL << limb_bit_position);
    return result;
}
*/

template <typename ComposerContext> class witness_t {
  public:
    witness_t() = default;

    witness_t(ComposerContext* parent_context, const barretenberg::fr& in)
    {
        context = parent_context;
        barretenberg::fr::__copy(in, witness);
        witness_index = context->add_variable(witness);
    }

    witness_t(ComposerContext* parent_context, const bool in)
    {
        context = parent_context;
        if (in) {
            barretenberg::fr::__copy(barretenberg::fr::one(), witness);
        } else {
            barretenberg::fr::__copy(barretenberg::fr::zero(), witness);
        }
        witness_index = context->add_variable(witness);
    }

    template <typename T> witness_t(ComposerContext* parent_context, T const in)
    {
        context = parent_context;
        witness = barretenberg::fr{ static_cast<uint64_t>(in), 0, 0, 0 }.to_montgomery_form();
        witness_index = context->add_variable(witness);
    }
    barretenberg::fr witness;
    uint32_t witness_index = static_cast<uint32_t>(-1);
    ComposerContext* context = nullptr;
};

template <typename ComposerContext> class public_witness_t : public witness_t<ComposerContext> {
  public:
    using witness_t<ComposerContext>::context;
    using witness_t<ComposerContext>::witness;
    using witness_t<ComposerContext>::witness_index;

    public_witness_t() = default;
    public_witness_t(ComposerContext* parent_context, const barretenberg::fr& in)
    {
        context = parent_context;
        barretenberg::fr::__copy(in, witness);
        witness_index = context->add_public_variable(witness);
    }

    public_witness_t(ComposerContext* parent_context, const bool in)
    {
        context = parent_context;
        if (in) {
            barretenberg::fr::__copy(barretenberg::fr::one(), witness);
        } else {
            barretenberg::fr::__copy(barretenberg::fr::zero(), witness);
        }
        witness_index = context->add_public_variable(witness);
    }

    template <typename T> public_witness_t(ComposerContext* parent_context, T const in)
    {
        context = parent_context;
        witness = barretenberg::fr{ static_cast<uint64_t>(in), 0, 0, 0 }.to_montgomery_form();
        witness_index = context->add_public_variable(witness);
    }
};

} // namespace stdlib
} // namespace plonk