#pragma once

#include "../../groups/group.hpp"
#include "./fq2.hpp"
#include "./fr.hpp"

namespace barretenberg {
struct Bn254G2Params {
    static constexpr bool USE_ENDOMORPHISM = false;
    static constexpr bool can_hash_to_curve = false;
    static constexpr bool small_elements = false;
    static constexpr fq2 one_x{ { 0x8e83b5d102bc2026, 0xdceb1935497b0172, 0xfbb8264797811adf, 0x19573841af96503b },
                                { 0xafb4737da84c6140, 0x6043dd5a5802d8c4, 0x09e950fc52a02f86, 0x14fef0833aea7b6b } };
    static constexpr fq2 one_y{ { 0x619dfa9d886be9f6, 0xfe7fd297f59e9b78, 0xff9e1a62231b7dfe, 0x28fd7eebae9e4206 },
                                { 0x64095b56c71856ee, 0xdc57f922327d3cbb, 0x55f935be33351076, 0x0da4a0e693fd6482 } };
    static constexpr fq2 b = fq2::twist_coeff_b();
};

typedef group<fq2, fr, Bn254G2Params> g2;
} // namespace barretenberg