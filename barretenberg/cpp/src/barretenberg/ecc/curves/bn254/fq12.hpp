#pragma once

#include "../../fields/field12.hpp"
#include "./fq2.hpp"
#include "./fq6.hpp"

namespace bb {
struct Bn254Fq12Params {

#if defined(__SIZEOF_INT128__) && !defined(__wasm__)
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
#else
    static constexpr fq2 frobenius_coefficients_1{
        { 0xb75446af8a0c2399UL, 0xb5e243df8d8526c8UL, 0x7f6d66278fc2b89bUL, 0x2e05603062b5af58UL },
        { 0xaeefbf6e3bc6cc33UL, 0x7f50c04b4ed87762UL, 0x9a8b7572eb6a58d4UL, 0x9b83e6c410c870UL }
    };

    static constexpr fq2 frobenius_coefficients_2{
        { 0xd96ee8726e4983b2UL, 0xe9b7ed6a458f581eUL, 0x5361c2c89ea5d262UL, 0x24594fd198a79c6eUL },
        { 0UL, 0UL, 0UL, 0UL }
    };

    static constexpr fq2 frobenius_coefficients_3{
        { 0x9dc006978e6a3d3dUL, 0x695b3f038ef4bf24UL, 0x1a238968ba7a7ccdUL, 0x103828f20e49839cUL },
        { 0x5cbbb0bd4f4e6b31UL, 0xe83ce8be1b5b282bUL, 0x646d437ef03fbae3UL, 0x133cf9860031f0c0UL }
    };
#endif
};

using fq12 = field12<fq2, fq6, Bn254Fq12Params>;
} // namespace bb