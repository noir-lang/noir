#include "barretenberg/flavor/flavor.hpp"
#include "barretenberg/relations/poseidon2_external_relation.hpp"
#include "barretenberg/relations/poseidon2_internal_relation.hpp"
#include "barretenberg/relations/relation_parameters.hpp"
#include <gtest/gtest.h>

using namespace bb;

using FF = fr;

class RelationManual : public testing::Test {};

TEST_F(RelationManual, Poseidon2ExternalRelationZeros)
{
    using Accumulator = std::array<FF, 4>;
    using Relation = bb::Poseidon2ExternalRelation<FF>;

    Accumulator acc{ 0, 0, 0, 0 };
    struct AllPoseidonValues {
        FF q_poseidon2_external;
        FF w_l;
        FF w_r;
        FF w_o;
        FF w_4;
        FF w_l_shift;
        FF w_r_shift;
        FF w_o_shift;
        FF w_4_shift;
        FF q_l;
        FF q_r;
        FF q_o;
        FF q_4;
    };
    AllPoseidonValues all_poseidon_values{ 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0 };

    const auto parameters = RelationParameters<FF>::get_random();
    Relation::accumulate(acc, all_poseidon_values, parameters, 1);
    EXPECT_EQ(acc[0], 0);
    EXPECT_EQ(acc[1], 0);
    EXPECT_EQ(acc[2], 0);
    EXPECT_EQ(acc[3], 0);
}

TEST_F(RelationManual, Poseidon2ExternalRelationRandom)
{
    using Accumulator = std::array<FF, 4>;
    using Relation = bb::Poseidon2ExternalRelation<FF>;

    Accumulator acc{ 0, 0, 0, 0 };
    struct AllPoseidonValues {
        FF q_poseidon2_external;
        FF w_l;
        FF w_r;
        FF w_o;
        FF w_4;
        FF q_l;
        FF q_r;
        FF q_o;
        FF q_4;
        FF w_l_shift;
        FF w_r_shift;
        FF w_o_shift;
        FF w_4_shift;
    };
    /*
     * v1 = w_1 + q_1 = 5 + 6 = 11
     * v2 = w_2 + q_2 = 4 + 9 = 13
     * v3 = w_3 + q_3 = 1 + 8 = 9
     * v4 = w_4 + q_4 = 7 + 3 = 10
     * u1 = v1^5 = 11^5 = 161051
     * u2 = v2^5 = 13^5 = 371293
     * u3 = v3^5 = 9^5 = 59049
     * u4 = v4^5 = 10^5 = 100000
     * matrix mul with calculator:
     * 1	3763355
     * 2	3031011
     * 3	2270175
     * 4	1368540
     */
    AllPoseidonValues all_poseidon_values{ 1, 5, 4, 1, 7, 6, 9, 8, 3, 3763355, 3031011, 2270175, 1368540 };

    const auto parameters = RelationParameters<FF>::get_random();
    Relation::accumulate(acc, all_poseidon_values, parameters, 1);
    EXPECT_EQ(acc[0], 0);
    EXPECT_EQ(acc[1], 0);
    EXPECT_EQ(acc[2], 0);
    EXPECT_EQ(acc[3], 0);
}

TEST_F(RelationManual, Poseidon2InternalRelationZeros)
{
    using Accumulator = std::array<FF, 4>;
    using Relation = bb::Poseidon2InternalRelation<FF>;

    Accumulator acc{ 0, 0, 0, 0 };
    struct AllPoseidonValues {
        FF q_poseidon2_internal;
        FF w_l;
        FF w_r;
        FF w_o;
        FF w_4;
        FF w_l_shift;
        FF w_r_shift;
        FF w_o_shift;
        FF w_4_shift;
        FF q_l;
        FF q_r;
        FF q_o;
        FF q_4;
    };
    AllPoseidonValues all_poseidon_values{ 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0 };

    const auto parameters = RelationParameters<FF>::get_random();
    Relation::accumulate(acc, all_poseidon_values, parameters, 1);
    EXPECT_EQ(acc[0], 0);
    EXPECT_EQ(acc[1], 0);
    EXPECT_EQ(acc[2], 0);
    EXPECT_EQ(acc[3], 0);
}

TEST_F(RelationManual, Poseidon2InternalRelationRandom)
{
    using Accumulator = std::array<FF, 4>;
    using Relation = bb::Poseidon2InternalRelation<FF>;

    Accumulator acc{ 0, 0, 0, 0 };
    struct AllPoseidonValues {
        FF q_poseidon2_internal;
        FF w_l;
        FF w_r;
        FF w_o;
        FF w_4;
        FF q_l;

        FF w_l_shift;
        FF w_r_shift;
        FF w_o_shift;
        FF w_4_shift;
    };
    /*
     * u1 = (w_1 + q_1)^5 = (1 + 5)^5 = 7776
     * sum = u1 + w_2 + w_3 + w_4 = 7776 + 2 + 3 + 4 = 7785
     * matrix mul with calculator:
     * 1    0x122d9ce41e83c533318954d77a4ebc40eb729f6543ebd5f2e4ecb175ced3bc74
     * 2	0x185028b6d489be7c029367a14616776b33bf2eada9bb370950d6719f68b5067f
     * 3	0x00fce289a96b3f4a18562d0ef0ab76ca165e613222aa0c24501377003c5622a8
     * 4	0x27e7677799fda1694819803f459b76d2fb1c45fdf0773375c72d61e8efb92893
     */
    AllPoseidonValues all_poseidon_values{
        1,
        1,
        2,
        3,
        4,
        5,
        FF(std::string("0x122d9ce41e83c533318954d77a4ebc40eb729f6543ebd5f2e4ecb175ced3bc74")),
        FF(std::string("0x185028b6d489be7c029367a14616776b33bf2eada9bb370950d6719f68b5067f")),
        FF(std::string("0x00fce289a96b3f4a18562d0ef0ab76ca165e613222aa0c24501377003c5622a8")),
        FF(std::string("0x27e7677799fda1694819803f459b76d2fb1c45fdf0773375c72d61e8efb92893"))
    };
    const auto parameters = RelationParameters<FF>::get_random();
    Relation::accumulate(acc, all_poseidon_values, parameters, 1);
    EXPECT_EQ(acc[0], 0);
    EXPECT_EQ(acc[1], 0);
    EXPECT_EQ(acc[2], 0);
    EXPECT_EQ(acc[3], 0);
}