#pragma once

#include "../../common/assert.hpp"
#include "./affine_element.hpp"
#include "./element.hpp"
#include "./wnaf.hpp"
#include "barretenberg/common/constexpr_utils.hpp"
#include "barretenberg/crypto/blake3s/blake3s.hpp"
#include <array>
#include <cinttypes>
#include <cstdint>
#include <cstdio>
#include <cstdlib>
namespace bb {

/**
 * @brief group class. Represents an elliptic curve group element.
 * Group is parametrised by coordinate_field and subgroup_field
 *
 * Note: Currently subgroup checks are NOT IMPLEMENTED
 * Our current Plonk implementation uses G1 points that have a cofactor of 1.
 * All G2 points are precomputed (generator [1]_2 and trusted setup point [x]_2).
 * Explicitly assume precomputed points are valid members of the prime-order subgroup for G2.
 *
 * @tparam coordinate_field
 * @tparam subgroup_field
 * @tparam GroupParams
 */
template <typename _coordinate_field, typename _subgroup_field, typename GroupParams> class group {
  public:
    // hoist coordinate_field, subgroup_field into the public namespace
    using coordinate_field = _coordinate_field;
    using subgroup_field = _subgroup_field;
    using element = group_elements::element<coordinate_field, subgroup_field, GroupParams>;
    using affine_element = group_elements::affine_element<coordinate_field, subgroup_field, GroupParams>;
    using Fq = coordinate_field;
    using Fr = subgroup_field;
    static constexpr bool USE_ENDOMORPHISM = GroupParams::USE_ENDOMORPHISM;
    static constexpr bool has_a = GroupParams::has_a;

    static constexpr element one{ GroupParams::one_x, GroupParams::one_y, coordinate_field::one() };
    static constexpr element point_at_infinity = one.set_infinity();
    static constexpr affine_element affine_one{ GroupParams::one_x, GroupParams::one_y };
    static constexpr affine_element affine_point_at_infinity = affine_one.set_infinity();
    static constexpr coordinate_field curve_a = GroupParams::a;
    static constexpr coordinate_field curve_b = GroupParams::b;

    /**
     * @brief Derives generator points via hash-to-curve
     *
     * ALGORITHM DESCRIPTION:
     *      1. Each generator has an associated "generator index" described by its location in the vector
     *      2. a 64-byte preimage buffer is generated with the following structure:
     *          bytes 0-31: BLAKE3 hash of domain_separator
     *          bytes 32-63: generator index in big-endian form
     *      3. The hash-to-curve algorithm is used to hash the above into a group element:
     *           a. iterate `count` upwards from `0`
     *           b. append `count` to the preimage buffer as a 1-byte integer in big-endian form
     *           c. compute BLAKE3 hash of concat(preimage buffer, 0)
     *           d. compute BLAKE3 hash of concat(preimage buffer, 1)
     *           e. interpret (c, d) as (hi, low) limbs of a 512-bit integer
     *           f. reduce 512-bit integer modulo coordinate_field to produce x-coordinate
     *           g. attempt to derive y-coordinate. If not successful go to step (a) and continue
     *           h. if parity of y-coordinate's least significant bit does not match parity of most significant bit of
     *              (d), invert y-coordinate.
     *           j. return (x, y)
     *
     * NOTE: In step 3b it is sufficient to use 1 byte to store `count`.
     *       Step 3 has a 50% chance of returning, the probability of `count` exceeding 256 is 1 in 2^256
     * NOTE: The domain separator is included to ensure that it is possible to derive independent sets of
     * index-addressable generators.
     * NOTE: we produce 64 bytes of BLAKE3 output when producing x-coordinate field
     * element, to ensure that x-coordinate is uniformly randomly distributed in the field. Using a 256-bit input adds
     * significant bias when reducing modulo a ~256-bit coordinate_field
     * NOTE: We ensure y-parity is linked to preimage
     * hash because there is no canonical deterministic square root algorithm (i.e. if a field element has a square
     * root, there are two of them and `field::sqrt` may return either one)
     * @param num_generators
     * @param domain_separator
     * @return std::vector<affine_element>
     */
    inline static constexpr std::vector<affine_element> derive_generators(
        const std::vector<uint8_t>& domain_separator_bytes,
        const size_t num_generators,
        const size_t starting_index = 0)
    {
        std::vector<affine_element> result;
        const auto domain_hash = blake3::blake3s_constexpr(&domain_separator_bytes[0], domain_separator_bytes.size());
        std::vector<uint8_t> generator_preimage;
        generator_preimage.reserve(64);
        std::copy(domain_hash.begin(), domain_hash.end(), std::back_inserter(generator_preimage));
        for (size_t i = 0; i < 32; ++i) {
            generator_preimage.emplace_back(0);
        }
        for (size_t i = starting_index; i < starting_index + num_generators; ++i) {
            auto generator_index = static_cast<uint32_t>(i);
            uint32_t mask = 0xff;
            generator_preimage[32] = static_cast<uint8_t>(generator_index >> 24);
            generator_preimage[33] = static_cast<uint8_t>((generator_index >> 16) & mask);
            generator_preimage[34] = static_cast<uint8_t>((generator_index >> 8) & mask);
            generator_preimage[35] = static_cast<uint8_t>(generator_index & mask);
            result.push_back(affine_element::hash_to_curve(generator_preimage));
        }
        return result;
    }

    inline static constexpr std::vector<affine_element> derive_generators(const std::string_view& domain_separator,
                                                                          const size_t num_generators,
                                                                          const size_t starting_index = 0)
    {
        std::vector<uint8_t> domain_bytes;
        for (char i : domain_separator) {
            domain_bytes.emplace_back(static_cast<unsigned char>(i));
        }
        return derive_generators(domain_bytes, num_generators, starting_index);
    }

    BB_INLINE static void conditional_negate_affine(const affine_element* src,
                                                    affine_element* dest,
                                                    uint64_t predicate);
};

} // namespace bb

#ifdef DISABLE_SHENANIGANS
#include "group_impl_int128.tcc"
#else
#include "group_impl_asm.tcc"
#endif
