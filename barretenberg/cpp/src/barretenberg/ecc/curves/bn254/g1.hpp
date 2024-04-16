#pragma once

#include "../../groups/group.hpp"
#include "./fq.hpp"
#include "./fr.hpp"

namespace bb {
struct Bn254G1Params {
    static constexpr bool USE_ENDOMORPHISM = true;
    static constexpr bool can_hash_to_curve = true;
    static constexpr bool small_elements = true;
    static constexpr bool has_a = false;
    static constexpr fq one_x = fq::one();
#if defined(__SIZEOF_INT128__) && !defined(__wasm__)
    static constexpr fq one_y{ 0xa6ba871b8b1e1b3aUL, 0x14f1d651eb8e167bUL, 0xccdd46def0f28c58UL, 0x1c14ef83340fbe5eUL };
#else
    static constexpr fq one_y{ 0x9d0709d62af99842UL, 0xf7214c0419c29186UL, 0xa603f5090339546dUL, 0x1b906c52ac7a88eaUL };
#endif
    static constexpr fq a{ 0UL, 0UL, 0UL, 0UL };
#if defined(__SIZEOF_INT128__) && !defined(__wasm__)
    static constexpr fq b{ 0x7a17caa950ad28d7UL, 0x1f6ac17ae15521b9UL, 0x334bea4e696bd284UL, 0x2a1f6744ce179d8eUL };
#else
    static constexpr fq b{ 0xeb8a8ec140766463UL, 0xf2b1f20626a3da49UL, 0xf905ef8d84d5fea4UL, 0x2958a27c02b7cd5fUL };
#endif
};

using g1 = group<fq, fr, Bn254G1Params>;

} // namespace bb

// specialize the name in msgpack schema generation
// consumed by the typescript schema compiler, helps disambiguate templates
inline std::string msgpack_schema_name(bb::g1::affine_element const& /*unused*/)
{
    return "G1AffineElement";
}
