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

    static constexpr uint64_t r_squared_0 = 8392367050913ULL;
    static constexpr uint64_t r_squared_1 = 1;
    static constexpr uint64_t r_squared_2 = 0;
    static constexpr uint64_t r_squared_3 = 0;

    static constexpr uint64_t coset_generators_0[8]{
        0x300000b73ULL, 0x400000f44ULL, 0x500001315ULL, 0x6000016e6ULL,
        0x700001ab7ULL, 0x800001e88ULL, 0x900002259ULL, 0xa0000262aULL,
    };
    static constexpr uint64_t coset_generators_1[8]{
        0, 0, 0, 0, 0, 0, 0, 0,
    };
    static constexpr uint64_t coset_generators_2[8]{
        0, 0, 0, 0, 0, 0, 0, 0,
    };
    static constexpr uint64_t coset_generators_3[8]{
        0, 0, 0, 0, 0, 0, 0, 0,
    };

    static constexpr uint64_t r_inv =
        15580212934572586289ULL;

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

    static constexpr uint64_t r_squared_0 = 9902555850136342848ULL;
    static constexpr uint64_t r_squared_1 = 8364476168144746616ULL;
    static constexpr uint64_t r_squared_2 = 16616019711348246470ULL;
    static constexpr uint64_t r_squared_3 = 11342065889886772165ULL;

    static constexpr uint64_t r_inv =
        5408259542528602431ULL;

    static constexpr uint64_t coset_generators_0[8]{
        0x40e4273feef0b9bbULL, 0x8111c8b31eba787aULL, 0xc13f6a264e843739ULL, 0x16d0b997e4df5f8ULL,
        0x419aad0cae17b4b7ULL, 0x81c84e7fdde17376ULL, 0xc1f5eff30dab3235ULL, 0x22391663d74f0f4ULL,
    };
    static constexpr uint64_t coset_generators_1[8]{
        0x5a95af7e9394ded5ULL, 0x9fe6d297e44c3e99ULL, 0xe537f5b135039e5dULL, 0x2a8918ca85bafe22ULL,
        0x6fda3be3d6725de6ULL, 0xb52b5efd2729bdaaULL, 0xfa7c821677e11d6eULL, 0x3fcda52fc8987d33ULL,
    };
    static constexpr uint64_t coset_generators_2[8]{
        0x6ULL, 0x7ULL, 0x8ULL, 0xaULL, 0xbULL, 0xcULL, 0xdULL, 0xfULL,
    };
    static constexpr uint64_t coset_generators_3[8]{
        0, 0, 0, 0, 0, 0, 0, 0,
    };

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