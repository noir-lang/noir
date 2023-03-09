#pragma once

#include <cstdint>
#include <iomanip>

#include "../../fields/field.hpp"

namespace barretenberg {
class Bn254FqParams {
  public:
    static constexpr uint64_t modulus_0 = 0x3C208C16D87CFD47UL;
    static constexpr uint64_t modulus_1 = 0x97816a916871ca8dUL;
    static constexpr uint64_t modulus_2 = 0xb85045b68181585dUL;
    static constexpr uint64_t modulus_3 = 0x30644e72e131a029UL;

    static constexpr uint64_t r_squared_0 = 0xF32CFC5B538AFA89UL;
    static constexpr uint64_t r_squared_1 = 0xB5E71911D44501FBUL;
    static constexpr uint64_t r_squared_2 = 0x47AB1EFF0A417FF6UL;
    static constexpr uint64_t r_squared_3 = 0x06D89F71CAB8351FUL;

    static constexpr uint64_t cube_root_0 = 0x71930c11d782e155UL;
    static constexpr uint64_t cube_root_1 = 0xa6bb947cffbe3323UL;
    static constexpr uint64_t cube_root_2 = 0xaa303344d4741444UL;
    static constexpr uint64_t cube_root_3 = 0x2c3b3f0d26594943UL;

    static constexpr uint64_t primitive_root_0 = 0UL;
    static constexpr uint64_t primitive_root_1 = 0UL;
    static constexpr uint64_t primitive_root_2 = 0UL;
    static constexpr uint64_t primitive_root_3 = 0UL;

    static constexpr uint64_t endo_g1_lo = 0x7a7bd9d4391eb18d;
    static constexpr uint64_t endo_g1_mid = 0x4ccef014a773d2cfUL;
    static constexpr uint64_t endo_g1_hi = 0x0000000000000002UL;
    static constexpr uint64_t endo_g2_lo = 0xd91d232ec7e0b3d2UL;
    static constexpr uint64_t endo_g2_mid = 0x0000000000000002UL;
    static constexpr uint64_t endo_minus_b1_lo = 0x8211bbeb7d4f1129UL;
    static constexpr uint64_t endo_minus_b1_mid = 0x6f4d8248eeb859fcUL;
    static constexpr uint64_t endo_b2_lo = 0x89d3256894d213e2UL;
    static constexpr uint64_t endo_b2_mid = 0UL;

    static constexpr uint64_t r_inv = 0x87d20782e4866389UL;

    static constexpr uint64_t coset_generators_0[8]{
        0x7a17caa950ad28d7ULL, 0x4d750e37163c3674ULL, 0x20d251c4dbcb4411ULL, 0xf42f9552a15a51aeULL,
        0x4f4bc0b2b5ef64bdULL, 0x22a904407b7e725aULL, 0xf60647ce410d7ff7ULL, 0xc9638b5c069c8d94ULL,
    };
    static constexpr uint64_t coset_generators_1[8]{
        0x1f6ac17ae15521b9ULL, 0x29e3aca3d71c2cf7ULL, 0x345c97cccce33835ULL, 0x3ed582f5c2aa4372ULL,
        0x1a4b98fbe78db996ULL, 0x24c48424dd54c4d4ULL, 0x2f3d6f4dd31bd011ULL, 0x39b65a76c8e2db4fULL,
    };
    static constexpr uint64_t coset_generators_2[8]{
        0x334bea4e696bd284ULL, 0x99ba8dbde1e518b0ULL, 0x29312d5a5e5edcULL,   0x6697d49cd2d7a508ULL,
        0x5c65ec9f484e3a79ULL, 0xc2d4900ec0c780a5ULL, 0x2943337e3940c6d1ULL, 0x8fb1d6edb1ba0cfdULL,
    };
    static constexpr uint64_t coset_generators_3[8]{
        0x2a1f6744ce179d8eULL, 0x3829df06681f7cbdULL, 0x463456c802275bedULL, 0x543ece899c2f3b1cULL,
        0x180a96573d3d9f8ULL,  0xf8b21270ddbb927ULL,  0x1d9598e8a7e39857ULL, 0x2ba010aa41eb7786ULL,
    };
};

typedef field<Bn254FqParams> fq;

} // namespace barretenberg