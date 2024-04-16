#pragma once

#include "../../fields/field.hpp"
#include "../../groups/group.hpp"

namespace bb::secp256r1 {
// NOLINTBEGIN(cppcoreguidelines-avoid-c-arrays)
struct FqParams {
    static constexpr uint64_t modulus_0 = 0xFFFFFFFFFFFFFFFFULL;
    static constexpr uint64_t modulus_1 = 0x00000000FFFFFFFFULL;
    static constexpr uint64_t modulus_2 = 0x0000000000000000ULL;
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

    static constexpr uint64_t modulus_wasm_0 = 0x1fffffff;
    static constexpr uint64_t modulus_wasm_1 = 0x1fffffff;
    static constexpr uint64_t modulus_wasm_2 = 0x1fffffff;
    static constexpr uint64_t modulus_wasm_3 = 0x1ff;
    static constexpr uint64_t modulus_wasm_4 = 0x0;
    static constexpr uint64_t modulus_wasm_5 = 0x0;
    static constexpr uint64_t modulus_wasm_6 = 0x40000;
    static constexpr uint64_t modulus_wasm_7 = 0x1fe00000;
    static constexpr uint64_t modulus_wasm_8 = 0xffffff;

    static constexpr uint64_t r_squared_wasm_0 = 0x0000000000000c00UL;
    static constexpr uint64_t r_squared_wasm_1 = 0xffffeffffffffc00UL;
    static constexpr uint64_t r_squared_wasm_2 = 0xfffffffffffffbffUL;
    static constexpr uint64_t r_squared_wasm_3 = 0x000013fffffff7ffUL;

    static constexpr uint64_t cube_root_wasm_0 = 0x0000000000000000UL;
    static constexpr uint64_t cube_root_wasm_1 = 0x0000000000000000UL;
    static constexpr uint64_t cube_root_wasm_2 = 0x0000000000000000UL;
    static constexpr uint64_t cube_root_wasm_3 = 0x0000000000000000UL;

    static constexpr uint64_t primitive_root_wasm_0 = 0x0000000000000000UL;
    static constexpr uint64_t primitive_root_wasm_1 = 0x0000000000000000UL;
    static constexpr uint64_t primitive_root_wasm_2 = 0x0000000000000000UL;
    static constexpr uint64_t primitive_root_wasm_3 = 0x0000000000000000UL;

    static constexpr uint64_t coset_generators_wasm_0[8] = { 0x0000000000000060ULL, 0x0000000000000080ULL,
                                                             0x00000000000000a0ULL, 0x00000000000000c0ULL,
                                                             0x00000000000000e0ULL, 0x0000000000000100ULL,
                                                             0x0000000000000120ULL, 0x0000000000000140ULL };
    static constexpr uint64_t coset_generators_wasm_1[8] = { 0xffffffa000000000ULL, 0xffffff8000000000ULL,
                                                             0xffffff6000000000ULL, 0xffffff4000000000ULL,
                                                             0xffffff2000000000ULL, 0xffffff0000000000ULL,
                                                             0xfffffee000000000ULL, 0xfffffec000000000ULL };
    static constexpr uint64_t coset_generators_wasm_2[8] = { 0xffffffffffffffffULL, 0xffffffffffffffffULL,
                                                             0xffffffffffffffffULL, 0xffffffffffffffffULL,
                                                             0xffffffffffffffffULL, 0xffffffffffffffffULL,
                                                             0xffffffffffffffffULL, 0xffffffffffffffffULL };
    static constexpr uint64_t coset_generators_wasm_3[8] = { 0x0000005fffffff9fULL, 0x0000007fffffff7fULL,
                                                             0x0000009fffffff5fULL, 0x000000bfffffff3fULL,
                                                             0x000000dfffffff1fULL, 0x000000fffffffeffULL,
                                                             0x0000011ffffffedfULL, 0x0000013ffffffebfULL };
};
using fq = field<FqParams>;

struct FrParams {
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

    static constexpr uint64_t modulus_wasm_0 = 0x1c632551;
    static constexpr uint64_t modulus_wasm_1 = 0x1dce5617;
    static constexpr uint64_t modulus_wasm_2 = 0x5e7a13c;
    static constexpr uint64_t modulus_wasm_3 = 0xdf55b4e;
    static constexpr uint64_t modulus_wasm_4 = 0x1ffffbce;
    static constexpr uint64_t modulus_wasm_5 = 0x1fffffff;
    static constexpr uint64_t modulus_wasm_6 = 0x3ffff;
    static constexpr uint64_t modulus_wasm_7 = 0x1fe00000;
    static constexpr uint64_t modulus_wasm_8 = 0xffffff;

    static constexpr uint64_t r_squared_wasm_0 = 0x45e9cfeeb48d9ef5UL;
    static constexpr uint64_t r_squared_wasm_1 = 0x1f11fc5bb2d31a99UL;
    static constexpr uint64_t r_squared_wasm_2 = 0x16c8e4adafb16586UL;
    static constexpr uint64_t r_squared_wasm_3 = 0x84b6556a65587f06UL;

    static constexpr uint64_t cube_root_wasm_0 = 0x0000000000000000UL;
    static constexpr uint64_t cube_root_wasm_1 = 0x0000000000000000UL;
    static constexpr uint64_t cube_root_wasm_2 = 0x0000000000000000UL;
    static constexpr uint64_t cube_root_wasm_3 = 0x0000000000000000UL;

    static constexpr uint64_t primitive_root_wasm_0 = 0x0000000000000000UL;
    static constexpr uint64_t primitive_root_wasm_1 = 0x0000000000000000UL;
    static constexpr uint64_t primitive_root_wasm_2 = 0x0000000000000000UL;
    static constexpr uint64_t primitive_root_wasm_3 = 0x0000000000000000UL;

    static constexpr uint64_t coset_generators_wasm_0[8] = { 0xbd6e9563293f5920ULL, 0x46353d039cdaaf00ULL,
                                                             0xcefbe4a4107604e0ULL, 0x57c28c4484115ac0ULL,
                                                             0xe08933e4f7acb0a0ULL, 0x694fdb856b480680ULL,
                                                             0xf2168325dee35c60ULL, 0x7add2ac6527eb240ULL };
    static constexpr uint64_t coset_generators_wasm_1[8] = { 0xb5e4a80dcb554baaULL, 0x19055258e8617b0cULL,
                                                             0x7c25fca4056daa6dULL, 0xdf46a6ef2279d9cfULL,
                                                             0x4267513a3f860930ULL, 0xa587fb855c923892ULL,
                                                             0x08a8a5d0799e67f3ULL, 0x6bc9501b96aa9755ULL };
    static constexpr uint64_t coset_generators_wasm_2[8] = { 0x000000000000003aULL, 0x0000000000000043ULL,
                                                             0x000000000000004bULL, 0x0000000000000053ULL,
                                                             0x000000000000005cULL, 0x0000000000000064ULL,
                                                             0x000000000000006dULL, 0x0000000000000075ULL };
    static constexpr uint64_t coset_generators_wasm_3[8] = { 0x000000dfffffff20ULL, 0x000000ffffffff00ULL,
                                                             0x0000011ffffffee0ULL, 0x0000013ffffffec0ULL,
                                                             0x0000015ffffffea0ULL, 0x0000017ffffffe80ULL,
                                                             0x0000019ffffffe60ULL, 0x000001bffffffe40ULL };
};
using fr = field<FrParams>;

struct G1Params {
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
using g1 = group<fq, fr, G1Params>;
} // namespace bb::secp256r1

namespace bb::curve {
class SECP256R1 {
  public:
    using ScalarField = secp256r1::fr;
    using BaseField = secp256r1::fq;
    using Group = secp256r1::g1;
    using Element = typename Group::element;
    using AffineElement = typename Group::affine_element;
};
} // namespace bb::curve

// NOLINTEND(cppcoreguidelines-avoid-c-arrays)
