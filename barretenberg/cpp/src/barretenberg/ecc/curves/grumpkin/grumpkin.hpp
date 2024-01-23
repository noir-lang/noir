#pragma once

#include "../../groups/group.hpp"
#include "../bn254/fq.hpp"
#include "../bn254/fr.hpp"

namespace grumpkin {

constexpr size_t MAX_NO_WRAP_INTEGER_BIT_LENGTH = 252;

using fq = bb::fr;
using fr = bb::fq;

struct GrumpkinG1Params {
    static constexpr bool USE_ENDOMORPHISM = true;
    static constexpr bool can_hash_to_curve = true;
    static constexpr bool small_elements = true;
    static constexpr bool has_a = false;
    // have checked in grumpkin.test_b that b is Montgomery form of -17
    static constexpr bb::fr b{ 0xdd7056026000005a, 0x223fa97acb319311, 0xcc388229877910c0, 0x34394632b724eaa };
    static constexpr bb::fr a{ 0UL, 0UL, 0UL, 0UL };

    // generator point = (x, y) = (1, sqrt(-16)), sqrt(-16) = 4i
    static constexpr bb::fr one_x = bb::fr::one();
    static constexpr bb::fr one_y{
        0x11b2dff1448c41d8UL, 0x23d3446f21c77dc3UL, 0xaa7b8cf435dfafbbUL, 0x14b34cf69dc25d68UL
    };
};
using g1 = bb::group<bb::fr, bb::fq, GrumpkinG1Params>;

}; // namespace grumpkin

namespace bb::curve {
class Grumpkin {
  public:
    using ScalarField = bb::fq;
    using BaseField = bb::fr;
    using Group = typename grumpkin::g1;
    using Element = typename Group::element;
    using AffineElement = typename Group::affine_element;

    // TODO(#673): This flag is temporary. It is needed in the verifier classes (GeminiVerifier, etc.) while these
    // classes are instantiated with "native" curve types. Eventually, the verifier classes will be instantiated only
    // with stdlib types, and "native" verification will be acheived via a simulated builder.
    static constexpr bool is_stdlib_type = false;
};
} // namespace bb::curve