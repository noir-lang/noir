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
};

typedef field<Bn254FqParams> fq;

} // namespace barretenberg