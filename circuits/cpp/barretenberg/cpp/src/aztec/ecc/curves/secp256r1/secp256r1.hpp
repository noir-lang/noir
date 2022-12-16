#pragma once

#include <numeric/uint256/uint256.hpp>
#include <numeric/uintx/uintx.hpp>

#include "../../fields/field.hpp"
#include "../../groups/group.hpp"
#include "../bn254/fq.hpp"
#include "../bn254/fr.hpp"

namespace secp256r1 {

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

struct Secp256r1FqParams {
    static constexpr uint64_t modulus_0 = 0xFFFFFFFFFFFFFFFFULL;
    static constexpr uint64_t modulus_1 = 0x00000000FFFFFFFFULL;
    static constexpr uint64_t modulus_2 = 0X0000000000000000ULL;
    static constexpr uint64_t modulus_3 = 0xFFFFFFFF00000001ULL;

    static constexpr uint64_t r_squared_0 = 3ULL;
    static constexpr uint64_t r_squared_1 = 18446744056529682431ULL;
    static constexpr uint64_t r_squared_2 = 18446744073709551614ULL;
    static constexpr uint64_t r_squared_3 = 21474836477ULL;

    static constexpr uint64_t r_inv = 1;

    static constexpr uint64_t coset_generators_0[8]{
        0x3ULL, 0x4ULL, 0x5ULL, 0x6ULL, 0x7ULL, 0x8ULL, 0x9ULL, 0xaULL,
    };
    static constexpr uint64_t coset_generators_1[8]{
        0xfffffffd00000000ULL, 0xfffffffc00000000ULL, 0xfffffffb00000000ULL, 0xfffffffa00000000ULL,
        0xfffffff900000000ULL, 0xfffffff800000000ULL, 0xfffffff700000000ULL, 0xfffffff600000000ULL,
    };
    static constexpr uint64_t coset_generators_2[8]{
        0xffffffffffffffffULL, 0xffffffffffffffffULL, 0xffffffffffffffffULL, 0xffffffffffffffffULL,
        0xffffffffffffffffULL, 0xffffffffffffffffULL, 0xffffffffffffffffULL, 0xffffffffffffffffULL,
    };
    static constexpr uint64_t coset_generators_3[8]{
        0x2fffffffcULL, 0x3fffffffbULL, 0x4fffffffaULL, 0x5fffffff9ULL,
        0x6fffffff8ULL, 0x7fffffff7ULL, 0x8fffffff6ULL, 0x9fffffff5ULL,
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

struct Secp256r1FrParams {
    static constexpr uint64_t modulus_0 = 0xF3B9CAC2FC632551ULL;
    static constexpr uint64_t modulus_1 = 0xBCE6FAADA7179E84ULL;
    static constexpr uint64_t modulus_2 = 0xFFFFFFFFFFFFFFFFULL;
    static constexpr uint64_t modulus_3 = 0xFFFFFFFF00000000ULL;

    static constexpr uint64_t r_squared_0 = 9449762124159643298ULL;
    static constexpr uint64_t r_squared_1 = 5087230966250696614ULL;
    static constexpr uint64_t r_squared_2 = 2901921493521525849ULL;
    static constexpr uint64_t r_squared_3 = 7413256579398063648ULL;

    static constexpr uint64_t r_inv = 14758798090332847183ULL;

    static constexpr uint64_t coset_generators_0[8]{
        0x55eb74ab1949fac9ULL, 0x6231a9e81ce6d578ULL, 0x6e77df252083b027ULL, 0x7abe146224208ad6ULL,
        0x8704499f27bd6585ULL, 0x934a7edc2b5a4034ULL, 0x9f90b4192ef71ae3ULL, 0xabd6e9563293f592ULL,
    };
    static constexpr uint64_t coset_generators_1[8]{
        0xd5af25406e5aaa5dULL, 0x18c82a92c7430bd8ULL, 0x5be12fe5202b6d53ULL, 0x9efa35377913ceceULL,
        0xe2133a89d1fc3049ULL, 0x252c3fdc2ae491c4ULL, 0x6845452e83ccf33fULL, 0xab5e4a80dcb554baULL,
    };
    static constexpr uint64_t coset_generators_2[8]{
        0x1ULL, 0x2ULL, 0x2ULL, 0x2ULL, 0x2ULL, 0x3ULL, 0x3ULL, 0x3ULL,
    };
    static constexpr uint64_t coset_generators_3[8]{
        0x6fffffff9ULL, 0x7fffffff8ULL, 0x8fffffff7ULL, 0x9fffffff6ULL,
        0xafffffff5ULL, 0xbfffffff4ULL, 0xcfffffff3ULL, 0xdfffffff2ULL,
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

typedef barretenberg::field<Secp256r1FqParams> fq;
typedef barretenberg::field<Secp256r1FrParams> fr;

struct Secp256r1G1Params {
    static constexpr bool USE_ENDOMORPHISM = false;
    static constexpr bool can_hash_to_curve = true;
    static constexpr bool small_elements = true;
    static constexpr bool has_a = true;

    static constexpr fq b =
        fq(0x3BCE3C3E27D2604B, 0x651D06B0CC53B0F6, 0xB3EBBD55769886BC, 0x5AC635D8AA3A93E7).to_montgomery_form();
    static constexpr fq a =
        fq(0xFFFFFFFFFFFFFFFC, 0x00000000FFFFFFFF, 0x0000000000000000, 0xFFFFFFFF00000001).to_montgomery_form();

    static constexpr fq one_x =
        fq(0xF4A13945D898C296, 0x77037D812DEB33A0, 0xF8BCE6E563A440F2, 0x6B17D1F2E12C4247).to_montgomery_form();
    static constexpr fq one_y =
        fq(0xCBB6406837BF51F5, 0x2BCE33576B315ECE, 0x8EE7EB4A7C0F9E16, 0x4FE342E2FE1A7F9B).to_montgomery_form();
};

typedef barretenberg::
    group<barretenberg::field<Secp256r1FqParams>, barretenberg::field<Secp256r1FrParams>, Secp256r1G1Params>
        g1;
g1::affine_element get_generator(const size_t generator_index);
} // namespace secp256r1