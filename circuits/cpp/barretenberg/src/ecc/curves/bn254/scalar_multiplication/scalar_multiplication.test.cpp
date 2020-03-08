#include <gtest/gtest.h>
#include <chrono>
#include <vector>
#include <srs/io.hpp>
#include "scalar_multiplication.cpp"

#define BARRETENBERG_SRS_PATH "../srs_db"

using namespace barretenberg;

// TODO: refactor pippenger tests. These are a hot mess...
TEST(scalar_multiplication, reduce_buckets_simple)
{
    constexpr size_t num_points = 128;
    std::array<g1::affine_element, num_points> monomials;
    g2::affine_element g2_x;
    io::read_transcript(&monomials[0], g2_x, num_points / 2, BARRETENBERG_SRS_PATH);
    scalar_multiplication::generate_pippenger_point_table(&monomials[0], &monomials[0], num_points / 2);

    std::array<uint64_t, num_points> point_schedule;
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
    std::array<g1::element, num_points> expected;
    for (size_t i = 0; i < num_points; ++i) {
        expected[i].self_set_infinity();
    }

    for (size_t i = 0; i < num_points; ++i) {
        uint64_t schedule = transcript[i] & 0x7fffffffU;
        {
            expected[schedule] += monomials[transcript_points[i]];
        }
    }

    std::array<g1::affine_element, num_points> point_pairs;
    std::array<g1::affine_element, num_points> output_buckets;
    std::array<fq, num_points> scratch_space;
    std::array<uint32_t, num_points> bucket_counts;
    std::array<uint32_t, num_points> bit_offsets = { 0 };

    scalar_multiplication::affine_product_runtime_state product_state{
        &monomials[0],          &point_pairs[0],   &output_buckets[0],
        &scratch_space[0],      &bucket_counts[0], &bit_offsets[0],
        &point_schedule[0],     num_points,        2,
        &bucket_empty_status[0]
    };

    g1::affine_element* output = scalar_multiplication::reduce_buckets(product_state, true);

    for (size_t i = 0; i < product_state.num_buckets; ++i) {
        expected[i] = expected[i].normalize();
        EXPECT_EQ((output[i].x == expected[i].x), true);
        EXPECT_EQ((output[i].y == expected[i].y), true);
    }
}

TEST(scalar_multiplication, reduce_buckets)
{
    constexpr size_t num_initial_points = 1 << 12;
    constexpr size_t num_points = num_initial_points * 2;
    g1::affine_element* monomials =
        (g1::affine_element*)(aligned_alloc(64, sizeof(g1::affine_element) * (num_points * 2)));
    g1::affine_element* scratch_points =
        (g1::affine_element*)(aligned_alloc(64, sizeof(g1::affine_element) * (num_points * 2)));
    g1::affine_element* point_pairs =
        (g1::affine_element*)(aligned_alloc(64, sizeof(g1::affine_element) * (num_points * 2)));
    g1::element* expected_buckets = (g1::element*)(aligned_alloc(64, sizeof(g1::element) * (num_points * 2)));
    bool* bucket_empty_status = (bool*)(aligned_alloc(64, sizeof(bool) * (num_points * 2)));

    memset((void*)scratch_points, 0x00, (num_points * 2) * sizeof(g1::affine_element));
    memset((void*)point_pairs, 0x00, (num_points * 2) * sizeof(g1::affine_element));
    memset((void*)expected_buckets, 0x00, (num_points * 2) * sizeof(g1::element));
    memset((void*)bucket_empty_status, 0x00, (num_points * 2) * sizeof(bool));

    fq* scratch_field = (fq*)(aligned_alloc(64, sizeof(fq) * (num_points)));

    memset((void*)scratch_field, 0x00, num_points * sizeof(fq));

    g2::affine_element g2_x;
    io::read_transcript(monomials, g2_x, num_initial_points, BARRETENBERG_SRS_PATH);

    scalar_multiplication::generate_pippenger_point_table(monomials, monomials, num_initial_points);

    fr* scalars = (fr*)(aligned_alloc(64, sizeof(fr) * num_initial_points));

    fr source_scalar = fr::random_element();
    for (size_t i = 0; i < num_initial_points; ++i) {
        // source_scalar.self_sqr();
        source_scalar = fr::random_element();
        fr::__copy(source_scalar, scalars[i]);
    }

    // scalar_multiplication::generate_pippenger_point_table(monomials, monomials, num_initial_points);
    scalar_multiplication::multiplication_runtime_state state;

    std::chrono::steady_clock::time_point start = std::chrono::steady_clock::now();
    scalar_multiplication::compute_wnaf_states<num_initial_points>(state, scalars);
    std::chrono::steady_clock::time_point end = std::chrono::steady_clock::now();
    std::chrono::milliseconds diff = std::chrono::duration_cast<std::chrono::milliseconds>(end - start);
    std::cout << "wnaf time: " << diff.count() << "ms" << std::endl;

    start = std::chrono::steady_clock::now();
    scalar_multiplication::organize_buckets<num_points>(state);
    end = std::chrono::steady_clock::now();
    diff = std::chrono::duration_cast<std::chrono::milliseconds>(end - start);
    std::cout << "organize bucket time: " << diff.count() << "ms" << std::endl;
    const size_t max_num_buckets = scalar_multiplication::get_num_buckets(num_points * 2);

    uint32_t* bucket_counts = static_cast<uint32_t*>(aligned_alloc(64, max_num_buckets * 100 * sizeof(uint32_t)));
    memset((void*)bucket_counts, 0x00, max_num_buckets * sizeof(uint32_t));
    std::array<uint32_t, 22> bit_offsets = { 0 };

    uint64_t* point_schedule_copy = static_cast<uint64_t*>(aligned_alloc(64, sizeof(uint64_t) * num_points * 2));
    for (size_t i = 0; i < num_points; ++i) {
        state.wnaf_table[i + num_points] = state.wnaf_table[i + num_points] & 0xffffffff7fffffffUL;
        // printf("state.wnaf_table[%lu] = %lx \n", i, state.wnaf_table[i]);
        point_schedule_copy[i] = state.wnaf_table[i + num_points];
    }
    const size_t first_bucket = point_schedule_copy[0] & 0x7fffffffULL;
    const size_t last_bucket = point_schedule_copy[num_points - 1] & 0x7fffffffULL;
    const size_t num_buckets = last_bucket - first_bucket + 1;

    scalar_multiplication::affine_product_runtime_state product_state{ monomials,
                                                                       point_pairs,
                                                                       scratch_points,
                                                                       scratch_field,
                                                                       bucket_counts,
                                                                       &bit_offsets[0],
                                                                       &state.wnaf_table[num_points],
                                                                       num_points,
                                                                       static_cast<uint32_t>(num_buckets),
                                                                       bucket_empty_status };

    start = std::chrono::steady_clock::now();
    // scalar_multiplication::scalar_multiplication_internal<num_points>(state, monomials);
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
        g1::element& bucket = expected_buckets[bucket_index - first_bucket];
        g1::affine_element& point = monomials[point_index];
        bucket.self_mixed_add_or_sub(point, predicate);
    }

    size_t it = 0;

    g1::affine_element* result_buckets = scalar_multiplication::reduce_buckets(product_state, true);

    printf("num buckets = %lu \n", num_buckets);
    for (size_t i = 0; i < num_buckets; ++i) {
        if (!bucket_empty_status[i]) {
            g1::element expected = expected_buckets[i].normalize();
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

TEST(scalar_multiplication, reduce_buckets_basic)
{
    constexpr size_t num_initial_points = 1 << 20;
    constexpr size_t num_points = num_initial_points * 2;
    g1::affine_element* monomials = (g1::affine_element*)(aligned_alloc(64, sizeof(g1::affine_element) * (num_points)));
    g1::affine_element* scratch_points =
        (g1::affine_element*)(aligned_alloc(64, sizeof(g1::affine_element) * (num_points)));
    g1::affine_element* point_pairs =
        (g1::affine_element*)(aligned_alloc(64, sizeof(g1::affine_element) * (num_points)));
    bool* bucket_empty_status = (bool*)(aligned_alloc(64, sizeof(bool) * (num_points)));

    fq* scratch_field = (fq*)(aligned_alloc(64, sizeof(fq) * (num_points)));

    memset((void*)scratch_points, 0x00, num_points * sizeof(g1::affine_element));
    memset((void*)point_pairs, 0x00, num_points * sizeof(g1::affine_element));
    memset((void*)scratch_field, 0x00, num_points * sizeof(fq));
    memset((void*)bucket_empty_status, 0x00, num_points * sizeof(bool));

    g2::affine_element g2_x;
    io::read_transcript(monomials, g2_x, num_initial_points, BARRETENBERG_SRS_PATH);

    scalar_multiplication::generate_pippenger_point_table(monomials, monomials, num_initial_points);

    fr* scalars = (fr*)(aligned_alloc(64, sizeof(fr) * num_initial_points));

    fr source_scalar = fr::random_element();
    for (size_t i = 0; i < num_initial_points; ++i) {
        source_scalar.self_sqr();
        fr::__copy(source_scalar, scalars[i]);
    }

    scalar_multiplication::multiplication_runtime_state state;
    scalar_multiplication::generate_pippenger_point_table(monomials, monomials, num_initial_points);

    std::chrono::steady_clock::time_point start = std::chrono::steady_clock::now();
    scalar_multiplication::compute_wnaf_states<num_initial_points>(state, scalars);
    std::chrono::steady_clock::time_point end = std::chrono::steady_clock::now();
    std::chrono::milliseconds diff = std::chrono::duration_cast<std::chrono::milliseconds>(end - start);
    std::cout << "wnaf time: " << diff.count() << "ms" << std::endl;

    start = std::chrono::steady_clock::now();
    scalar_multiplication::organize_buckets<num_points>(state);
    end = std::chrono::steady_clock::now();
    diff = std::chrono::duration_cast<std::chrono::milliseconds>(end - start);
    std::cout << "organize bucket time: " << diff.count() << "ms" << std::endl;
    const size_t max_num_buckets = scalar_multiplication::get_num_buckets(num_points * 2);

    uint32_t* bucket_counts = static_cast<uint32_t*>(aligned_alloc(64, max_num_buckets * sizeof(uint32_t)));
    memset((void*)bucket_counts, 0x00, max_num_buckets * sizeof(uint32_t));
    std::array<uint32_t, 22> bit_offsets = { 0 };
    const size_t first_bucket = state.wnaf_table[0] & 0x7fffffffULL;
    const size_t last_bucket = state.wnaf_table[num_points - 1] & 0x7fffffffULL;
    const size_t num_buckets = last_bucket - first_bucket + 1;

    scalar_multiplication::affine_product_runtime_state product_state{
        monomials,          point_pairs,   scratch_points,
        scratch_field,      bucket_counts, &bit_offsets[0],
        state.wnaf_table,   num_points,    static_cast<uint32_t>(num_buckets),
        bucket_empty_status
    };

    start = std::chrono::steady_clock::now();
    scalar_multiplication::reduce_buckets(product_state, true);
    // scalar_multiplication::scalar_multiplication_internal<num_points>(state, monomials);
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

TEST(scalar_multiplication, add_affine_points)
{
    constexpr size_t num_points = 20;
    g1::affine_element* points = (g1::affine_element*)(aligned_alloc(64, sizeof(g1::affine_element) * (num_points)));
    fq* scratch_space = (fq*)(aligned_alloc(64, sizeof(fq) * (num_points * 2)));
    fq* lambda = (fq*)(aligned_alloc(64, sizeof(fq) * (num_points * 2)));

    g1::element* points_copy = (g1::element*)(aligned_alloc(64, sizeof(g1::element) * (num_points)));
    for (size_t i = 0; i < num_points; ++i) {
        points[i] = g1::affine_element(g1::element::random_element());
        points_copy[i].x = points[i].x;
        points_copy[i].y = points[i].y;
        points_copy[i].z = fq::one();
    }

    size_t count = num_points - 1;
    for (size_t i = num_points - 2; i < num_points; i -= 2) {
        points_copy[count--] = points_copy[i] + points_copy[i + 1];
        points_copy[count + 1] = points_copy[count + 1].normalize();
    }

    scalar_multiplication::add_affine_points(points, num_points, scratch_space);
    for (size_t i = num_points - 1; i > num_points - 1 - (num_points / 2); --i) {
        EXPECT_EQ((points[i].x == points_copy[i].x), true);
        EXPECT_EQ((points[i].y == points_copy[i].y), true);
    }
    aligned_free(lambda);
    aligned_free(points);
    aligned_free(points_copy);
    aligned_free(scratch_space);
}

TEST(scalar_multiplication, construct_addition_chains)
{
    constexpr size_t num_initial_points = 1 << 20;
    constexpr size_t num_points = num_initial_points * 2;
    g1::affine_element* monomials = (g1::affine_element*)(aligned_alloc(64, sizeof(g1::affine_element) * (num_points)));

    g2::affine_element g2_x;
    io::read_transcript(monomials, g2_x, num_initial_points, BARRETENBERG_SRS_PATH);

    scalar_multiplication::generate_pippenger_point_table(monomials, monomials, num_initial_points);

    fr* scalars = (fr*)(aligned_alloc(64, sizeof(fr) * num_initial_points));

    fr source_scalar = fr::random_element();
    for (size_t i = 0; i < num_initial_points; ++i) {
        source_scalar.self_sqr();
        fr::__copy(source_scalar, scalars[i]);
    }

    scalar_multiplication::multiplication_runtime_state state;
    scalar_multiplication::generate_pippenger_point_table(monomials, monomials, num_initial_points);

    std::chrono::steady_clock::time_point start = std::chrono::steady_clock::now();
    scalar_multiplication::compute_wnaf_states<num_initial_points>(state, scalars);
    std::chrono::steady_clock::time_point end = std::chrono::steady_clock::now();
    std::chrono::milliseconds diff = std::chrono::duration_cast<std::chrono::milliseconds>(end - start);
    std::cout << "wnaf time: " << diff.count() << "ms" << std::endl;

    start = std::chrono::steady_clock::now();
    scalar_multiplication::organize_buckets<num_points>(state);
    end = std::chrono::steady_clock::now();
    diff = std::chrono::duration_cast<std::chrono::milliseconds>(end - start);
    std::cout << "organize bucket time: " << diff.count() << "ms" << std::endl;
    const size_t max_num_buckets = scalar_multiplication::get_num_buckets(num_points * 2);
    bool* bucket_empty_status = static_cast<bool*>(aligned_alloc(64, num_points * sizeof(bool)));
    uint32_t* bucket_counts = static_cast<uint32_t*>(aligned_alloc(64, max_num_buckets * sizeof(uint32_t)));
    memset((void*)bucket_counts, 0x00, max_num_buckets * sizeof(uint32_t));
    std::array<uint32_t, 22> bit_offsets = { 0 };
    const size_t first_bucket = state.wnaf_table[0] & 0x7fffffffULL;
    const size_t last_bucket = state.wnaf_table[num_points - 1] & 0x7fffffffULL;
    const size_t num_buckets = last_bucket - first_bucket + 1;

    scalar_multiplication::affine_product_runtime_state product_state{ monomials,
                                                                       monomials,
                                                                       monomials,
                                                                       nullptr,
                                                                       bucket_counts,
                                                                       &bit_offsets[0],
                                                                       state.wnaf_table,
                                                                       num_points,
                                                                       static_cast<uint32_t>(num_buckets),
                                                                       bucket_empty_status };

    start = std::chrono::steady_clock::now();
    scalar_multiplication::construct_addition_chains(product_state, true);
    // scalar_multiplication::scalar_multiplication_internal<num_points>(state, monomials);
    end = std::chrono::steady_clock::now();
    diff = std::chrono::duration_cast<std::chrono::milliseconds>(end - start);
    std::cout << "scalar mul: " << diff.count() << "ms" << std::endl;

    aligned_free(bucket_empty_status);
    aligned_free(scalars);
    aligned_free(monomials);
    aligned_free(bucket_counts);
}

// TEST(scalar_multiplication, pippenger_timing)
// {
//     constexpr size_t num_initial_points = 1 << 20;
//     constexpr size_t num_points = num_initial_points * 2;
//     g1::affine_element* monomials = (g1::affine_element*)(aligned_alloc(64, sizeof(g1::affine_element) *
//     (num_points))); g2::affine_element g2_x; io::read_transcript(monomials, g2_x, num_initial_points,
//     BARRETENBERG_SRS_PATH);

//     scalar_multiplication::generate_pippenger_point_table(monomials, monomials, num_initial_points);

//     fr* scalars = (fr*)(aligned_alloc(64, sizeof(fr) * num_initial_points));

//     fr source_scalar = fr::random_element();
//     for (size_t i = 0; i < num_initial_points; ++i) {
//         source_scalar.self_sqr();
//         fr::__copy(source_scalar, scalars[i]);
//     }

//     scalar_multiplication::multiplication_runtime_state state;
//     scalar_multiplication::generate_pippenger_point_table(monomials, monomials, num_initial_points);

//     std::chrono::steady_clock::time_point start = std::chrono::steady_clock::now();
//     scalar_multiplication::compute_wnaf_states<num_initial_points>(state, scalars);
//     std::chrono::steady_clock::time_point end = std::chrono::steady_clock::now();
//     std::chrono::milliseconds diff = std::chrono::duration_cast<std::chrono::milliseconds>(end - start);
//     std::cout << "wnaf time: " << diff.count() << "ms" << std::endl;

//     start = std::chrono::steady_clock::now();
//     scalar_multiplication::organize_buckets<num_points>(state);
//     end = std::chrono::steady_clock::now();
//     diff = std::chrono::duration_cast<std::chrono::milliseconds>(end - start);
//     std::cout << "organize bucket time: " << diff.count() << "ms" << std::endl;

//     start = std::chrono::steady_clock::now();
//     scalar_multiplication::scalar_multiplication_internal<num_points>(state, monomials);
//     end = std::chrono::steady_clock::now();
//     diff = std::chrono::duration_cast<std::chrono::milliseconds>(end - start);
//     std::cout << "scalar mul: " << diff.count() << "ms" << std::endl;

//     aligned_free(scalars);
//     aligned_free(monomials);
// }

TEST(scalar_multiplication, endomorphism_split)
{
    fr scalar = fr::random_element();

    g1::element expected = g1::one * scalar;

    // we want to test that we can split a scalar into two half-length components, using the same location in memory.
    fr* k1_t = &scalar;
    fr* k2_t = (fr*)&scalar.data[2];

    fr::split_into_endomorphism_scalars(scalar, *k1_t, *k2_t);

    fr k1{ (*k1_t).data[0], (*k1_t).data[1], 0, 0 };
    fr k2{ (*k2_t).data[0], (*k2_t).data[1], 0, 0 };

    g1::element result;
    g1::element t1 = g1::affine_one * k1;
    g1::affine_element beta = g1::affine_one;
    beta.x = beta.x * fq::beta();
    beta.y = -beta.y;
    g1::element t2 = beta * k2;
    result = t1 + t2;

    EXPECT_EQ(result == expected, true);
}

TEST(scalar_multiplication, radix_sort)
{
    // check that our radix sort correctly sorts!
    constexpr size_t target_degree = 1 << 8;
    constexpr size_t num_rounds = scalar_multiplication::get_num_rounds(target_degree * 2);
    fr* scalars = (fr*)(aligned_alloc(64, sizeof(fr) * target_degree));

    fr source_scalar = fr::random_element();
    for (size_t i = 0; i < target_degree; ++i) {
        source_scalar.self_sqr();
        fr::__copy(source_scalar, scalars[i]);
    }

    scalar_multiplication::multiplication_runtime_state state;
    scalar_multiplication::compute_wnaf_states<target_degree>(state, scalars);

    uint64_t* wnaf_copy = (uint64_t*)(aligned_alloc(64, sizeof(uint64_t) * target_degree * 2 * num_rounds));
    memcpy((void*)wnaf_copy, (void*)state.wnaf_table, sizeof(uint64_t) * target_degree * 2 * num_rounds);

    scalar_multiplication::organize_buckets<target_degree * 2>(state);
    for (size_t i = 0; i < num_rounds; ++i) {
        uint64_t* unsorted_wnaf = &wnaf_copy[i * target_degree * 2];
        uint64_t* sorted_wnaf = &state.wnaf_table[i * target_degree * 2];

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

TEST(scalar_multiplication, oversized_inputs)
{
    // for point ranges with more than 1 << 20 points, we split into chunks of smaller multi-exps.
    // Check that this is done correctly
    size_t transcript_degree = 1 << 20;
    size_t target_degree = 1200000;
    g1::affine_element* monomials =
        (g1::affine_element*)(aligned_alloc(64, sizeof(g1::affine_element) * (2 * target_degree)));
    g2::affine_element g2_x;
    io::read_transcript(monomials, g2_x, transcript_degree, BARRETENBERG_SRS_PATH);

    memcpy((void*)(monomials + (2 * transcript_degree)),
           (void*)monomials,
           ((2 * target_degree - 2 * transcript_degree) * sizeof(g1::affine_element)));
    scalar_multiplication::generate_pippenger_point_table(monomials, monomials, target_degree);

    fr* scalars = (fr*)(aligned_alloc(64, sizeof(fr) * target_degree));

    fr source_scalar = fr::random_element();
    for (size_t i = 0; i < target_degree; ++i) {
        source_scalar.self_sqr();
        fr::__copy(source_scalar, scalars[i]);
    }

    g1::element first = scalar_multiplication::pippenger(scalars, monomials, target_degree);
    first = first.normalize();

    for (size_t i = 0; i < target_degree; ++i) {
        scalars[i].self_neg();
    }

    g1::element second = scalar_multiplication::pippenger(scalars, monomials, target_degree);
    second = second.normalize();

    EXPECT_EQ((first.z == second.z), true);
    EXPECT_EQ((first.z == fq::one()), true);
    EXPECT_EQ((first.x == second.x), true);
    EXPECT_EQ((first.y == -second.y), true);

    aligned_free(monomials);
    aligned_free(scalars);
}

TEST(scalar_multiplication, undersized_inputs)
{
    // we fall back to traditional scalar multiplication algorithm for small input sizes.
    // Check this is done correctly
    size_t num_points = 17;

    fr* scalars = (fr*)aligned_alloc(32, sizeof(fr) * num_points);

    g1::affine_element* points =
        (g1::affine_element*)aligned_alloc(32, sizeof(g1::affine_element) * num_points * 2 + 1);

    for (size_t i = 0; i < num_points; ++i) {
        scalars[i] = fr::random_element();
        points[i] = g1::affine_element(g1::element::random_element());
    }

    g1::element expected;
    expected.self_set_infinity();
    for (size_t i = 0; i < num_points; ++i) {
        g1::element temp = points[i] * scalars[i];
        expected += temp;
    }
    expected = expected.normalize();
    scalar_multiplication::generate_pippenger_point_table(points, points, num_points);

    g1::element result = scalar_multiplication::pippenger(scalars, points, num_points);
    result = result.normalize();

    aligned_free(scalars);
    aligned_free(points);

    EXPECT_EQ(result == expected, true);
}

TEST(scalar_multiplication, pippenger)
{
    constexpr size_t num_points = 8192;

    fr* scalars = (fr*)aligned_alloc(32, sizeof(fr) * num_points);

    g1::affine_element* points =
        (g1::affine_element*)aligned_alloc(32, sizeof(g1::affine_element) * num_points * 2 + 1);

    for (size_t i = 0; i < num_points; ++i) {
        scalars[i] = fr::random_element();
        points[i] = g1::affine_element(g1::element::random_element());
    }

    g1::element expected;
    expected.self_set_infinity();
    for (size_t i = 0; i < num_points; ++i) {
        g1::element temp = points[i] * scalars[i];
        expected += temp;
    }
    expected = expected.normalize();
    scalar_multiplication::generate_pippenger_point_table(points, points, num_points);

    g1::element result = scalar_multiplication::pippenger(scalars, points, num_points);
    result = result.normalize();

    aligned_free(scalars);
    aligned_free(points);

    EXPECT_EQ(result == expected, true);
}

TEST(scalar_multiplication, pippenger_unsafe)
{
    constexpr size_t num_points = 8192;

    fr* scalars = (fr*)aligned_alloc(32, sizeof(fr) * num_points);

    g1::affine_element* points =
        (g1::affine_element*)aligned_alloc(32, sizeof(g1::affine_element) * num_points * 2 + 1);

    for (size_t i = 0; i < num_points; ++i) {
        scalars[i] = fr::random_element();
        points[i] = g1::affine_element(g1::element::random_element());
    }

    g1::element expected;
    expected.self_set_infinity();
    for (size_t i = 0; i < num_points; ++i) {
        g1::element temp = points[i] * scalars[i];
        expected += temp;
    }
    expected = expected.normalize();
    scalar_multiplication::generate_pippenger_point_table(points, points, num_points);

    g1::element result = scalar_multiplication::pippenger_unsafe(scalars, points, num_points);
    result = result.normalize();

    aligned_free(scalars);
    aligned_free(points);

    EXPECT_EQ(result == expected, true);
}

TEST(scalar_multiplication, pippenger_one)
{
    size_t num_points = 1;

    fr* scalars = (fr*)aligned_alloc(32, sizeof(fr) * 1);

    g1::affine_element* points =
        (g1::affine_element*)aligned_alloc(32, sizeof(g1::affine_element) * num_points * 2 + 1);

    for (size_t i = 0; i < num_points; ++i) {
        scalars[i] = fr::random_element();
        points[i] = g1::affine_element(g1::element::random_element());
    }

    g1::element expected;
    expected.self_set_infinity();
    for (size_t i = 0; i < num_points; ++i) {
        g1::element temp = points[i] * scalars[i];
        expected += temp;
    }
    expected = expected.normalize();
    scalar_multiplication::generate_pippenger_point_table(points, points, num_points);

    g1::element result = scalar_multiplication::pippenger(scalars, points, num_points);
    result = result.normalize();

    aligned_free(scalars);
    aligned_free(points);

    EXPECT_EQ(result == expected, true);
}

TEST(scalar_multiplication, pippenger_zero_points)
{
    fr* scalars = (fr*)aligned_alloc(32, sizeof(fr));

    g1::affine_element* points = (g1::affine_element*)aligned_alloc(32, sizeof(g1::affine_element) * 2 + 1);

    g1::element result = scalar_multiplication::pippenger(scalars, points, 0);
    EXPECT_EQ(result.is_point_at_infinity(), true);
}

TEST(scalar_multiplication, pippenger_mul_by_zero)
{
    fr* scalars = (fr*)aligned_alloc(32, sizeof(fr));

    g1::affine_element* points = (g1::affine_element*)aligned_alloc(32, sizeof(g1::affine_element) * 2 + 1);

    scalars[0] = fr::zero();
    points[0] = g1::affine_one;
    scalar_multiplication::generate_pippenger_point_table(points, points, 1);

    g1::element result = scalar_multiplication::pippenger(scalars, points, 1);
    EXPECT_EQ(result.is_point_at_infinity(), true);
}
