#pragma once

#include "../../fields/field6.hpp"
#include "./fq.hpp"
#include "./fq2.hpp"

namespace bb {
struct Bn254Fq6Params {

#if defined(__SIZEOF_INT128__) && !defined(__wasm__)
    static constexpr fq2 frobenius_coeffs_c1_1{
        { 0xb5773b104563ab30UL, 0x347f91c8a9aa6454UL, 0x7a007127242e0991UL, 0x1956bcd8118214ecUL },
        { 0x6e849f1ea0aa4757UL, 0xaa1c7b6d89f89141UL, 0xb6e713cdfae0ca3aUL, 0x26694fbb4e82ebc3UL }
    };

    static constexpr fq2 frobenius_coeffs_c1_2{
        { 0x3350c88e13e80b9cUL, 0x7dce557cdb5e56b9UL, 0x6001b4b8b615564aUL, 0x2682e617020217e0UL },
        { 0UL, 0UL, 0UL, 0UL }
    };

    static constexpr fq2 frobenius_coeffs_c1_3{
        { 0xc9af22f716ad6badUL, 0xb311782a4aa662b2UL, 0x19eeaf64e248c7f4UL, 0x20273e77e3439f82UL },
        { 0xacc02860f7ce93acUL, 0x3933d5817ba76b4cUL, 0x69e6188b446c8467UL, 0x0a46036d4417cc55UL }
    };

    static constexpr fq2 frobenius_coeffs_c2_1{
        { 0x7361d77f843abe92UL, 0xa5bb2bd3273411fbUL, 0x9c941f314b3e2399UL, 0x15df9cddbb9fd3ecUL },
        { 0x5dddfd154bd8c949UL, 0x62cb29a5a4445b60UL, 0x37bc870a0c7dd2b9UL, 0x24830a9d3171f0fdUL }
    };

    static constexpr fq2 frobenius_coeffs_c2_2{
        { 0x71930c11d782e155UL, 0xa6bb947cffbe3323UL, 0xaa303344d4741444UL, 0x2c3b3f0d26594943UL },
        { 0UL, 0UL, 0UL, 0UL }
    };

    static constexpr fq2 frobenius_coeffs_c2_3{
        { 0x448a93a57b6762dfUL, 0xbfd62df528fdeadfUL, 0xd858f5d00e9bd47aUL, 0x06b03d4d3476ec58UL },
        { 0x2b19daf4bcc936d1UL, 0xa1a54e7a56f4299fUL, 0xb533eee05adeaef1UL, 0x170c812b84dda0b2UL }
    };
#else
    static constexpr fq2 frobenius_coeffs_c1_1{
        { 0xecdea09b24a59190UL, 0x17db8ffeae2fe1c2UL, 0xbb09c97c6dabac4dUL, 0x2492b3d41d289af3UL },
        { 0xf1663598f1142ef1UL, 0x77ec057e0bf56062UL, 0xdd0baaecb677a631UL, 0x135e4e31d284d463UL }
    };

    static constexpr fq2 frobenius_coeffs_c1_2{
        { 0x8aeb638758ccb791UL, 0xee27476838ae0f5bUL, 0x5fc8441d09282bUL, 0x169119a8426a57f9UL }, { 0UL, 0UL, 0UL, 0UL }
    };

    static constexpr fq2 frobenius_coeffs_c1_3{
        { 0x4738e103136caecdUL, 0xf491475bc376b8c3UL, 0x1f4034a3a97cbee8UL, 0xcad5f8fef61ccd7UL },
        { 0x2f41c395e6e485d6UL, 0x997230c70242aa46UL, 0xeae16f2184887ab5UL, 0x266696f73bcfc9b2UL }
    };

    static constexpr fq2 frobenius_coeffs_c2_1{
        { 0x227346b0b081f85eUL, 0x6e51a67130492bb5UL, 0x7e20162e52b19e16UL, 0x1677516f2343bb4bUL },
        { 0x18b280852f616a78UL, 0x25433712bde06eceUL, 0xb00a58256b9a0e66UL, 0x6f9f8e111971bbdUL }
    };

    static constexpr fq2 frobenius_coeffs_c2_2{
        { 0x62b1a3a46a337995UL, 0xadc97d2722e2726eUL, 0x64ee82ede2db85faUL, 0xc0afea1488a03bbUL },
        { 0UL, 0UL, 0UL, 0UL }
    };

    static constexpr fq2 frobenius_coeffs_c2_3{
        { 0xa0d044540af866c4UL, 0x9cc0145f7df631b3UL, 0x29dda327cd752de1UL, 0x14766fdb0a170a74UL },
        { 0xdd532940e9d402f7UL, 0x541490c5bfda559eUL, 0xd9c9c659c541b0b8UL, 0xbaf8cb569cbb3e4UL }
    };
#endif

    // non residue = 9 + i \in Fq2
    static inline constexpr fq2 mul_by_non_residue(const fq2& a)
    {
        // non residue = 9 + i \in Fq2
        // r.c0 = 9a0 - a1
        // r.c1 = 9a1 + a0
        fq T0 = a.c0 + a.c0;
        T0 += T0;
        T0 += T0;
        T0 += a.c0;
        fq T1 = a.c1 + a.c1;
        T1 += T1;
        T1 += T1;
        T1 += a.c1;
        fq T2 = T0 - a.c1;

        return { T2, T1 + a.c0 };
        T0 = a.c0 + a.c0;
    }
};

using fq6 = field6<fq2, Bn254Fq6Params>;
} // namespace bb