#pragma once

#include "../../fields/field.hpp"
#include "../../groups/group.hpp"
#include "../types.hpp"

// NOLINTBEGIN(cppcoreguidelines-avoid-c-arrays)

namespace bb::secp256k1 {
struct FqParams {
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

    static constexpr uint64_t r_inv = 15580212934572586289ULL;

    static constexpr uint64_t cube_root_0 = 0x58a4361c8e81894eULL;
    static constexpr uint64_t cube_root_1 = 0x03fde1631c4b80afULL;
    static constexpr uint64_t cube_root_2 = 0xf8e98978d02e3905ULL;
    static constexpr uint64_t cube_root_3 = 0x7a4a36aebcbb3d53ULL;

    static constexpr uint64_t primitive_root_0 = 0UL;
    static constexpr uint64_t primitive_root_1 = 0UL;
    static constexpr uint64_t primitive_root_2 = 0UL;
    static constexpr uint64_t primitive_root_3 = 0UL;

    static constexpr uint64_t modulus_wasm_0 = 0x1ffffc2f;
    static constexpr uint64_t modulus_wasm_1 = 0x1ffffff7;
    static constexpr uint64_t modulus_wasm_2 = 0x1fffffff;
    static constexpr uint64_t modulus_wasm_3 = 0x1fffffff;
    static constexpr uint64_t modulus_wasm_4 = 0x1fffffff;
    static constexpr uint64_t modulus_wasm_5 = 0x1fffffff;
    static constexpr uint64_t modulus_wasm_6 = 0x1fffffff;
    static constexpr uint64_t modulus_wasm_7 = 0x1fffffff;
    static constexpr uint64_t modulus_wasm_8 = 0xffffff;

    static constexpr uint64_t r_squared_wasm_0 = 0x001e88003a428400UL;
    static constexpr uint64_t r_squared_wasm_1 = 0x0000000000000400UL;
    static constexpr uint64_t r_squared_wasm_2 = 0x0000000000000000UL;
    static constexpr uint64_t r_squared_wasm_3 = 0x0000000000000000UL;

    static constexpr uint64_t cube_root_wasm_0 = 0x1486c3a0d03162ffUL;
    static constexpr uint64_t cube_root_wasm_1 = 0x7fbc2c63897015ebUL;
    static constexpr uint64_t cube_root_wasm_2 = 0x1d312f1a05c720a0UL;
    static constexpr uint64_t cube_root_wasm_3 = 0x4946d5d79767aa7fUL;

    static constexpr uint64_t primitive_root_wasm_0 = 0x0000000000000000UL;
    static constexpr uint64_t primitive_root_wasm_1 = 0x0000000000000000UL;
    static constexpr uint64_t primitive_root_wasm_2 = 0x0000000000000000UL;
    static constexpr uint64_t primitive_root_wasm_3 = 0x0000000000000000UL;

    static constexpr uint64_t coset_generators_wasm_0[8] = { 0x0000006000016e60ULL, 0x000000800001e880ULL,
                                                             0x000000a0000262a0ULL, 0x000000c00002dcc0ULL,
                                                             0x000000e0000356e0ULL, 0x000001000003d100ULL,
                                                             0x0000012000044b20ULL, 0x000001400004c540ULL };
    static constexpr uint64_t coset_generators_wasm_1[8] = { 0x0000000000000000ULL, 0x0000000000000000ULL,
                                                             0x0000000000000000ULL, 0x0000000000000000ULL,
                                                             0x0000000000000000ULL, 0x0000000000000000ULL,
                                                             0x0000000000000000ULL, 0x0000000000000000ULL };
    static constexpr uint64_t coset_generators_wasm_2[8] = { 0x0000000000000000ULL, 0x0000000000000000ULL,
                                                             0x0000000000000000ULL, 0x0000000000000000ULL,
                                                             0x0000000000000000ULL, 0x0000000000000000ULL,
                                                             0x0000000000000000ULL, 0x0000000000000000ULL };
    static constexpr uint64_t coset_generators_wasm_3[8] = { 0x0000000000000000ULL, 0x0000000000000000ULL,
                                                             0x0000000000000000ULL, 0x0000000000000000ULL,
                                                             0x0000000000000000ULL, 0x0000000000000000ULL,
                                                             0x0000000000000000ULL, 0x0000000000000000ULL };
};
using fq = field<FqParams>;

struct FrParams {
    static constexpr uint64_t modulus_0 = 0xBFD25E8CD0364141ULL;
    static constexpr uint64_t modulus_1 = 0xBAAEDCE6AF48A03BULL;
    static constexpr uint64_t modulus_2 = 0xFFFFFFFFFFFFFFFEULL;
    static constexpr uint64_t modulus_3 = 0xFFFFFFFFFFFFFFFFULL;

    static constexpr uint64_t r_squared_0 = 9902555850136342848ULL;
    static constexpr uint64_t r_squared_1 = 8364476168144746616ULL;
    static constexpr uint64_t r_squared_2 = 16616019711348246470ULL;
    static constexpr uint64_t r_squared_3 = 11342065889886772165ULL;

    static constexpr uint64_t r_inv = 5408259542528602431ULL;

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

    static constexpr uint64_t cube_root_0 = 0xf07deb3dc9926c9eULL;
    static constexpr uint64_t cube_root_1 = 0x2c93e7ad83c6944cULL;
    static constexpr uint64_t cube_root_2 = 0x73a9660652697d91ULL;
    static constexpr uint64_t cube_root_3 = 0x532840178558d639ULL;

    static constexpr uint64_t endo_minus_b1_lo = 0x6F547FA90ABFE4C3ULL;
    static constexpr uint64_t endo_minus_b1_mid = 0xE4437ED6010E8828ULL;

    static constexpr uint64_t endo_b2_lo = 0xe86c90e49284eb15ULL;
    static constexpr uint64_t endo_b2_mid = 0x3086d221a7d46bcdULL;

    static constexpr uint64_t endo_g1_lo = 0xE893209A45DBB031ULL;
    static constexpr uint64_t endo_g1_mid = 0x3DAA8A1471E8CA7FULL;
    static constexpr uint64_t endo_g1_hi = 0xE86C90E49284EB15ULL;
    static constexpr uint64_t endo_g1_hihi = 0x3086D221A7D46BCDULL;

    static constexpr uint64_t endo_g2_lo = 0x1571B4AE8AC47F71ULL;
    static constexpr uint64_t endo_g2_mid = 0x221208AC9DF506C6ULL;
    static constexpr uint64_t endo_g2_hi = 0x6F547FA90ABFE4C4ULL;
    static constexpr uint64_t endo_g2_hihi = 0xE4437ED6010E8828ULL;

    static constexpr uint64_t primitive_root_0 = 0UL;
    static constexpr uint64_t primitive_root_1 = 0UL;
    static constexpr uint64_t primitive_root_2 = 0UL;
    static constexpr uint64_t primitive_root_3 = 0UL;

    static constexpr uint64_t modulus_wasm_0 = 0x10364141;
    static constexpr uint64_t modulus_wasm_1 = 0x1e92f466;
    static constexpr uint64_t modulus_wasm_2 = 0x12280eef;
    static constexpr uint64_t modulus_wasm_3 = 0x1db9cd5e;
    static constexpr uint64_t modulus_wasm_4 = 0x1fffebaa;
    static constexpr uint64_t modulus_wasm_5 = 0x1fffffff;
    static constexpr uint64_t modulus_wasm_6 = 0x1fffffff;
    static constexpr uint64_t modulus_wasm_7 = 0x1fffffff;
    static constexpr uint64_t modulus_wasm_8 = 0xffffff;

    static constexpr uint64_t r_squared_wasm_0 = 0x63e601a3c9f6ab4bUL;
    static constexpr uint64_t r_squared_wasm_1 = 0xa2b6456d46702f57UL;
    static constexpr uint64_t r_squared_wasm_2 = 0x5fd7916f341f1cefUL;
    static constexpr uint64_t r_squared_wasm_3 = 0x9c7356071a6f179aUL;

    static constexpr uint64_t cube_root_wasm_0 = 0x9185b639102f0736UL;
    static constexpr uint64_t cube_root_wasm_1 = 0x47a854ad9ffc4748UL;
    static constexpr uint64_t cube_root_wasm_2 = 0x752cc0ca4d2fb232UL;
    static constexpr uint64_t cube_root_wasm_3 = 0x650802f0ab1ac72eUL;

    static constexpr uint64_t primitive_root_wasm_0 = 0x0000000000000000UL;
    static constexpr uint64_t primitive_root_wasm_1 = 0x0000000000000000UL;
    static constexpr uint64_t primitive_root_wasm_2 = 0x0000000000000000UL;
    static constexpr uint64_t primitive_root_wasm_3 = 0x0000000000000000UL;

    static constexpr uint64_t coset_generators_wasm_0[8] = { 0x1c84e7fdde173760ULL, 0x22391663d74f0f40ULL,
                                                             0x27ed44c9d086e720ULL, 0x2da1732fc9bebf00ULL,
                                                             0x3355a195c2f696e0ULL, 0x3909cffbbc2e6ec0ULL,
                                                             0x3ebdfe61b56646a0ULL, 0x44722cc7ae9e1e80ULL };
    static constexpr uint64_t coset_generators_wasm_1[8] = { 0x52b5efd2729bdaa8ULL, 0xfcda52fc8987d330ULL,
                                                             0xa6feb626a073cbb8ULL, 0x51231950b75fc440ULL,
                                                             0xfb477c7ace4bbcc8ULL, 0xa56bdfa4e537b550ULL,
                                                             0x4f9042cefc23add8ULL, 0xf9b4a5f9130fa660ULL };
    static constexpr uint64_t coset_generators_wasm_2[8] = { 0x00000000000000cbULL, 0x00000000000000f3ULL,
                                                             0x000000000000011cULL, 0x0000000000000145ULL,
                                                             0x000000000000016dULL, 0x0000000000000196ULL,
                                                             0x00000000000001bfULL, 0x00000000000001e7ULL };
    static constexpr uint64_t coset_generators_wasm_3[8] = { 0x0000000000000000ULL, 0x0000000000000000ULL,
                                                             0x0000000000000000ULL, 0x0000000000000000ULL,
                                                             0x0000000000000000ULL, 0x0000000000000000ULL,
                                                             0x0000000000000000ULL, 0x0000000000000000ULL };
};
using fr = field<FrParams>;

struct G1Params {
    static constexpr bool USE_ENDOMORPHISM = false;
    static constexpr bool can_hash_to_curve = true;
    static constexpr bool small_elements = true;
    static constexpr bool has_a = false;

    static constexpr fq b = fq(7);
    static constexpr fq a = fq(0);

    static constexpr fq one_x =
        fq(0x59F2815B16F81798UL, 0x029BFCDB2DCE28D9UL, 0x55A06295CE870B07UL, 0x79BE667EF9DCBBACUL).to_montgomery_form();
    static constexpr fq one_y =
        fq(0x9C47D08FFB10D4B8UL, 0xFD17B448A6855419UL, 0x5DA4FBFC0E1108A8UL, 0x483ADA7726A3C465UL).to_montgomery_form();
};
using g1 = group<fq, fr, G1Params>;
} // namespace bb::secp256k1

namespace bb::curve {
class SECP256K1 {
  public:
    using ScalarField = secp256k1::fr;
    using BaseField = secp256k1::fq;
    using Group = secp256k1::g1;
    using Element = typename Group::element;
    using AffineElement = typename Group::affine_element;
};
} // namespace bb::curve

// NOLINTEND(cppcoreguidelines-avoid-c-arrays)
