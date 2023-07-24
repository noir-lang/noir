#pragma once

#include <cstdint>

namespace barretenberg {

// // copies src into dest. n.b. both src and dest must be aligned on 32 byte boundaries
// template <typename coordinate_field, typename subgroup_field, typename GroupParams>
// inline void group<coordinate_field, subgroup_field, GroupParams>::copy(const affine_element* src, affine_element*
// dest)
// {
//     *dest = *src;
// }

// // copies src into dest. n.b. both src and dest must be aligned on 32 byte boundaries
// template <typename coordinate_field, typename subgroup_field, typename GroupParams>
// inline void group<coordinate_field, subgroup_field, GroupParams>::copy(const element* src, element* dest)
// {
//     *dest = *src;
// }

template <typename coordinate_field, typename subgroup_field, typename GroupParams>
inline void group<coordinate_field, subgroup_field, GroupParams>::conditional_negate_affine(const affine_element* src,
                                                                                            affine_element* dest,
                                                                                            uint64_t predicate)
{
    *dest = predicate ? -(*src) : (*src);
}
} // namespace barretenberg