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
template <typename coordinate_field, typename subgroup_field, typename GroupParams> class group {
  public:
    typedef group_elements::element<coordinate_field, subgroup_field, GroupParams> element;
    typedef group_elements::affine_element<coordinate_field, subgroup_field, GroupParams> affine_element;

    static constexpr element one{ GroupParams::one_x, GroupParams::one_y, coordinate_field::one() };
    static constexpr element point_at_infinity = one.set_infinity();
    static constexpr affine_element affine_one{ GroupParams::one_x, GroupParams::one_y };
    static constexpr coordinate_field curve_b = GroupParams::b;

    template <size_t N> static inline std::array<affine_element, N> derive_generators()
    {
        std::array<affine_element, N> generators;
        size_t count = 0;
        size_t seed = 0;
        while (count < N) {
            ++seed;
            affine_element candidate = affine_element::hash_to_curve(seed);
            if (candidate.on_curve()) {
                generators[count] = candidate;
                ++count;
            }
        }

        return generators;
    }

    BBERG_INLINE static void conditional_negate_affine(const affine_element* src,
                                                       affine_element* dest,
                                                       uint64_t predicate);

}; // class group
} // namespace barretenberg

#ifdef DISABLE_SHENANIGANS
#include "group_impl_int128.tcc"
#else
#include "group_impl_asm.tcc"
#endif
