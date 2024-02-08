#include "barretenberg/stdlib/primitives/field/field_conversion.hpp"

namespace bb::stdlib::field_conversion {

/**
 * @brief Converts a challenge to a fq<Builder>
 * @details We sometimes need challenges that are a bb::fq element, so we need to convert the bb::fr challenge to a
 * bb::fq type. We do this by in a similar fashion to the convert_from_bn254_frs function that converts to a
 * fq<Builder>. In fact, we do call that function that the end, but we first have to split the fr<Builder> into two
 * pieces, one that is the 136 lower bits and one that is the 118 higher bits. Then, we can split these two pieces into
 * their bigfield limbs through convert_from_bn254_frs, which is actually just a bigfield constructor that takes in two
 * two-limb frs.
 *
 * TODO(https://github.com/AztecProtocol/barretenberg/issues/850): audit this function more carefully
 * @tparam Builder
 */
template <typename Builder> fq<Builder> convert_to_grumpkin_fr(Builder& builder, const fr<Builder>& f)
{
    constexpr uint64_t NUM_BITS_IN_TWO_LIMBS = 2 * NUM_LIMB_BITS;                // 136
    constexpr uint64_t UPPER_TWO_LIMB_BITS = TOTAL_BITS - NUM_BITS_IN_TWO_LIMBS; // 118
    constexpr uint256_t shift = (uint256_t(1) << NUM_BITS_IN_TWO_LIMBS);
    // split f into low_bits_in and high_bits_in
    constexpr uint256_t LIMB_MASK = shift - 1; // mask for upper 128 bits
    const uint256_t value = f.get_value();
    const uint256_t low_val = static_cast<uint256_t>(value & LIMB_MASK);
    const uint256_t hi_val = static_cast<uint256_t>(value >> NUM_BITS_IN_TWO_LIMBS);

    fr<Builder> low{ witness_t<Builder>(&builder, low_val) };
    fr<Builder> hi{ witness_t<Builder>(&builder, hi_val) };
    // range constrain low to 136 bits and hi to 118 bits
    builder.create_range_constraint(low.witness_index, NUM_BITS_IN_TWO_LIMBS, "create_range_constraint");
    builder.create_range_constraint(hi.witness_index, UPPER_TWO_LIMB_BITS, "create_range_constraint");

    ASSERT(static_cast<uint256_t>(low_val) + (static_cast<uint256_t>(hi_val) << NUM_BITS_IN_TWO_LIMBS) == value);
    // checks this decomposition low + hi * 2^64 = value with an assert_equal
    auto sum = low + hi * shift;
    builder.assert_equal(f.witness_index, sum.witness_index, "assert_equal");

    std::vector<fr<Builder>> fr_vec{ low, hi };
    return convert_from_bn254_frs<Builder, fq<Builder>>(builder, fr_vec);
}

template fq<UltraCircuitBuilder> convert_to_grumpkin_fr<UltraCircuitBuilder>(UltraCircuitBuilder& builder,
                                                                             const fr<UltraCircuitBuilder>& f);
template fq<GoblinUltraCircuitBuilder> convert_to_grumpkin_fr<GoblinUltraCircuitBuilder>(
    GoblinUltraCircuitBuilder& builder, const fr<GoblinUltraCircuitBuilder>& f);

} // namespace bb::stdlib::field_conversion
