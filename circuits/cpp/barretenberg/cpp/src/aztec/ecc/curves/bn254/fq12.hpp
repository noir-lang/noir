#pragma once

#include "../../fields/field12.hpp"
#include "./fq2.hpp"
#include "./fq6.hpp"

namespace barretenberg {
struct Bn254Fq12Params {
    static constexpr fq2 frobenius_coefficients_1{
        { 0xaf9ba69633144907UL, 0xca6b1d7387afb78aUL, 0x11bded5ef08a2087UL, 0x02f34d751a1f3a7cUL },
        { 0xa222ae234c492d72UL, 0xd00f02a4565de15bUL, 0xdc2ff3a253dfc926UL, 0x10a75716b3899551UL }
    };

    static constexpr fq2 frobenius_coefficients_2{
        { 0xca8d800500fa1bf2UL, 0xf0c5d61468b39769UL, 0x0e201271ad0d4418UL, 0x04290f65bad856e6UL },
        { 0UL, 0UL, 0UL, 0UL }
    };

    static constexpr fq2 frobenius_coefficients_3{
        { 0x365316184e46d97dUL, 0x0af7129ed4c96d9fUL, 0x659da72fca1009b5UL, 0x08116d8983a20d23UL },
        { 0xb1df4af7c39c1939UL, 0x3d9f02878a73bf7fUL, 0x9b2220928caf0ae0UL, 0x26684515eff054a6UL }
    };
};

typedef field12<fq2, fq6, Bn254Fq12Params> fq12;
} // namespace barretenberg