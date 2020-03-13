#pragma once

#include "../../groups/group.hpp"
#include "../bn254/fq.hpp"
#include "../bn254/fr.hpp"

namespace grumpkin {
typedef barretenberg::fr fq;
typedef barretenberg::fq fr;

struct GrumpkinG1Params {
    static constexpr bool USE_ENDOMORPHISM = true;
    static constexpr bool can_hash_to_curve = true;
    static constexpr bool small_elements = true;
    // b = -17 in montgomery form
    // curve formula: y^2 = x^3 - 17
    // TODO: erm, I think this is -17 in montgomery form. Should double check this
    static constexpr barretenberg::fr b{
        0xdd7056026000005a, 0x223fa97acb319311, 0xcc388229877910c0, 0x34394632b724eaa
    };

    // generator point = (x, y) = (1, sqrt(-15))
    static constexpr barretenberg::fr one_x = barretenberg::fr::one();
    static constexpr barretenberg::fr one_y{
        0x11b2dff1448c41d8UL, 0x23d3446f21c77dc3UL, 0xaa7b8cf435dfafbbUL, 0x14b34cf69dc25d68UL
    };
};
typedef barretenberg::group<barretenberg::fr, barretenberg::fq, GrumpkinG1Params> g1;

g1::affine_element get_generator(const size_t generator_index);
} // namespace grumpkin