#pragma once

#include <cstdint>
#include <iomanip>
#include <ostream>

#include "../../fields/field.hpp"

namespace barretenberg {
class Bn254FrParams {
  public:
    // Note: limbs here are combined as concat(_3, _2, _1, _0)
    // E.g. this modulus forms the value:
    // 0x30644E72E131A029B85045B68181585D2833E84879B9709143E1F593F0000001
    // = 21888242871839275222246405745257275088548364400416034343698204186575808495617
    static constexpr uint64_t modulus_0 = 0x43E1F593F0000001UL;
    static constexpr uint64_t modulus_1 = 0x2833E84879B97091UL;
    static constexpr uint64_t modulus_2 = 0xB85045B68181585DUL;
    static constexpr uint64_t modulus_3 = 0x30644E72E131A029UL;

    static constexpr uint64_t r_squared_0 = 0x1BB8E645AE216DA7UL;
    static constexpr uint64_t r_squared_1 = 0x53FE3AB1E35C59E3UL;
    static constexpr uint64_t r_squared_2 = 0x8C49833D53BB8085UL;
    static constexpr uint64_t r_squared_3 = 0x216D0B17F4E44A5UL;

    static constexpr uint64_t cube_root_0 = 0x93e7cede4a0329b3UL;
    static constexpr uint64_t cube_root_1 = 0x7d4fdca77a96c167UL;
    static constexpr uint64_t cube_root_2 = 0x8be4ba08b19a750aUL;
    static constexpr uint64_t cube_root_3 = 0x1cbd5653a5661c25UL;

    static constexpr uint64_t primitive_root_0 = 0x636e735580d13d9cUL;
    static constexpr uint64_t primitive_root_1 = 0xa22bf3742445ffd6UL;
    static constexpr uint64_t primitive_root_2 = 0x56452ac01eb203d8UL;
    static constexpr uint64_t primitive_root_3 = 0x1860ef942963f9e7UL;

    static constexpr uint64_t endo_g1_lo = 0x7a7bd9d4391eb18dUL;
    static constexpr uint64_t endo_g1_mid = 0x4ccef014a773d2cfUL;
    static constexpr uint64_t endo_g1_hi = 0x0000000000000002UL;
    static constexpr uint64_t endo_g2_lo = 0xd91d232ec7e0b3d7UL;
    static constexpr uint64_t endo_g2_mid = 0x0000000000000002UL;
    static constexpr uint64_t endo_minus_b1_lo = 0x8211bbeb7d4f1128UL;
    static constexpr uint64_t endo_minus_b1_mid = 0x6f4d8248eeb859fcUL;
    static constexpr uint64_t endo_b2_lo = 0x89d3256894d213e3UL;
    static constexpr uint64_t endo_b2_mid = 0UL;

    static constexpr uint64_t r_inv = 0xc2e1f593efffffffUL;

    static constexpr uint64_t coset_generators_0[8]{
        0x5eef048d8fffffe7ULL, 0xb8538a9dfffffe2ULL,  0x3057819e4fffffdbULL, 0xdcedb5ba9fffffd6ULL,
        0x8983e9d6efffffd1ULL, 0x361a1df33fffffccULL, 0xe2b0520f8fffffc7ULL, 0x8f46862bdfffffc2ULL,
    };
    static constexpr uint64_t coset_generators_1[8]{
        0x12ee50ec1ce401d0ULL, 0x49eac781bc44cefaULL, 0x307f6d866832bb01ULL, 0x677be41c0793882aULL,
        0x9e785ab1a6f45554ULL, 0xd574d1474655227eULL, 0xc7147dce5b5efa7ULL,  0x436dbe728516bcd1ULL,
    };
    static constexpr uint64_t coset_generators_2[8]{
        0x29312d5a5e5ee7ULL,   0x6697d49cd2d7a515ULL, 0x5c65ec9f484e3a89ULL, 0xc2d4900ec0c780b7ULL,
        0x2943337e3940c6e5ULL, 0x8fb1d6edb1ba0d13ULL, 0xf6207a5d2a335342ULL, 0x5c8f1dcca2ac9970ULL,
    };
    static constexpr uint64_t coset_generators_3[8]{
        0x463456c802275bedULL, 0x543ece899c2f3b1cULL, 0x180a96573d3d9f8ULL,  0xf8b21270ddbb927ULL,
        0x1d9598e8a7e39857ULL, 0x2ba010aa41eb7786ULL, 0x39aa886bdbf356b5ULL, 0x47b5002d75fb35e5ULL,
    };
};

typedef field<Bn254FrParams> fr;

} // namespace barretenberg