#pragma once

#include <cstdint>

namespace barretenberg {
// copies src into dest. n.b. both src and dest must be aligned on 32 byte boundaries
// template <typename coordinate_field, typename subgroup_field, typename GroupParams>
// inline void group<coordinate_field, subgroup_field, GroupParams>::copy(const affine_element* src, affine_element*
// dest)
// {
//     if constexpr (GroupParams::small_elements) {
// #if defined __AVX__ && defined USE_AVX
//         ASSERT((((uintptr_t)src & 0x1f) == 0));
//         ASSERT((((uintptr_t)dest & 0x1f) == 0));
//         __asm__ __volatile__("vmovdqa 0(%0), %%ymm0              \n\t"
//                              "vmovdqa 32(%0), %%ymm1             \n\t"
//                              "vmovdqa %%ymm0, 0(%1)              \n\t"
//                              "vmovdqa %%ymm1, 32(%1)             \n\t"
//                              :
//                              : "r"(src), "r"(dest)
//                              : "%ymm0", "%ymm1", "memory");
// #else
//         *dest = *src;
// #endif
//     } else {
//         *dest = *src;
//     }
// }

// // copies src into dest. n.b. both src and dest must be aligned on 32 byte boundaries
// template <typename coordinate_field, typename subgroup_field, typename GroupParams>
// inline void group<coordinate_field, subgroup_field, GroupParams>::copy(const element* src, element* dest)
// {
//     if constexpr (GroupParams::small_elements) {
// #if defined __AVX__ && defined USE_AVX
//         ASSERT((((uintptr_t)src & 0x1f) == 0));
//         ASSERT((((uintptr_t)dest & 0x1f) == 0));
//         __asm__ __volatile__("vmovdqa 0(%0), %%ymm0              \n\t"
//                              "vmovdqa 32(%0), %%ymm1             \n\t"
//                              "vmovdqa 64(%0), %%ymm2             \n\t"
//                              "vmovdqa %%ymm0, 0(%1)              \n\t"
//                              "vmovdqa %%ymm1, 32(%1)             \n\t"
//                              "vmovdqa %%ymm2, 64(%1)             \n\t"
//                              :
//                              : "r"(src), "r"(dest)
//                              : "%ymm0", "%ymm1", "%ymm2", "memory");
// #else
//         *dest = *src;
// #endif
//     } else {
//         *dest = src;
//     }
// }

// copies src into dest, inverting y-coordinate if 'predicate' is true
// n.b. requires src and dest to be aligned on 32 byte boundary
template <typename coordinate_field, typename subgroup_field, typename GroupParams>
inline void group<coordinate_field, subgroup_field, GroupParams>::conditional_negate_affine(const affine_element* src,
                                                                                            affine_element* dest,
                                                                                            uint64_t predicate)
{
    constexpr uint256_t twice_modulus = coordinate_field::modulus + coordinate_field::modulus;

    constexpr uint64_t twice_modulus_0 = twice_modulus.data[0];
    constexpr uint64_t twice_modulus_1 = twice_modulus.data[1];
    constexpr uint64_t twice_modulus_2 = twice_modulus.data[2];
    constexpr uint64_t twice_modulus_3 = twice_modulus.data[3];

    if constexpr (GroupParams::small_elements) {
#if defined __AVX__ && defined USE_AVX
        ASSERT((((uintptr_t)src & 0x1f) == 0));
        ASSERT((((uintptr_t)dest & 0x1f) == 0));
        __asm__ __volatile__("xorq %%r8, %%r8                              \n\t"
                             "movq 32(%0), %%r8                            \n\t"
                             "movq 40(%0), %%r9                            \n\t"
                             "movq 48(%0), %%r10                          \n\t"
                             "movq 56(%0), %%r11                          \n\t"
                             "movq %[modulus_0], %%r12                  \n\t"
                             "movq %[modulus_1], %%r13                  \n\t"
                             "movq %[modulus_2], %%r14                  \n\t"
                             "movq %[modulus_3], %%r15                  \n\t"
                             "subq %%r8, %%r12                               \n\t"
                             "sbbq %%r9, %%r13                               \n\t"
                             "sbbq %%r10, %%r14                              \n\t"
                             "sbbq %%r11, %%r15                              \n\t"
                             "testq %2, %2                                   \n\t"
                             "cmovnzq %%r12, %%r8                               \n\t"
                             "cmovnzq %%r13, %%r9                               \n\t"
                             "cmovnzq %%r14, %%r10                              \n\t"
                             "cmovnzq %%r15, %%r11                              \n\t"
                             "vmovdqa 0(%0), %%ymm0                         \n\t"
                             "vmovdqa %%ymm0, 0(%1)                      \n\t"
                             "movq %%r8, 32(%1)                             \n\t"
                             "movq %%r9, 40(%1)                             \n\t"
                             "movq %%r10, 48(%1)                           \n\t"
                             "movq %%r11, 56(%1)                           \n\t"
                             :
                             : "r"(src),
                               "r"(dest),
                               "r"(predicate),
                               [ modulus_0 ] "i"(twice_modulus_0),
                               [ modulus_1 ] "i"(twice_modulus_1),
                               [ modulus_2 ] "i"(twice_modulus_2),
                               [ modulus_3 ] "i"(twice_modulus_3)
                             : "%r8", "%r9", "%r10", "%r11", "%r12", "%r13", "%r14", "%r15", "%ymm0", "memory", "cc");
#else
        __asm__ __volatile__("xorq %%r8, %%r8                              \n\t"
                             "movq 32(%0), %%r8                            \n\t"
                             "movq 40(%0), %%r9                            \n\t"
                             "movq 48(%0), %%r10                          \n\t"
                             "movq 56(%0), %%r11                          \n\t"
                             "movq %[modulus_0], %%r12                  \n\t"
                             "movq %[modulus_1], %%r13                  \n\t"
                             "movq %[modulus_2], %%r14                  \n\t"
                             "movq %[modulus_3], %%r15                  \n\t"
                             "subq %%r8, %%r12                               \n\t"
                             "sbbq %%r9, %%r13                               \n\t"
                             "sbbq %%r10, %%r14                              \n\t"
                             "sbbq %%r11, %%r15                              \n\t"
                             "testq %2, %2                                   \n\t"
                             "cmovnzq %%r12, %%r8                               \n\t"
                             "cmovnzq %%r13, %%r9                               \n\t"
                             "cmovnzq %%r14, %%r10                              \n\t"
                             "cmovnzq %%r15, %%r11                              \n\t"
                             "movq 0(%0), %%r12                            \n\t"
                             "movq 8(%0), %%r13                            \n\t"
                             "movq 16(%0), %%r14                          \n\t"
                             "movq 24(%0), %%r15                          \n\t"
                             "movq %%r8, 32(%1)                             \n\t"
                             "movq %%r9, 40(%1)                             \n\t"
                             "movq %%r10, 48(%1)                           \n\t"
                             "movq %%r11, 56(%1)                           \n\t"
                             "movq %%r12, 0(%1)                              \n\t"
                             "movq %%r13, 8(%1)                          \n\t"
                             "movq %%r14, 16(%1)                          \n\t"
                             "movq %%r15, 24(%1)                          \n\t"
                             :
                             : "r"(src),
                               "r"(dest),
                               "r"(predicate),
                               [ modulus_0 ] "i"(twice_modulus_0),
                               [ modulus_1 ] "i"(twice_modulus_1),
                               [ modulus_2 ] "i"(twice_modulus_2),
                               [ modulus_3 ] "i"(twice_modulus_3)
                             : "%r8", "%r9", "%r10", "%r11", "%r12", "%r13", "%r14", "%r15", "memory", "cc");
#endif
    } else {
        if (predicate) {
            coordinate_field::__copy(src->x, dest->x);
            dest->y = -src->y;
        } else {
            copy_affine(*src, *dest);
        }
    }
}

} // namespace barretenberg