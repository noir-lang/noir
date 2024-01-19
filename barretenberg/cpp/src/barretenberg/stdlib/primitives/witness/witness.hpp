#pragma once
#include "barretenberg/ecc/curves/bn254/fr.hpp"

namespace bb::plonk {
namespace stdlib {

// indicates whether a witness index actually contains a constant
static constexpr uint32_t IS_CONSTANT = UINT32_MAX;

template <typename Builder> class witness_t {
  public:
    witness_t() = default;

    witness_t(Builder* parent_context, const bb::fr& in)
    {
        context = parent_context;
        witness = in;
        witness_index = context->add_variable(witness);
    }

    witness_t(Builder* parent_context, const bool in)
    {
        context = parent_context;
        if (in) {
            bb::fr::__copy(bb::fr::one(), witness);
        } else {
            bb::fr::__copy(bb::fr::zero(), witness);
        }
        witness_index = context->add_variable(witness);
    }

    witness_t(Builder* parent_context, IntegralOrEnum auto const in)
    {
        context = parent_context;
        witness = bb::fr{ static_cast<uint64_t>(in), 0, 0, 0 }.to_montgomery_form();
        witness_index = context->add_variable(witness);
    }

    static witness_t create_constant_witness(Builder* parent_context, const bb::fr& in)
    {
        witness_t out(parent_context, in);
        parent_context->assert_equal_constant(out.witness_index, in);
        return out;
    }

    bb::fr witness;
    uint32_t witness_index = IS_CONSTANT;
    Builder* context = nullptr;
};

template <typename Builder> class public_witness_t : public witness_t<Builder> {
  public:
    using witness_t<Builder>::context;
    using witness_t<Builder>::witness;
    using witness_t<Builder>::witness_index;

    public_witness_t() = default;
    public_witness_t(Builder* parent_context, const bb::fr& in)
    {
        context = parent_context;
        bb::fr::__copy(in, witness);
        witness_index = context->add_public_variable(witness);
    }

    public_witness_t(Builder* parent_context, const bool in)
    {
        context = parent_context;
        if (in) {
            bb::fr::__copy(bb::fr::one(), witness);
        } else {
            bb::fr::__copy(bb::fr::zero(), witness);
        }
        witness_index = context->add_public_variable(witness);
    }

    template <typename T> public_witness_t(Builder* parent_context, T const in)
    {
        context = parent_context;
        witness = bb::fr{ static_cast<uint64_t>(in), 0, 0, 0 }.to_montgomery_form();
        witness_index = context->add_public_variable(witness);
    }
};

} // namespace stdlib
} // namespace bb::plonk
