#include "pairing.hpp"
#include <gtest/gtest.h>

using namespace barretenberg;

TEST(pairing, reduced_ate_pairing_check_against_constants)
{
    constexpr g1::affine_element P = {
        uint256_t(0x956e256b9db00c13, 0x66d29ac18e1b2bff, 0x5d6f055e34402f6e, 0x5bfcbaaff0feb62),
        uint256_t(0x564099dc0ef0a96, 0xa97eca7453f67dd2, 0x850e976b207e8c18, 0x20187f89a1d789cd)
    };
    constexpr g2::affine_element Q = {
        { uint256_t(0x3b25f1ad9a7f9cd2, 0xddb8b066d21ce86, 0xf8a4e318abd3cff7, 0x1272ee5f2e7e9dc1),
          uint256_t(0xc7b14ea54dc1436f, 0x1f9384eb12b6941a, 0x3afe17a00720e8e3, 0x2a171f424ab98d8) },
        { uint256_t(0x890d5a50c1d88e96, 0x6ae79a7a2b439172, 0x4c120a629ced363c, 0x295bd556fe685dd),
          uint256_t(0xa3189c7f120d4738, 0x4416da0df17c8ee, 0x4cc514acc1c2ac45, 0xb17d8f998e4ebe6) }
    };
    constexpr fq12 expected = {

        { { uint256_t(0xd3b91c8dc40a9b8c, 0x5c8a39a470fcb4ea, 0x763e904e585a87e7, 0x2026f0077c50afa4),
            uint256_t(0xddc69495371e5f38, 0x290bfc6512704e60, 0xc208c0f8e90bd52f, 0x2e82c92370a2f000) },
          { uint256_t(0xdcbc2917451b8e12, 0x183016aa113a74eb, 0x9a2ff2a059f7d14d, 0x1166fc0ed488820c),
            uint256_t(0x3b2c1e19e47214ff, 0x374df83e0ac59c1a, 0x3e1c5ed4fd611cb2, 0x26179258a104da1a) },
          { uint256_t(0xc948bdff07912922, 0x3417ba2a42303918, 0x89336b54f20ff8a9, 0xb7eed88572fcac4),

            uint256_t(0x85524385a79574ba, 0xe7746ad78e659d8e, 0x997e4848cc70eca5, 0x2a9e3f37c50e6c9a) } },

        { { uint256_t(0xc7eed1ca5aaa5a82, 0xea8d1f0be1ef0d7, 0xd7d539fd8136038a, 0x27196e24cd6d028e),
            uint256_t(0xcb7b6528984002e4, 0x1d3221c223e0587, 0xda44f3e957677f97, 0x1e3df34445cc3876) },
          { uint256_t(0xf3e958491c2b4c43, 0x1dbafe473f7034b9, 0x129efae93ff9d8c9, 0xdedbf49d35171b9),
            uint256_t(0x7da7c99cf811a603, 0xfcb99b8309663279, 0x1d80151ef8fcdb59, 0x1b09a01856170269) },
          { uint256_t(0xa048b10941003960, 0x73d941c906a24cd0, 0x9c10f82a6bf78e2e, 0x13a41dbdd3d616d),
            uint256_t(0x31d7525fa8914a4c, 0xe1ed738718e2e8b8, 0x18305c749a9d97a2, 0x20534d878e1e9db0) } }
    };

    constexpr fq12 result = pairing::reduced_ate_pairing(P, Q);

    static_assert(result == expected); // test to see if compiler can evaluate bilinear pairing at compile time
    EXPECT_EQ(result, expected);
}

TEST(pairing, reduced_ate_pairing_consistency_check)
{
    g1::affine_element P = g1::affine_element(g1::element::random_element());
    g2::affine_element Q = g2::affine_element(g2::element::random_element());

    fr scalar = fr::random_element();

    g1::affine_element Pmul = P * scalar;
    g2::affine_element Qmul = Q * scalar;

    fq12 result = pairing::reduced_ate_pairing(Pmul, Q).from_montgomery_form();
    fq12 expected = pairing::reduced_ate_pairing(P, Qmul).from_montgomery_form();

    EXPECT_EQ(result, expected);
}

TEST(pairing, reduced_ate_pairing_consistency_check_batch)
{
    size_t num_points = 10;

    g1::affine_element P_a[num_points];
    g2::affine_element Q_a[num_points];

    g1::affine_element P_b[num_points];
    g2::affine_element Q_b[num_points];

    fr scalars[num_points + num_points];
    for (size_t i = 0; i < 10; ++i) {
        scalars[i] = fr::random_element();
        scalars[i + num_points] = fr::random_element();
        g1::affine_element P = g1::affine_element(g1::element::random_element());
        g2::affine_element Q = g2::affine_element(g2::element::random_element());
        P_a[i] = P;
        Q_a[i] = Q;
        P_b[i] = P;
        Q_b[i] = Q;
    }

    for (size_t i = 0; i < 10; ++i) {
        P_a[i] = P_a[i] * scalars[i];
        Q_b[i] = Q_b[i] * scalars[i];
        P_b[i] = P_b[i] * scalars[i + num_points];
        Q_a[i] = Q_a[i] * scalars[i + num_points];
    }

    fq12 result = pairing::reduced_ate_pairing_batch(&P_a[0], &Q_a[0], num_points).from_montgomery_form();
    fq12 expected = pairing::reduced_ate_pairing_batch(&P_b[0], &Q_b[0], num_points).from_montgomery_form();

    EXPECT_EQ(result, expected);
}

TEST(pairing, reduced_ate_pairing_precompute_consistency_check_batch)
{
    size_t num_points = 10;
    g1::affine_element P_a[num_points];
    g2::affine_element Q_a[num_points];
    g1::affine_element P_b[num_points];
    g2::affine_element Q_b[num_points];
    pairing::miller_lines precompute_miller_lines[num_points];
    fr scalars[num_points + num_points];
    for (size_t i = 0; i < 10; ++i) {
        scalars[i] = fr::random_element();
        scalars[i + num_points] = fr::random_element();
        g1::affine_element P = g1::affine_element(g1::element::random_element());
        g2::affine_element Q = g2::affine_element(g2::element::random_element());
        P_a[i] = P;
        Q_a[i] = Q;
        P_b[i] = P;
        Q_b[i] = Q;
    }
    for (size_t i = 0; i < 10; ++i) {
        P_a[i] = P_a[i] * scalars[i];
        Q_b[i] = Q_b[i] * scalars[i];
        P_b[i] = P_b[i] * scalars[i + num_points];
        Q_a[i] = Q_a[i] * scalars[i + num_points];
    }
    for (size_t i = 0; i < 10; ++i) {
        g2::element jac;
        jac = g2::element(Q_a[i]);
        pairing::precompute_miller_lines(jac, precompute_miller_lines[i]);
    }
    fq12 result = pairing::reduced_ate_pairing_batch_precomputed(&P_a[0], &precompute_miller_lines[0], num_points)
                      .from_montgomery_form();
    fq12 expected = pairing::reduced_ate_pairing_batch(&P_b[0], &Q_b[0], num_points).from_montgomery_form();

    EXPECT_EQ(result, expected);
}