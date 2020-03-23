#pragma once

#include "../../groups/group.hpp"
#include "./fq.hpp"
#include "./fr.hpp"

namespace barretenberg {
struct Bn254G1Params {
    static constexpr bool USE_ENDOMORPHISM = true;
    static constexpr bool can_hash_to_curve = true;
    static constexpr bool small_elements = true;
    static constexpr bool has_a = false;
    static constexpr fq one_x = fq::one();
    static constexpr fq one_y{ 0xa6ba871b8b1e1b3aUL, 0x14f1d651eb8e167bUL, 0xccdd46def0f28c58UL, 0x1c14ef83340fbe5eUL };
    static constexpr fq b{ 0x7a17caa950ad28d7UL, 0x1f6ac17ae15521b9UL, 0x334bea4e696bd284UL, 0x2a1f6744ce179d8eUL };
};

typedef group<fq, fr, Bn254G1Params> g1;
} // namespace barretenberg