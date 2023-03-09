#pragma once

#include "../../fields/field6.hpp"
#include "./fq.hpp"
#include "./fq2.hpp"

namespace barretenberg {
struct Bn254Fq6Params {
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

typedef field6<fq2, Bn254Fq6Params> fq6;
} // namespace barretenberg