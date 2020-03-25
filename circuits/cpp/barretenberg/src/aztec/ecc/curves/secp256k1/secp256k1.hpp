#pragma once

#include <numeric/uint256/uint256.hpp>
#include <numeric/uintx/uintx.hpp>

#include "../../fields/field.hpp"
#include "../../groups/group.hpp"
#include "../bn254/fq.hpp"
#include "../bn254/fr.hpp"

namespace secp256k1 {

constexpr uint256_t get_r_squared(const uint256_t prime_256)
{
    uint512_t R(0, 1);
    uint1024_t R_1024 = uint1024_t(R);
    uint1024_t R_squared = R_1024 * R_1024;
    uint1024_t modulus = uint1024_t(uint512_t(prime_256));

    uint1024_t R_squared_mod_p = R_squared % modulus;
    return R_squared_mod_p.lo.lo;
}

constexpr uint64_t get_r_inv(const uint256_t prime_256)
{
    uint512_t r{ 0, 1 };
    // -(1/q) mod r
    uint512_t q{ -prime_256, 0 };
    uint256_t q_inv = q.invmod(r).lo;
    return (q_inv).data[0];
}

struct Secp256k1FqParams {
    static constexpr uint64_t modulus_0 = 0xFFFFFFFEFFFFFC2FULL;
    static constexpr uint64_t modulus_1 = 0xFFFFFFFFFFFFFFFFULL;
    static constexpr uint64_t modulus_2 = 0xFFFFFFFFFFFFFFFFULL;
    static constexpr uint64_t modulus_3 = 0xFFFFFFFFFFFFFFFFULL;

    static constexpr uint64_t r_squared_0 =
        get_r_squared(uint256_t(modulus_0, modulus_1, modulus_2, modulus_3)).data[0];
    static constexpr uint64_t r_squared_1 =
        get_r_squared(uint256_t(modulus_0, modulus_1, modulus_2, modulus_3)).data[1];
    static constexpr uint64_t r_squared_2 =
        get_r_squared(uint256_t(modulus_0, modulus_1, modulus_2, modulus_3)).data[2];
    static constexpr uint64_t r_squared_3 =
        get_r_squared(uint256_t(modulus_0, modulus_1, modulus_2, modulus_3)).data[3];

    static constexpr uint64_t r_inv = get_r_inv(uint256_t(modulus_0, modulus_1, modulus_2, modulus_3));

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
    static constexpr uint64_t modulus_0 = 0xBFD25E8CD0364141ULL;
    static constexpr uint64_t modulus_1 = 0xBAAEDCE6AF48A03BULL;
    static constexpr uint64_t modulus_2 = 0xFFFFFFFFFFFFFFFEULL;
    static constexpr uint64_t modulus_3 = 0xFFFFFFFFFFFFFFFFULL;

    static constexpr uint64_t r_squared_0 =
        get_r_squared(uint256_t(modulus_0, modulus_1, modulus_2, modulus_3)).data[0];
    static constexpr uint64_t r_squared_1 =
        get_r_squared(uint256_t(modulus_0, modulus_1, modulus_2, modulus_3)).data[1];
    static constexpr uint64_t r_squared_2 =
        get_r_squared(uint256_t(modulus_0, modulus_1, modulus_2, modulus_3)).data[2];
    static constexpr uint64_t r_squared_3 =
        get_r_squared(uint256_t(modulus_0, modulus_1, modulus_2, modulus_3)).data[3];

    static constexpr uint64_t r_inv = get_r_inv(uint256_t(modulus_0, modulus_1, modulus_2, modulus_3));

    static constexpr uint64_t cube_root_0 = 0UL;
    static constexpr uint64_t cube_root_1 = 0UL;
    static constexpr uint64_t cube_root_2 = 0UL;
    static constexpr uint64_t cube_root_3 = 0UL;

    static constexpr uint64_t primitive_root_0 = 0UL;
    static constexpr uint64_t primitive_root_1 = 0UL;
    static constexpr uint64_t primitive_root_2 = 0UL;
    static constexpr uint64_t primitive_root_3 = 0UL;
};

typedef barretenberg::field<Secp256k1FqParams> fq;
typedef barretenberg::field<Secp256k1FrParams> fr;

struct Secp256k1G1Params {
    static constexpr bool USE_ENDOMORPHISM = false;
    static constexpr bool can_hash_to_curve = true;
    static constexpr bool small_elements = true;
    static constexpr bool has_a = false;

    static constexpr fq b = fq(7);

    static constexpr fq one_x =
        fq(0x59F2815B16F81798UL, 0x029BFCDB2DCE28D9UL, 0x55A06295CE870B07UL, 0x79BE667EF9DCBBACUL).to_montgomery_form();
    static constexpr fq one_y =
        fq(0x9C47D08FFB10D4B8UL, 0xFD17B448A6855419UL, 0x5DA4FBFC0E1108A8UL, 0x483ADA7726A3C465UL).to_montgomery_form();
};

typedef barretenberg::
    group<barretenberg::field<Secp256k1FqParams>, barretenberg::field<Secp256k1FrParams>, Secp256k1G1Params>
        g1;

g1::affine_element get_generator(const size_t generator_index);
} // namespace secp256k1