#pragma once

#include "../../groups/group.hpp"
#include "../bn254/fq.hpp"
#include "../bn254/fr.hpp"

namespace bb::grumpkin {

constexpr size_t MAX_NO_WRAP_INTEGER_BIT_LENGTH = 252;

using fq = bb::fr;
using fr = bb::fq;

struct G1Params {
    static constexpr bool USE_ENDOMORPHISM = true;
    static constexpr bool can_hash_to_curve = true;
    static constexpr bool small_elements = true;
    static constexpr bool has_a = false;
// have checked in grumpkin.test_b that b is Montgomery form of -17
#if defined(__SIZEOF_INT128__) && !defined(__wasm__)
    static constexpr bb::fr b{ 0xdd7056026000005a, 0x223fa97acb319311, 0xcc388229877910c0, 0x34394632b724eaa };
#else
    static constexpr bb::fr b{ 0x2646d52420000b3eUL, 0xf78d5ec872bf8119UL, 0x166fb9c3ec1f6749UL, 0x7a9ef7fabe69506UL };
#endif
    static constexpr bb::fr a{ 0UL, 0UL, 0UL, 0UL };

    // generator point = (x, y) = (1, sqrt(-16)), sqrt(-16) = 4i
    static constexpr bb::fr one_x = bb::fr::one();
#if defined(__SIZEOF_INT128__) && !defined(__wasm__)
    static constexpr bb::fr one_y{
        0x11b2dff1448c41d8UL, 0x23d3446f21c77dc3UL, 0xaa7b8cf435dfafbbUL, 0x14b34cf69dc25d68UL
    };
#else
    static constexpr bb::fr one_y{
        0xc3e285a561883af3UL, 0x6fc5c2360a850101UL, 0xf35e144228647aa9UL, 0x2151a2fe48c68af6UL
    };
#endif
};
using g1 = bb::group<bb::fr, bb::fq, G1Params>;

}; // namespace bb::grumpkin

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