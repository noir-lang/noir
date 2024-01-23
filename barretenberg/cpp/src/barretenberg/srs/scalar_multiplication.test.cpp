/**
 * @file scalar_multiplication.test.cpp
 * @brief Tests of our implementation of Pippenger's multi-scalar multiplication algorithm.
 *
 * @details This file is here with the SRS code, rather than being next to the Pippenger implementation, to avoid a
 * cyclic dependency between our srs and ecc modules. Namely, srs depends on ecc via the FileProverCrs constructor that
 * constructs a Pippenger point table. It may make sense to create a function in the ecc module that initializes a CRS,
 * but for now a low-impact solution (to a newly-encountered linker error) is to move this test file, as it was the sole
 * reason for for ecc to depend on srs.
 */

#include "barretenberg/ecc/scalar_multiplication/scalar_multiplication.hpp"
#include "barretenberg/common/mem.hpp"
#include "barretenberg/common/test.hpp"
#include "barretenberg/ecc/scalar_multiplication/point_table.hpp"
#include "barretenberg/numeric/random/engine.hpp"
#include "barretenberg/srs/factories/file_crs_factory.hpp"
#include "barretenberg/srs/io.hpp"

#include <cstddef>
#include <vector>

using namespace bb;

namespace {
auto& engine = numeric::get_debug_randomness();
}

template <typename Curve> class ScalarMultiplicationTests : public ::testing::Test {
  public:
    const std::string SRS_PATH = []() {
        if constexpr (std::same_as<Curve, curve::BN254>) {
            return "../srs_db/ignition";
        } else if constexpr (std::same_as<Curve, curve::Grumpkin>) {
            return "../srs_db/grumpkin";
        }
    }();

    static void read_transcript_g2(std::string const& srs_path)
        requires srs::HasG2<Curve>
    {
        typename Curve::G2AffineElement g2_x;
        srs::IO<Curve>::read_transcript_g2(g2_x, srs_path);
    };

    static void read_transcript(typename Curve::AffineElement* monomials, size_t degree, std::string const& srs_path)
    {
        if constexpr (srs::HasG2<Curve>) {
            typename Curve::G2AffineElement g2_x;
            srs::IO<Curve>::read_transcript(monomials, g2_x, degree, srs_path);
        } else {
            srs::IO<Curve>::read_transcript(monomials, degree, srs_path);
        }
    }
};

using Curves = ::testing::Types<curve::BN254, curve::Grumpkin>;

TYPED_TEST_SUITE(ScalarMultiplicationTests, Curves);

TYPED_TEST(ScalarMultiplicationTests, ReduceBucketsSimple)
{
    using Curve = TypeParam;
    using Element = typename Curve::Element;
    using AffineElement = typename Curve::AffineElement;
    using Fq = typename Curve::BaseField;

    constexpr size_t num_points = 128;
    if constexpr (srs::HasG2<Curve>) {
        TestFixture::read_transcript_g2(TestFixture::SRS_PATH);
    }
    auto crs = srs::factories::FileProverCrs<Curve>(num_points / 2, TestFixture::SRS_PATH);
    auto monomials = crs.get_monomial_points();

    std::vector<uint64_t> point_schedule(bb::scalar_multiplication::point_table_size(num_points / 2));
    std::array<bool, num_points> bucket_empty_status;
    // 16 buckets, each bucket has one point
    std::array<uint64_t, num_points> transcript;
    std::array<uint64_t, num_points> transcript_points;
    transcript_points[0] = 0x0;
    transcript_points[1] = 0x2;
    transcript_points[2] = 0x4;
    transcript_points[3] = 0x6;
    transcript_points[4] = 0xb;
    transcript_points[5] = 0xc;
    transcript_points[6] = 0xe;
    transcript_points[7] = 0x11;
    transcript_points[8] = 0x13;
    transcript_points[9] = 0x14;
    transcript_points[10] = 0x15;
    transcript_points[11] = 0x16;
    transcript_points[12] = 0x17;
    transcript_points[13] = 0x18;
    transcript_points[14] = 0x20;
    transcript_points[15] = 0x21;
    transcript_points[16] = 0x22;
    transcript_points[17] = 0x27;
    transcript_points[18] = 0x29;
    transcript_points[19] = 0x2b;
    transcript_points[20] = 0x2c;
    transcript_points[21] = 0x2d;
    transcript_points[22] = 0x2e;
    transcript_points[23] = 0x36;
    transcript_points[24] = 0x37;
    transcript_points[25] = 0x38;
    transcript_points[26] = 0x3e;
    transcript_points[27] = 0x3f;
    transcript_points[28] = 0x4e;
    transcript_points[29] = 0x4f;
    transcript_points[30] = 0x50;
    transcript_points[31] = 0x51;
    transcript_points[32] = 0x41;
    transcript_points[33] = 0x52;
    transcript_points[34] = 0x53;
    transcript_points[35] = 0x54;
    transcript_points[36] = 0x43;
    transcript_points[37] = 0x57;
    transcript_points[38] = 0x46;
    transcript_points[39] = 0x58;
    transcript_points[40] = 0x5b;
    transcript_points[41] = 0x5e;
    transcript_points[42] = 0x42;
    transcript_points[43] = 0x47;
    transcript_points[44] = 0x4b;
    transcript_points[45] = 0x4d;
    transcript_points[46] = 0x6b;
    transcript_points[47] = 0x65;
    transcript_points[48] = 0x6d;
    transcript_points[49] = 0x67;
    transcript_points[50] = 0x6f;
    transcript_points[51] = 0x68;
    transcript_points[52] = 0x69;
    transcript_points[53] = 0x6a;
    transcript_points[54] = 0x71;
    transcript_points[55] = 0x72;
    transcript_points[56] = 0x73;
    transcript_points[57] = 0x74;
    transcript_points[58] = 0x75;
    transcript_points[59] = 0x66;
    transcript_points[60] = 0x79;
    transcript_points[62] = 0x7c;
    transcript_points[61] = 0x7e;
    transcript_points[63] = 0x7f;
    transcript_points[64] = 0x1;
    transcript_points[65] = 0x3;
    transcript_points[66] = 0x5;
    transcript_points[67] = 0x7;
    transcript_points[68] = 0x8;
    transcript_points[69] = 0x9;
    transcript_points[70] = 0xa;
    transcript_points[71] = 0xd;
    transcript_points[72] = 0xf;
    transcript_points[73] = 0x10;
    transcript_points[74] = 0x12;
    transcript_points[75] = 0x19;
    transcript_points[76] = 0x1a;
    transcript_points[77] = 0x1b;
    transcript_points[78] = 0x1c;
    transcript_points[79] = 0x1d;
    transcript_points[80] = 0x1e;
    transcript_points[81] = 0x1f;
    transcript_points[82] = 0x23;
    transcript_points[83] = 0x24;
    transcript_points[84] = 0x25;
    transcript_points[85] = 0x26;
    transcript_points[86] = 0x28;
    transcript_points[87] = 0x2a;
    transcript_points[88] = 0x2f;
    transcript_points[89] = 0x30;
    transcript_points[90] = 0x31;
    transcript_points[91] = 0x32;
    transcript_points[92] = 0x33;
    transcript_points[93] = 0x34;
    transcript_points[94] = 0x35;
    transcript_points[95] = 0x39;
    transcript_points[96] = 0x3a;
    transcript_points[97] = 0x3b;
    transcript_points[98] = 0x3c;
    transcript_points[99] = 0x3d;
    transcript_points[100] = 0x48;
    transcript_points[101] = 0x49;
    transcript_points[102] = 0x55;
    transcript_points[103] = 0x56;
    transcript_points[104] = 0x4a;
    transcript_points[105] = 0x44;
    transcript_points[106] = 0x45;
    transcript_points[107] = 0x40;
    transcript_points[108] = 0x59;
    transcript_points[109] = 0x5a;
    transcript_points[110] = 0x5c;
    transcript_points[111] = 0x5d;
    transcript_points[112] = 0x5f;
    transcript_points[113] = 0x60;
    transcript_points[114] = 0x61;
    transcript_points[115] = 0x62;
    transcript_points[116] = 0x63;
    transcript_points[117] = 0x4c;
    transcript_points[118] = 0x6c;
    transcript_points[119] = 0x6e;
    transcript_points[120] = 0x64;
    transcript_points[121] = 0x70;
    transcript_points[122] = 0x77;
    transcript_points[123] = 0x78;
    transcript_points[124] = 0x76;
    transcript_points[125] = 0x7a;
    transcript_points[126] = 0x7b;
    transcript_points[127] = 0x7d;

    for (size_t i = 0; i < 64; ++i) {
        transcript[i] = 0;
        transcript[i + 64] = 1;
    }
    for (size_t i = 0; i < num_points; ++i) {
        point_schedule[i] = (static_cast<uint64_t>(transcript_points[i]) << 32ULL) + transcript[i];
    }
    std::array<Element, num_points> expected;
    for (size_t i = 0; i < num_points; ++i) {
        expected[i].self_set_infinity();
    }

    for (size_t i = 0; i < num_points; ++i) {
        size_t schedule = transcript[i] & 0x7fffffffU;
        {
            expected[schedule] += monomials[static_cast<size_t>(transcript_points[i])];
        }
    }

    std::array<AffineElement, num_points> point_pairs;
    std::array<AffineElement, num_points> output_buckets;
    std::array<Fq, num_points> scratch_space;
    std::array<uint32_t, num_points> bucket_counts;
    std::array<uint32_t, num_points> bit_offsets = { 0 };

    scalar_multiplication::affine_product_runtime_state<Curve> product_state{
        &monomials[0],          &point_pairs[0],   &output_buckets[0],
        &scratch_space[0],      &bucket_counts[0], &bit_offsets[0],
        &point_schedule[0],     num_points,        2,
        &bucket_empty_status[0]
    };

    AffineElement* output = scalar_multiplication::reduce_buckets<Curve>(product_state, true);

    for (size_t i = 0; i < product_state.num_buckets; ++i) {
        expected[i] = expected[i].normalize();
        EXPECT_EQ((output[i].x == expected[i].x), true);
        EXPECT_EQ((output[i].y == expected[i].y), true);
    }
}

TYPED_TEST(ScalarMultiplicationTests, ReduceBuckets)
{
    using Curve = TypeParam;
    using Element = typename Curve::Element;
    using AffineElement = typename Curve::AffineElement;
    using Fr = typename Curve::ScalarField;
    using Fq = typename Curve::BaseField;

    constexpr size_t num_initial_points = 1 << 12;
    constexpr size_t num_points = num_initial_points * 2;
    AffineElement* monomials = (AffineElement*)(aligned_alloc(64, sizeof(AffineElement) * (num_points * 2)));
    AffineElement* scratch_points = (AffineElement*)(aligned_alloc(64, sizeof(AffineElement) * (num_points * 2)));
    AffineElement* point_pairs = (AffineElement*)(aligned_alloc(64, sizeof(AffineElement) * (num_points * 2)));
    Element* expected_buckets = (Element*)(aligned_alloc(64, sizeof(Element) * (num_points * 2)));
    bool* bucket_empty_status = (bool*)(aligned_alloc(64, sizeof(bool) * (num_points * 2)));

    memset((void*)scratch_points, 0x00, (num_points * 2) * sizeof(AffineElement));
    memset((void*)point_pairs, 0x00, (num_points * 2) * sizeof(AffineElement));
    memset((void*)expected_buckets, 0x00, (num_points * 2) * sizeof(Element));
    memset((void*)bucket_empty_status, 0x00, (num_points * 2) * sizeof(bool));

    Fq* scratch_field = (Fq*)(aligned_alloc(64, sizeof(Fq) * (num_points)));

    memset((void*)scratch_field, 0x00, num_points * sizeof(Fq));

    TestFixture::read_transcript(monomials, num_initial_points, TestFixture::SRS_PATH);

    scalar_multiplication::generate_pippenger_point_table<Curve>(monomials, monomials, num_initial_points);

    Fr* scalars = (Fr*)(aligned_alloc(64, sizeof(Fr) * num_initial_points));

    for (size_t i = 0; i < num_initial_points; ++i) {
        scalars[i] = Fr::random_element();
    }

    scalar_multiplication::pippenger_runtime_state<Curve> state(num_initial_points);

    std::chrono::steady_clock::time_point start = std::chrono::steady_clock::now();
    scalar_multiplication::compute_wnaf_states<Curve>(
        state.point_schedule, state.skew_table, state.round_counts, scalars, num_initial_points);
    std::chrono::steady_clock::time_point end = std::chrono::steady_clock::now();
    std::chrono::milliseconds diff = std::chrono::duration_cast<std::chrono::milliseconds>(end - start);
    std::cout << "wnaf time: " << diff.count() << "ms" << std::endl;

    start = std::chrono::steady_clock::now();
    scalar_multiplication::organize_buckets(state.point_schedule, num_points);
    end = std::chrono::steady_clock::now();
    diff = std::chrono::duration_cast<std::chrono::milliseconds>(end - start);
    std::cout << "organize bucket time: " << diff.count() << "ms" << std::endl;
    const size_t max_num_buckets = scalar_multiplication::get_num_buckets(num_points * 2);

    uint32_t* bucket_counts = static_cast<uint32_t*>(aligned_alloc(64, max_num_buckets * 100 * sizeof(uint32_t)));
    memset((void*)bucket_counts, 0x00, max_num_buckets * sizeof(uint32_t));
    std::array<uint32_t, 22> bit_offsets = { 0 };

    uint64_t* point_schedule_copy = static_cast<uint64_t*>(aligned_alloc(64, sizeof(uint64_t) * num_points * 2));
    for (size_t i = 0; i < num_points; ++i) {
        state.point_schedule[i + num_points] = state.point_schedule[i + num_points] & 0xffffffff7fffffffUL;
        // printf("state.point_schedule[%lu] = %lx \n", i, state.point_schedule[i]);
        point_schedule_copy[i] = state.point_schedule[i + num_points];
    }
    const size_t first_bucket = point_schedule_copy[0] & 0x7fffffffULL;
    const size_t last_bucket = point_schedule_copy[num_points - 1] & 0x7fffffffULL;
    const size_t num_buckets = last_bucket - first_bucket + 1;

    scalar_multiplication::affine_product_runtime_state<Curve> product_state{ monomials,
                                                                              point_pairs,
                                                                              scratch_points,
                                                                              scratch_field,
                                                                              bucket_counts,
                                                                              &bit_offsets[0],
                                                                              &state.point_schedule[num_points],
                                                                              num_points,
                                                                              static_cast<uint32_t>(num_buckets),
                                                                              bucket_empty_status };

    start = std::chrono::steady_clock::now();
    // scalar_multiplication::scalar_multiplication_internal<Curve><num_points>(state, monomials);
    end = std::chrono::steady_clock::now();
    diff = std::chrono::duration_cast<std::chrono::milliseconds>(end - start);
    std::cout << "scalar mul: " << diff.count() << "ms" << std::endl;

    for (size_t i = 0; i < num_points; ++i) {
        expected_buckets[i].self_set_infinity();
    }
    for (size_t i = 0; i < num_points; ++i) {
        uint64_t schedule = point_schedule_copy[i];
        uint64_t bucket_index = schedule & 0x7fffffffU;
        uint64_t point_index = schedule >> 32ULL;
        uint64_t predicate = (schedule >> 31ULL) & 1ULL;
        // printf("expected bucket index = %lu \n", bucket_index - first_bucket);
        Element& bucket = expected_buckets[bucket_index - first_bucket];
        AffineElement& point = monomials[point_index];
        bucket.self_mixed_add_or_sub(point, predicate);
    }

    size_t it = 0;

    AffineElement* result_buckets = scalar_multiplication::reduce_buckets<Curve>(product_state, true);

    printf("num buckets = %zu \n", num_buckets);
    for (size_t i = 0; i < num_buckets; ++i) {
        if (!bucket_empty_status[i]) {
            Element expected = expected_buckets[i].normalize();
            EXPECT_EQ((expected.x == result_buckets[it].x), true);
            EXPECT_EQ((expected.y == result_buckets[it].y), true);
            ++it;
        } else {
            printf("recorded empty bucket???\n");
        }
    }
    aligned_free(bucket_empty_status);
    aligned_free(expected_buckets);
    aligned_free(point_schedule_copy);
    aligned_free(point_pairs);
    aligned_free(scratch_points);
    aligned_free(scratch_field);
    aligned_free(scalars);
    aligned_free(monomials);
    aligned_free(bucket_counts);
}

// This test intermittenly fails.
TYPED_TEST(ScalarMultiplicationTests, DISABLED_ReduceBucketsBasic)
{
    using Curve = TypeParam;
    using AffineElement = typename Curve::AffineElement;
    using Fr = typename Curve::ScalarField;
    using Fq = typename Curve::BaseField;

    constexpr size_t num_initial_points = 1 << 20;
    constexpr size_t num_points = num_initial_points * 2;
    AffineElement* monomials = (AffineElement*)(aligned_alloc(64, sizeof(AffineElement) * (num_points)));
    AffineElement* scratch_points = (AffineElement*)(aligned_alloc(64, sizeof(AffineElement) * (num_points)));
    AffineElement* point_pairs = (AffineElement*)(aligned_alloc(64, sizeof(AffineElement) * (num_points)));
    bool* bucket_empty_status = (bool*)(aligned_alloc(64, sizeof(bool) * (num_points)));

    Fq* scratch_field = (Fq*)(aligned_alloc(64, sizeof(Fq) * (num_points)));

    memset((void*)scratch_points, 0x00, num_points * sizeof(AffineElement));
    memset((void*)point_pairs, 0x00, num_points * sizeof(AffineElement));
    memset((void*)scratch_field, 0x00, num_points * sizeof(Fq));
    memset((void*)bucket_empty_status, 0x00, num_points * sizeof(bool));

    TestFixture::read_transcript(monomials, num_initial_points, TestFixture::SRS_PATH);

    Fr* scalars = (Fr*)(aligned_alloc(64, sizeof(Fr) * num_initial_points));

    Fr source_scalar = Fr::random_element();
    for (size_t i = 0; i < num_initial_points; ++i) {
        source_scalar.self_sqr();
        Fr::__copy(source_scalar, scalars[i]);
    }

    scalar_multiplication::pippenger_runtime_state<Curve> state(num_initial_points);
    scalar_multiplication::generate_pippenger_point_table<Curve>(monomials, monomials, num_initial_points);

    std::chrono::steady_clock::time_point start = std::chrono::steady_clock::now();
    scalar_multiplication::compute_wnaf_states<Curve>(
        state.point_schedule, state.skew_table, state.round_counts, scalars, num_initial_points);
    std::chrono::steady_clock::time_point end = std::chrono::steady_clock::now();
    std::chrono::milliseconds diff = std::chrono::duration_cast<std::chrono::milliseconds>(end - start);
    std::cout << "wnaf time: " << diff.count() << "ms" << std::endl;

    start = std::chrono::steady_clock::now();
    scalar_multiplication::organize_buckets(state.point_schedule, num_points);
    end = std::chrono::steady_clock::now();
    diff = std::chrono::duration_cast<std::chrono::milliseconds>(end - start);
    std::cout << "organize bucket time: " << diff.count() << "ms" << std::endl;
    const size_t max_num_buckets = scalar_multiplication::get_num_buckets(num_points * 2);

    uint32_t* bucket_counts = static_cast<uint32_t*>(aligned_alloc(64, max_num_buckets * sizeof(uint32_t)));
    memset((void*)bucket_counts, 0x00, max_num_buckets * sizeof(uint32_t));
    std::array<uint32_t, 22> bit_offsets = { 0 };
    const size_t first_bucket = state.point_schedule[0] & 0x7fffffffULL;
    const size_t last_bucket = state.point_schedule[num_points - 1] & 0x7fffffffULL;
    const size_t num_buckets = last_bucket - first_bucket + 1;

    scalar_multiplication::affine_product_runtime_state<Curve> product_state{ monomials,
                                                                              point_pairs,
                                                                              scratch_points,
                                                                              scratch_field,
                                                                              bucket_counts,
                                                                              &bit_offsets[0],
                                                                              state.point_schedule,
                                                                              (uint32_t)state.round_counts[0],
                                                                              static_cast<uint32_t>(num_buckets),
                                                                              bucket_empty_status };

    start = std::chrono::steady_clock::now();
    scalar_multiplication::reduce_buckets<Curve>(product_state, true);
    // scalar_multiplication::scalar_multiplication_internal<Curve><num_points>(state, monomials);
    end = std::chrono::steady_clock::now();
    diff = std::chrono::duration_cast<std::chrono::milliseconds>(end - start);
    std::cout << "scalar mul: " << diff.count() << "ms" << std::endl;

    aligned_free(bucket_empty_status);
    aligned_free(point_pairs);
    aligned_free(scratch_points);
    aligned_free(scratch_field);
    aligned_free(scalars);
    aligned_free(monomials);
    aligned_free(bucket_counts);
}

TYPED_TEST(ScalarMultiplicationTests, AddAffinePoints)
{
    using Curve = TypeParam;
    using Element = typename Curve::Element;
    using AffineElement = typename Curve::AffineElement;
    using Fq = typename Curve::BaseField;

    constexpr size_t num_points = 20;
    AffineElement* points = (AffineElement*)(aligned_alloc(64, sizeof(AffineElement) * (num_points)));
    Fq* scratch_space = (Fq*)(aligned_alloc(64, sizeof(Fq) * (num_points * 2)));
    Fq* lambda = (Fq*)(aligned_alloc(64, sizeof(Fq) * (num_points * 2)));

    Element* points_copy = (Element*)(aligned_alloc(64, sizeof(Element) * (num_points)));
    for (size_t i = 0; i < num_points; ++i) {
        points[i] = AffineElement(Element::random_element());
        points_copy[i].x = points[i].x;
        points_copy[i].y = points[i].y;
        points_copy[i].z = Fq::one();
    }

    size_t count = num_points - 1;
    for (size_t i = num_points - 2; i < num_points; i -= 2) {
        points_copy[count--] = points_copy[i] + points_copy[i + 1];
        points_copy[count + 1] = points_copy[count + 1].normalize();
    }

    scalar_multiplication::add_affine_points<Curve>(points, num_points, scratch_space);
    for (size_t i = num_points - 1; i > num_points - 1 - (num_points / 2); --i) {
        EXPECT_EQ((points[i].x == points_copy[i].x), true);
        EXPECT_EQ((points[i].y == points_copy[i].y), true);
    }
    aligned_free(lambda);
    aligned_free(points);
    aligned_free(points_copy);
    aligned_free(scratch_space);
}

TYPED_TEST(ScalarMultiplicationTests, ConstructAdditionChains)
{
    using Curve = TypeParam;
    using AffineElement = typename Curve::AffineElement;
    using Fr = typename Curve::ScalarField;

    constexpr size_t num_initial_points = 1 << 20;
    constexpr size_t num_points = num_initial_points * 2;
    AffineElement* monomials = (AffineElement*)(aligned_alloc(64, sizeof(AffineElement) * (num_points)));

    TestFixture::read_transcript(monomials, num_initial_points, TestFixture::SRS_PATH);

    Fr* scalars = (Fr*)(aligned_alloc(64, sizeof(Fr) * num_initial_points));

    Fr source_scalar = Fr::random_element();
    for (size_t i = 0; i < num_initial_points; ++i) {
        source_scalar.self_sqr();
        Fr::__copy(source_scalar, scalars[i]);
    }

    scalar_multiplication::pippenger_runtime_state<Curve> state(num_initial_points);
    scalar_multiplication::generate_pippenger_point_table<Curve>(monomials, monomials, num_initial_points);

    std::chrono::steady_clock::time_point start = std::chrono::steady_clock::now();
    scalar_multiplication::compute_wnaf_states<Curve>(
        state.point_schedule, state.skew_table, state.round_counts, scalars, num_initial_points);
    std::chrono::steady_clock::time_point end = std::chrono::steady_clock::now();
    std::chrono::milliseconds diff = std::chrono::duration_cast<std::chrono::milliseconds>(end - start);
    std::cout << "wnaf time: " << diff.count() << "ms" << std::endl;

    start = std::chrono::steady_clock::now();
    scalar_multiplication::organize_buckets(state.point_schedule, num_points);
    end = std::chrono::steady_clock::now();
    diff = std::chrono::duration_cast<std::chrono::milliseconds>(end - start);
    std::cout << "organize bucket time: " << diff.count() << "ms" << std::endl;
    const size_t max_num_buckets = scalar_multiplication::get_num_buckets(num_points * 2);
    bool* bucket_empty_status = static_cast<bool*>(aligned_alloc(64, num_points * sizeof(bool)));
    uint32_t* bucket_counts = static_cast<uint32_t*>(aligned_alloc(64, max_num_buckets * sizeof(uint32_t)));
    memset((void*)bucket_counts, 0x00, max_num_buckets * sizeof(uint32_t));
    std::array<uint32_t, 22> bit_offsets = { 0 };
    const size_t first_bucket = state.point_schedule[0] & 0x7fffffffULL;
    const size_t last_bucket = state.point_schedule[state.round_counts[0] - 1] & 0x7fffffffULL;
    const size_t num_buckets = last_bucket - first_bucket + 1;

    scalar_multiplication::affine_product_runtime_state<Curve> product_state{ monomials,
                                                                              monomials,
                                                                              monomials,
                                                                              nullptr,
                                                                              bucket_counts,
                                                                              &bit_offsets[0],
                                                                              state.point_schedule,
                                                                              static_cast<uint32_t>(
                                                                                  state.round_counts[0]),
                                                                              static_cast<uint32_t>(num_buckets),
                                                                              bucket_empty_status };

    start = std::chrono::steady_clock::now();
    scalar_multiplication::construct_addition_chains<Curve>(product_state, true);
    end = std::chrono::steady_clock::now();
    diff = std::chrono::duration_cast<std::chrono::milliseconds>(end - start);
    info("construct addition chains: ", diff.count(), "ms");
    std::cout << "scalar mul: " << diff.count() << "ms" << std::endl;

    aligned_free(bucket_empty_status);
    aligned_free(scalars);
    aligned_free(monomials);
    aligned_free(bucket_counts);
}

TYPED_TEST(ScalarMultiplicationTests, EndomorphismSplit)
{
    using Curve = TypeParam;
    using Group = typename Curve::Group;
    using Element = typename Curve::Element;
    using AffineElement = typename Curve::AffineElement;
    using Fr = typename Curve::ScalarField;
    using Fq = typename Curve::BaseField;

    Fr scalar = Fr::random_element();

    Element expected = Group::one * scalar;

    // we want to test that we can split a scalar into two half-length components, using the same location in memory.
    Fr* k1_t = &scalar;
    Fr* k2_t = (Fr*)&scalar.data[2];

    Fr::split_into_endomorphism_scalars(scalar, *k1_t, *k2_t);
    // The compiler really doesn't like what we're doing here,
    // and disabling the array-bounds error project-wide seems unsafe.
    // The large macro blocks are here to warn that we should be careful when
    // aliasing the arguments to split_into_endomorphism_scalars
#if !defined(__clang__) && defined(__GNUC__)
#pragma GCC diagnostic push
#pragma GCC diagnostic ignored "-Warray-bounds"
#endif
    Fr k1{ (*k1_t).data[0], (*k1_t).data[1], 0, 0 };
    Fr k2{ (*k2_t).data[0], (*k2_t).data[1], 0, 0 };
#if !defined(__clang__) && defined(__GNUC__)
#pragma GCC diagnostic pop
#endif
    Element result;
    Element t1 = Group::affine_one * k1;
    AffineElement generator = Group::affine_one;
    Fq beta = Fq::cube_root_of_unity();
    generator.x = generator.x * beta;
    generator.y = -generator.y;
    Element t2 = generator * k2;
    result = t1 + t2;

    EXPECT_EQ(result == expected, true);
}

TYPED_TEST(ScalarMultiplicationTests, RadixSort)
{
    using Curve = TypeParam;
    using Fr = typename Curve::ScalarField;

    // check that our radix sort correctly sorts!
    constexpr size_t target_degree = 1 << 8;
    constexpr size_t num_rounds = scalar_multiplication::get_num_rounds(target_degree * 2);
    Fr* scalars = (Fr*)(aligned_alloc(64, sizeof(Fr) * target_degree));

    Fr source_scalar = Fr::random_element();
    for (size_t i = 0; i < target_degree; ++i) {
        source_scalar.self_sqr();
        Fr::__copy(source_scalar, scalars[i]);
    }

    scalar_multiplication::pippenger_runtime_state<Curve> state(target_degree);
    scalar_multiplication::compute_wnaf_states<Curve>(
        state.point_schedule, state.skew_table, state.round_counts, scalars, target_degree);

    uint64_t* wnaf_copy = (uint64_t*)(aligned_alloc(64, sizeof(uint64_t) * target_degree * 2 * num_rounds));
    memcpy((void*)wnaf_copy, (void*)state.point_schedule, sizeof(uint64_t) * target_degree * 2 * num_rounds);

    scalar_multiplication::organize_buckets(state.point_schedule, target_degree * 2);
    for (size_t i = 0; i < num_rounds; ++i) {
        uint64_t* unsorted_wnaf = &wnaf_copy[i * target_degree * 2];
        uint64_t* sorted_wnaf = &state.point_schedule[i * target_degree * 2];

        const auto find_entry = [unsorted_wnaf, num_entries = target_degree * 2](auto x) {
            for (size_t k = 0; k < num_entries; ++k) {
                if (unsorted_wnaf[k] == x) {
                    return true;
                }
            }
            return false;
        };
        for (size_t j = 0; j < target_degree * 2; ++j) {
            EXPECT_EQ(find_entry(sorted_wnaf[j]), true);
            if (j > 0) {
                EXPECT_EQ((sorted_wnaf[j] & 0x7fffffffU) >= (sorted_wnaf[j - 1] & 0x7fffffffU), true);
            }
        }
    }

    free(scalars);
    free(wnaf_copy);
}

TYPED_TEST(ScalarMultiplicationTests, OversizedInputs)
{
    using Curve = TypeParam;
    using Element = typename Curve::Element;
    using AffineElement = typename Curve::AffineElement;
    using Fr = typename Curve::ScalarField;
    using Fq = typename Curve::BaseField;

    // for point ranges with more than 1 << 20 points, we split into chunks of smaller multi-exps.
    // Check that this is done correctly
    size_t transcript_degree = 1 << 20;
    size_t target_degree = 1200000;
    AffineElement* monomials = (AffineElement*)(aligned_alloc(64, sizeof(AffineElement) * (2 * target_degree)));

    TestFixture::read_transcript(monomials, transcript_degree, TestFixture::SRS_PATH);

    memcpy((void*)(monomials + (2 * transcript_degree)),
           (void*)monomials,
           ((2 * target_degree - 2 * transcript_degree) * sizeof(AffineElement)));
    scalar_multiplication::generate_pippenger_point_table<Curve>(monomials, monomials, target_degree);

    Fr* scalars = (Fr*)(aligned_alloc(64, sizeof(Fr) * target_degree));

    Fr source_scalar = Fr::random_element();
    Fr accumulator = source_scalar;
    for (size_t i = 0; i < target_degree; ++i) {
        accumulator *= source_scalar;
        Fr::__copy(accumulator, scalars[i]);
    }
    scalar_multiplication::pippenger_runtime_state<Curve> state(target_degree);

    Element first = scalar_multiplication::pippenger<Curve>(scalars, monomials, target_degree, state);
    first = first.normalize();

    for (size_t i = 0; i < target_degree; ++i) {
        scalars[i].self_neg();
    }
    scalar_multiplication::pippenger_runtime_state<Curve> state_2(target_degree);

    Element second = scalar_multiplication::pippenger<Curve>(scalars, monomials, target_degree, state_2);
    second = second.normalize();

    EXPECT_EQ((first.z == second.z), true);
    EXPECT_EQ((first.z == Fq::one()), true);
    EXPECT_EQ((first.x == second.x), true);
    EXPECT_EQ((first.y == -second.y), true);

    aligned_free(monomials);
    aligned_free(scalars);
}

TYPED_TEST(ScalarMultiplicationTests, UndersizedInputs)
{
    using Curve = TypeParam;
    using Element = typename Curve::Element;
    using AffineElement = typename Curve::AffineElement;
    using Fr = typename Curve::ScalarField;

    // we fall back to traditional scalar multiplication algorithm for small input sizes.
    // Check this is done correctly
    size_t num_points = 17;

    Fr* scalars = (Fr*)aligned_alloc(32, sizeof(Fr) * num_points);

    AffineElement* points = (AffineElement*)aligned_alloc(32, sizeof(AffineElement) * (num_points * 2 + 1));

    for (size_t i = 0; i < num_points; ++i) {
        scalars[i] = Fr::random_element();
        points[i] = AffineElement(Element::random_element());
    }

    Element expected;
    expected.self_set_infinity();
    for (size_t i = 0; i < num_points; ++i) {
        Element temp = points[i] * scalars[i];
        expected += temp;
    }
    expected = expected.normalize();
    scalar_multiplication::generate_pippenger_point_table<Curve>(points, points, num_points);

    scalar_multiplication::pippenger_runtime_state<Curve> state(num_points);

    Element result = scalar_multiplication::pippenger<Curve>(scalars, points, num_points, state);
    result = result.normalize();

    aligned_free(scalars);
    aligned_free(points);

    EXPECT_EQ(result == expected, true);
}

TYPED_TEST(ScalarMultiplicationTests, PippengerSmall)
{
    using Curve = TypeParam;
    using Element = typename Curve::Element;
    using AffineElement = typename Curve::AffineElement;
    using Fr = typename Curve::ScalarField;

    constexpr size_t num_points = 8192;

    Fr* scalars = (Fr*)aligned_alloc(32, sizeof(Fr) * num_points);

    AffineElement* points = (AffineElement*)aligned_alloc(32, sizeof(AffineElement) * (num_points * 2 + 1));

    for (size_t i = 0; i < num_points; ++i) {
        scalars[i] = Fr::random_element();
        points[i] = AffineElement(Element::random_element());
    }

    Element expected;
    expected.self_set_infinity();
    for (size_t i = 0; i < num_points; ++i) {
        Element temp = points[i] * scalars[i];
        expected += temp;
    }
    expected = expected.normalize();
    scalar_multiplication::generate_pippenger_point_table<Curve>(points, points, num_points);
    scalar_multiplication::pippenger_runtime_state<Curve> state(num_points);

    Element result = scalar_multiplication::pippenger<Curve>(scalars, points, num_points, state);
    result = result.normalize();

    aligned_free(scalars);
    aligned_free(points);

    EXPECT_EQ(result == expected, true);
}

TYPED_TEST(ScalarMultiplicationTests, PippengerEdgeCaseDbl)
{
    using Curve = TypeParam;
    using Element = typename Curve::Element;
    using AffineElement = typename Curve::AffineElement;
    using Fr = typename Curve::ScalarField;

    constexpr size_t num_points = 128;

    Fr* scalars = (Fr*)aligned_alloc(32, sizeof(Fr) * num_points);

    AffineElement* points = (AffineElement*)aligned_alloc(32, sizeof(AffineElement) * (num_points * 2 + 1));

    AffineElement point = AffineElement(Element::random_element());
    for (size_t i = 0; i < num_points; ++i) {
        scalars[i] = Fr::random_element();
        points[i] = point;
    }

    Element expected;
    expected.self_set_infinity();
    for (size_t i = 0; i < num_points; ++i) {
        Element temp = points[i] * scalars[i];
        expected += temp;
    }
    if (!expected.is_point_at_infinity()) {
        expected = expected.normalize();
    }
    scalar_multiplication::generate_pippenger_point_table<Curve>(points, points, num_points);
    scalar_multiplication::pippenger_runtime_state<Curve> state(num_points);
    Element result = scalar_multiplication::pippenger<Curve>(scalars, points, num_points, state);
    result = result.normalize();

    aligned_free(scalars);
    aligned_free(points);

    EXPECT_EQ(result == expected, true);
}

TYPED_TEST(ScalarMultiplicationTests, PippengerShortInputs)
{
    using Curve = TypeParam;
    using Element = typename Curve::Element;
    using AffineElement = typename Curve::AffineElement;
    using Fr = typename Curve::ScalarField;

    constexpr size_t num_points = 8192;

    Fr* scalars = (Fr*)aligned_alloc(32, sizeof(Fr) * num_points);

    auto points = scalar_multiplication::point_table_alloc<AffineElement>(num_points);

    for (std::ptrdiff_t i = 0; i < (std::ptrdiff_t)num_points; ++i) {
        points[i] = AffineElement(Element::random_element());
    }
    for (size_t i = 0; i < (num_points / 4); ++i) {
        scalars[i * 4].data[0] = engine.get_random_uint32();
        scalars[i * 4].data[1] = engine.get_random_uint32();
        scalars[i * 4].data[2] = engine.get_random_uint32();
        scalars[i * 4].data[3] = engine.get_random_uint32();
        scalars[i * 4] = scalars[i * 4].to_montgomery_form();
        scalars[i * 4 + 1].data[0] = 0;
        scalars[i * 4 + 1].data[1] = 0;
        scalars[i * 4 + 1].data[2] = 0;
        scalars[i * 4 + 1].data[3] = 0;
        scalars[i * 4 + 1] = scalars[i * 4 + 1].to_montgomery_form();
        scalars[i * 4 + 2].data[0] = engine.get_random_uint32();
        scalars[i * 4 + 2].data[1] = engine.get_random_uint32();
        scalars[i * 4 + 2].data[2] = 0;
        scalars[i * 4 + 2].data[3] = 0;
        scalars[i * 4 + 2] = scalars[i * 4 + 2].to_montgomery_form();
        scalars[i * 4 + 3].data[0] = (engine.get_random_uint32() & 0x07ULL);
        scalars[i * 4 + 3].data[1] = 0;
        scalars[i * 4 + 3].data[2] = 0;
        scalars[i * 4 + 3].data[3] = 0;
        scalars[i * 4 + 3] = scalars[i * 4 + 3].to_montgomery_form();
    }

    Element expected;
    expected.self_set_infinity();
    for (std::ptrdiff_t i = 0; i < (std::ptrdiff_t)num_points; ++i) {
        Element temp = points[i] * scalars[i];
        expected += temp;
    }
    expected = expected.normalize();
    scalar_multiplication::generate_pippenger_point_table<Curve>(points.get(), points.get(), num_points);
    scalar_multiplication::pippenger_runtime_state<Curve> state(num_points);

    Element result = scalar_multiplication::pippenger<Curve>(scalars, points.get(), num_points, state);
    result = result.normalize();

    aligned_free(scalars);

    EXPECT_EQ(result == expected, true);
}

TYPED_TEST(ScalarMultiplicationTests, PippengerUnsafe)
{
    using Curve = TypeParam;
    using Element = typename Curve::Element;
    using AffineElement = typename Curve::AffineElement;
    using Fr = typename Curve::ScalarField;

    constexpr size_t num_points = 8192;

    Fr* scalars = (Fr*)aligned_alloc(32, sizeof(Fr) * num_points);

    auto points = scalar_multiplication::point_table_alloc<AffineElement>(num_points);

    for (std::ptrdiff_t i = 0; i < (std::ptrdiff_t)num_points; ++i) {
        scalars[i] = Fr::random_element();
        points[i] = AffineElement(Element::random_element());
    }

    Element expected;
    expected.self_set_infinity();
    for (std::ptrdiff_t i = 0; i < (std::ptrdiff_t)num_points; ++i) {
        Element temp = points[i] * scalars[i];
        expected += temp;
    }
    expected = expected.normalize();
    scalar_multiplication::generate_pippenger_point_table<Curve>(points.get(), points.get(), num_points);

    scalar_multiplication::pippenger_runtime_state<Curve> state(num_points);
    Element result = scalar_multiplication::pippenger_unsafe<Curve>(scalars, points.get(), num_points, state);
    result = result.normalize();

    aligned_free(scalars);

    EXPECT_EQ(result == expected, true);
}

TYPED_TEST(ScalarMultiplicationTests, PippengerUnsafeShortInputs)
{
    using Curve = TypeParam;
    using Element = typename Curve::Element;
    using AffineElement = typename Curve::AffineElement;
    using Fr = typename Curve::ScalarField;

    constexpr size_t num_points = 8192;

    Fr* scalars = (Fr*)aligned_alloc(32, sizeof(Fr) * num_points);

    AffineElement* points = (AffineElement*)aligned_alloc(32, sizeof(AffineElement) * (num_points * 2 + 1));

    for (size_t i = 0; i < num_points; ++i) {
        points[i] = AffineElement(Element::random_element());
    }
    for (size_t i = 0; i < (num_points / 4); ++i) {
        scalars[i * 4].data[0] = engine.get_random_uint32();
        scalars[i * 4].data[1] = engine.get_random_uint32();
        scalars[i * 4].data[2] = engine.get_random_uint32();
        scalars[i * 4].data[3] = engine.get_random_uint32();
        scalars[i * 4] = scalars[i * 4].to_montgomery_form();
        scalars[i * 4 + 1].data[0] = 0;
        scalars[i * 4 + 1].data[1] = 0;
        scalars[i * 4 + 1].data[2] = 0;
        scalars[i * 4 + 1].data[3] = 0;
        scalars[i * 4 + 1] = scalars[i * 4 + 1].to_montgomery_form();
        scalars[i * 4 + 2].data[0] = engine.get_random_uint32();
        scalars[i * 4 + 2].data[1] = engine.get_random_uint32();
        scalars[i * 4 + 2].data[2] = 0;
        scalars[i * 4 + 2].data[3] = 0;
        scalars[i * 4 + 2] = scalars[i * 4 + 2].to_montgomery_form();
        scalars[i * 4 + 3].data[0] = (engine.get_random_uint32() & 0x07ULL);
        scalars[i * 4 + 3].data[1] = 0;
        scalars[i * 4 + 3].data[2] = 0;
        scalars[i * 4 + 3].data[3] = 0;
        scalars[i * 4 + 3] = scalars[i * 4 + 3].to_montgomery_form();
    }

    Element expected;
    expected.self_set_infinity();
    for (size_t i = 0; i < num_points; ++i) {
        Element temp = points[i] * scalars[i];
        expected += temp;
    }
    expected = expected.normalize();
    scalar_multiplication::generate_pippenger_point_table<Curve>(points, points, num_points);
    scalar_multiplication::pippenger_runtime_state<Curve> state(num_points);

    Element result = scalar_multiplication::pippenger_unsafe<Curve>(scalars, points, num_points, state);
    result = result.normalize();

    aligned_free(scalars);
    aligned_free(points);

    EXPECT_EQ(result == expected, true);
}

TYPED_TEST(ScalarMultiplicationTests, PippengerOne)
{
    using Curve = TypeParam;
    using Element = typename Curve::Element;
    using AffineElement = typename Curve::AffineElement;
    using Fr = typename Curve::ScalarField;

    size_t num_points = 1;

    Fr* scalars = (Fr*)aligned_alloc(32, sizeof(Fr) * 1);

    AffineElement* points = (AffineElement*)aligned_alloc(32, sizeof(AffineElement) * (num_points * 2 + 1));

    for (size_t i = 0; i < num_points; ++i) {
        scalars[i] = Fr::random_element();
        points[i] = AffineElement(Element::random_element());
    }

    Element expected;
    expected.self_set_infinity();
    for (size_t i = 0; i < num_points; ++i) {
        Element temp = points[i] * scalars[i];
        expected += temp;
    }
    expected = expected.normalize();
    scalar_multiplication::generate_pippenger_point_table<Curve>(points, points, num_points);
    scalar_multiplication::pippenger_runtime_state<Curve> state(num_points);

    Element result = scalar_multiplication::pippenger<Curve>(scalars, points, num_points, state);
    result = result.normalize();

    aligned_free(scalars);
    aligned_free(points);

    EXPECT_EQ(result == expected, true);
}

TYPED_TEST(ScalarMultiplicationTests, PippengerZeroPoints)
{
    using Curve = TypeParam;
    using Element = typename Curve::Element;
    using AffineElement = typename Curve::AffineElement;
    using Fr = typename Curve::ScalarField;

    Fr* scalars = (Fr*)aligned_alloc(32, sizeof(Fr));

    AffineElement* points = (AffineElement*)aligned_alloc(32, sizeof(AffineElement) * (2 + 1));

    scalar_multiplication::pippenger_runtime_state<Curve> state(0);
    Element result = scalar_multiplication::pippenger<Curve>(scalars, points, 0, state);

    aligned_free(scalars);
    aligned_free(points);

    EXPECT_EQ(result.is_point_at_infinity(), true);
}

TYPED_TEST(ScalarMultiplicationTests, PippengerMulByZero)
{
    using Curve = TypeParam;
    using Group = typename Curve::Group;
    using Element = typename Curve::Element;
    using AffineElement = typename Curve::AffineElement;
    using Fr = typename Curve::ScalarField;

    Fr* scalars = (Fr*)aligned_alloc(32, sizeof(Fr));

    AffineElement* points = (AffineElement*)aligned_alloc(32, sizeof(AffineElement) * (2 + 1));

    scalars[0] = Fr::zero();
    points[0] = Group::affine_one;
    scalar_multiplication::generate_pippenger_point_table<Curve>(points, points, 1);

    scalar_multiplication::pippenger_runtime_state<Curve> state(1);
    Element result = scalar_multiplication::pippenger<Curve>(scalars, points, 1, state);

    aligned_free(scalars);
    aligned_free(points);

    EXPECT_EQ(result.is_point_at_infinity(), true);
}
