
#include "barretenberg/ecc/fields/field_conversion.hpp"
#include "barretenberg/stdlib/primitives/bigfield/constants.hpp"

namespace bb::field_conversion {

static constexpr uint64_t NUM_LIMB_BITS = stdlib::NUM_LIMB_BITS_IN_FIELD_SIMULATION;
static constexpr uint64_t TOTAL_BITS = 254;

/**
 * @brief Converts 2 bb::fr elements to grumpkin::fr
 * @details First, this function must take in 2 bb::fr elements because the grumpkin::fr field has a larger modulus than
 * the bb::fr field, so we choose to send 1 grumpkin::fr element to 2 bb::fr elements to maintain injectivity.
 * For the implementation, we want to minimize the number of constraints created by the circuit form, which happens to
 * use 68 bit limbs to represent a grumpkin::fr (as a bigfield). Therefore, our mapping will split a grumpkin::fr into a
 * 136 bit chunk for the lower two bigfield limbs and the upper chunk for the upper two limbs. The upper chunk ends up
 * being 254 - 2*68 = 118 bits as a result. This is why we check that the bb::frs must be at most 136 and 118 bits
 * respectively (to ensure no overflow). Then, we converts the two chunks to a grumpkin::fr using uint256_t conversions.
 * @param low_bits_in
 * @param high_bits_in
 * @return grumpkin::fr
 */
grumpkin::fr convert_grumpkin_fr_from_bn254_frs(std::span<const bb::fr> fr_vec)
{
    // Combines the two elements into one uint256_t, and then convert that to a grumpkin::fr
    ASSERT(uint256_t(fr_vec[0]) < (uint256_t(1) << (NUM_LIMB_BITS * 2)));              // lower 136 bits
    ASSERT(uint256_t(fr_vec[1]) < (uint256_t(1) << (TOTAL_BITS - NUM_LIMB_BITS * 2))); // upper 254-136=118 bits
    uint256_t value = uint256_t(fr_vec[0]) + (uint256_t(fr_vec[1]) << (NUM_LIMB_BITS * 2));
    grumpkin::fr result(value);
    return result;
}

/**
 * @brief Converts grumpkin::fr to 2 bb::fr elements
 * @details First, this function must return 2 bb::fr elements because the grumpkin::fr field has a larger modulus than
 * the bb::fr field, so we choose to send 1 grumpkin::fr element to 2 bb::fr elements to maintain injectivity.
 * This function the reverse of convert_from_bn254_frs(std::span<const bb::fr> fr_vec, grumpkin::fr*) by merging the two
 * pairs of limbs back into the 2 bb::fr elements. For the implementation, we want to minimize the number of constraints
 * created by the circuit form, which happens to use 68 bit limbs to represent a grumpkin::fr (as a bigfield).
 * Therefore, our mapping will split a grumpkin::fr into a 136 bit chunk for the lower two bigfield limbs and the upper
 * chunk for the upper two limbs. The upper chunk ends up being 254 - 2*68 = 118 bits as a result. We manipulate the
 * value using bitwise masks and shifts to obtain our two chunks.
 * @param input
 * @return std::array<bb::fr, 2>
 */
std::vector<bb::fr> convert_grumpkin_fr_to_bn254_frs(const grumpkin::fr& val)
{
    // Goal is to slice up the 64 bit limbs of grumpkin::fr/uint256_t to mirror the 68 bit limbs of bigfield
    // We accomplish this by dividing the grumpkin::fr's value into two 68*2=136 bit pieces.
    constexpr uint64_t LOWER_BITS = 2 * NUM_LIMB_BITS;
    constexpr uint256_t LOWER_MASK = (uint256_t(1) << LOWER_BITS) - 1;
    auto value = uint256_t(val);
    ASSERT(value < (uint256_t(1) << TOTAL_BITS));
    std::vector<bb::fr> result(2);
    result[0] = static_cast<uint256_t>(value & LOWER_MASK);
    result[1] = static_cast<uint256_t>(value >> LOWER_BITS);
    ASSERT(static_cast<uint256_t>(result[1]) < (uint256_t(1) << (TOTAL_BITS - LOWER_BITS)));
    return result;
}

grumpkin::fr convert_to_grumpkin_fr(const bb::fr& f)
{
    const uint64_t NUM_BITS_IN_TWO_LIMBS = 2 * NUM_LIMB_BITS; // the number of bits in 2 bigfield limbs which is 136

    constexpr uint256_t LIMB_MASK =
        (uint256_t(1) << NUM_BITS_IN_TWO_LIMBS) - 1; // split bn254_fr into two 136 bit pieces
    const uint256_t value = f;
    const uint256_t low = static_cast<uint256_t>(value & LIMB_MASK);
    const uint256_t hi = static_cast<uint256_t>(value >> NUM_BITS_IN_TWO_LIMBS);
    ASSERT(static_cast<uint256_t>(low) + (static_cast<uint256_t>(hi) << NUM_BITS_IN_TWO_LIMBS) == value);

    std::vector<bb::fr> fr_vec{ low, hi };
    return convert_from_bn254_frs<grumpkin::fr>(fr_vec);
}

} // namespace bb::field_conversion
