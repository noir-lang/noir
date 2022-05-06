#pragma once

#include "../../common/assert.hpp"
#include "./wnaf.hpp"
#include <array>
#include <cinttypes>
#include <cstdint>
#include <cstdio>
#include <cstdlib>
#include "./affine_element.hpp"
#include "./element.hpp"

namespace barretenberg {

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
    typedef group_elements::element<coordinate_field, subgroup_field, GroupParams> element;
    typedef group_elements::affine_element<coordinate_field, subgroup_field, GroupParams> affine_element;

    static constexpr bool USE_ENDOMORPHISM = GroupParams::USE_ENDOMORPHISM;
    static constexpr bool has_a = GroupParams::has_a;

    static constexpr element one{ GroupParams::one_x, GroupParams::one_y, coordinate_field::one() };
    static constexpr element point_at_infinity = one.set_infinity();
    static constexpr affine_element affine_one{ GroupParams::one_x, GroupParams::one_y };
    static constexpr affine_element affine_point_at_infinity = affine_one.set_infinity();

    static constexpr coordinate_field curve_a = GroupParams::a;
    static constexpr coordinate_field curve_b = GroupParams::b;

    template <size_t N> static inline auto derive_generators()
    {
        std::array<affine_element, N> generators;
        size_t count = 0;
        size_t seed = 0;
        while (count < N) {
            ++seed;
            auto [on_curve, candidate] = affine_element::hash_to_curve(seed);
            if (on_curve && !candidate.is_point_at_infinity()) {
                generators[count] = candidate;
                ++count;
            }
        }

        return generators;
    }

    BBERG_INLINE static void conditional_negate_affine(const affine_element* src,
                                                       affine_element* dest,
                                                       uint64_t predicate);
};

} // namespace barretenberg

#ifdef DISABLE_SHENANIGANS
#include "group_impl_int128.tcc"
#else
#include "group_impl_asm.tcc"
#endif
