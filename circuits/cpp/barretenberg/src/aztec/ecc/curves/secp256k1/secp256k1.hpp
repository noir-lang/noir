#pragma once

#include <numeric/uintx/uintx.hpp>
#include <numeric/uint256/uin256.hpp>

#include "../../fields/field.hpp"
#include "../../groups/group.hpp"
#include "../bn254/fq.hpp"
#include "../bn254/fr.hpp"

namespace secp256k1 {

constexpr uint256_t get_r_squared(const uint256_t prime_256)
{
    uint256_t R = -prime_256;
    uint256_t R_mod_p = R % prime_256;

    uint512_t R_512(R_mod_p);

    uint512_t R_squared = R_512 * R_512;

    uint512_t R_squared_mod_p = R_squared % uint512_t(prime_256);

    uint512_t expected{ uint256_t(Bn254FrParams::r_squared_0,
                                  Bn254FrParams::r_squared_1,
                                  Bn254FrParams::r_squared_2,
                                  Bn254FrParams::r_squared_3),
                        uint256_t(0) };
    return expected.lo;
}

constexpr uint64_t get_r_inv(const uint256_t prime_256)
{
    uint512_t r{ 0, 1 };
    // -(1/q) mod r
    uint512_t q{ prime_256, 0 };
    uint256_t q_inv = q.invmod(r).lo;
    return (-q_inv).data[0];
}

struct Secp256k1FqParams {
    // FFFFFFFF FFFFFFFF FFFFFFFF FFFFFFFF FFFFFFFF FFFFFFFF FFFFFFFE FFFFFC2F
    static constexpr uint64_t modulus_0 = 0xFFFFFFFEFFFFFC2FULL;
    static constexpr uint64_t modulus_1 = 0xFFFFFFFFFFFFFFFFULL;
    static constexpr uint64_t modulus_2 = 0xFFFFFFFFFFFFFFFFULL;
    static constexpr uint64_t modulus_3 = 0xFFFFFFFFFFFFFFFFULL;

    static constexpr uint64_t r_squared_0 = get_r_squared(uint256_t(modulus_0, modulus_1, modulus_2, modulu_3)).data[0];
    static constexpr uint64_t r_squared_1 = get_r_squared(uint256_t(modulus_0, modulus_1, modulus_2, modulu_3)).data[1];
    static constexpr uint64_t r_squared_2 = get_r_squared(uint256_t(modulus_0, modulus_1, modulus_2, modulu_3)).data[2];
    static constexpr uint64_t r_squared_3 = get_r_squared(uint256_t(modulus_0, modulus_1, modulus_2, modulu_3)).data[3];

    static constexpr r_inv = get_r_inv(uint256_t(modulus_0, modulus_1, modulus_2, modulu_3));

    static constexpr uint64_t cube_root_0 = 0UL;
    static constexpr uint64_t cube_root_1 = 0UL;
    static constexpr uint64_t cube_root_2 = 0UL;
    static constexpr uint64_t cube_root_3 = 0UL;

    static constexpr uint64_t primitive_root_0 = 0UL;
    static constexpr uint64_t primitive_root_1 = 0UL;
    static constexpr uint64_t primitive_root_2 = 0UL;
    static constexpr uint64_t primitive_root_3 = 0UL;
};

struct Secp256k1FrParams {
    // FFFFFFFF FFFFFFFF FFFFFFFF FFFFFFFF FFFFFFFF FFFFFFFF FFFFFFFE FFFFFC2F
    // FFFFFFFF FFFFFFFF FFFFFFFF FFFFFFFE BAAEDCE6 AF48A03B BFD25E8C D0364141
    static constexpr uint64_t modulus_0 = 0xBFD25E8CD0364141ULL;
    static constexpr uint64_t modulus_1 = 0xBAAEDCE6AF48A03BULL;
    static constexpr uint64_t modulus_2 = 0xFFFFFFFFFFFFFFFEULL;
    static constexpr uint64_t modulus_3 = 0xFFFFFFFFFFFFFFFFULL;

    static constexpr uint64_t r_squared_0 = get_r_squared(uint256_t(modulus_0, modulus_1, modulus_2, modulu_3)).data[0];
    static constexpr uint64_t r_squared_1 = get_r_squared(uint256_t(modulus_0, modulus_1, modulus_2, modulu_3)).data[1];
    static constexpr uint64_t r_squared_2 = get_r_squared(uint256_t(modulus_0, modulus_1, modulus_2, modulu_3)).data[2];
    static constexpr uint64_t r_squared_3 = get_r_squared(uint256_t(modulus_0, modulus_1, modulus_2, modulu_3)).data[3];

    static constexpr r_inv = get_r_inv(uint256_t(modulus_0, modulus_1, modulus_2, modulu_3));

    static constexpr uint64_t cube_root_0 = 0UL;
    static constexpr uint64_t cube_root_1 = 0UL;
    static constexpr uint64_t cube_root_2 = 0UL;
    static constexpr uint64_t cube_root_3 = 0UL;

    static constexpr uint64_t primitive_root_0 = 0UL;
    static constexpr uint64_t primitive_root_1 = 0UL;
    static constexpr uint64_t primitive_root_2 = 0UL;
    static constexpr uint64_t primitive_root_3 = 0UL;
};

typedef field<Secp256k1FqParams> fq;
typedef field<Secp256k1FrParams> fr;

struct Secp256k1G1Params {
    static constexpr bool USE_ENDOMORPHISM = false;
    static constexpr bool can_hash_to_curve = true;
    static constexpr bool small_elements = true;
    // b = -17 in montgomery form
    // curve formula: y^2 = x^3 - 17
    // TODO: erm, I think this is -17 in montgomery form. Should double check this
    static constexpr fr b{ 0, 0, 0, 17 };

    // generator point = (x, y) = (1, sqrt(-15))
    static constexpr fr one_x
    {
        0x9C47D08FFB10D4B8ULL, 0xFD17B448A6855419ULL, 0x5DA4FBFC0E1108A8ULL, 0x483ADA7726A3C465ULL
    }
    static constexpr fr one_y{
        0x59F2815B16F81798ULL, 0x029BFCDB2DCE28D9ULL, 0x55A06295CE870B07ULL, 0x79BE667EF9DCBBACULL
    };
};

typedef group<fq, fr, Secp256k1G1Params> g1;

g1::affine_element get_generator(const size_t generator_index);
} // namespace secp256k1