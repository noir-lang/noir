#pragma once

#include "../../groups/group.hpp"
#include "./fq2.hpp"
#include "./fr.hpp"

namespace bb {
struct Bn254G2Params {
    static constexpr bool USE_ENDOMORPHISM = false;
    static constexpr bool can_hash_to_curve = false;
    static constexpr bool small_elements = false;
    static constexpr bool has_a = false;

#if defined(__SIZEOF_INT128__) && !defined(__wasm__)
    static constexpr fq2 one_x{ { 0x8e83b5d102bc2026, 0xdceb1935497b0172, 0xfbb8264797811adf, 0x19573841af96503b },
                                { 0xafb4737da84c6140, 0x6043dd5a5802d8c4, 0x09e950fc52a02f86, 0x14fef0833aea7b6b } };
    static constexpr fq2 one_y{ { 0x619dfa9d886be9f6, 0xfe7fd297f59e9b78, 0xff9e1a62231b7dfe, 0x28fd7eebae9e4206 },
                                { 0x64095b56c71856ee, 0xdc57f922327d3cbb, 0x55f935be33351076, 0x0da4a0e693fd6482 } };
#else
    static constexpr fq2 one_x{
        { 0xe6df8b2cfb43050UL, 0x254c7d92a843857eUL, 0xf2006d8ad80dd622UL, 0x24a22107dfb004e3UL },
        { 0xe8e7528c0b334b65UL, 0x56e941e8b293cf69UL, 0xe1169545c074740bUL, 0x2ac61491edca4b42UL }
    };
    static constexpr fq2 one_y{
        { 0xdc508d48384e8843UL, 0xd55415a8afd31226UL, 0x834bf204bacb6e00UL, 0x51b9758138c5c79UL },
        { 0x64067e0b46a5f641UL, 0x37726529a3a77875UL, 0x4454445bd915f391UL, 0x10d5ac894edeed3UL }
    };
#endif
    static constexpr fq2 a = fq2::zero();
    static constexpr fq2 b = fq2::twist_coeff_b();
};

using g2 = group<fq2, fr, Bn254G2Params>;
} // namespace bb