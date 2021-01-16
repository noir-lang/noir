#include "./pedersen.hpp"
#include <common/throw_or_abort.hpp>
#include <iostream>

#ifndef NO_MULTITHREADING
#include <omp.h>
#endif

namespace crypto {
namespace pedersen {
namespace {

// The number of unique base points with precomputed lookup tables
static constexpr size_t num_generators = 2048;

/**
 * The number of bits in each precomputed lookup table. Regular pedersen hashes use 254 bits, some other
 * fixed-base scalar mul subroutines (e.g. verifying schnorr signatures) use 256 bits.
 * bit_length must be 2 bits larger than the maximum hash size
 * because we represent scalar multipliers via an array of 2-bit windowed non-adjacent form entries
 *
 * When representing an n-bit integer via a WNAF with a window size of b-bits,
 * one requires a minimum of min = (n/b + 1) windows to represent any integer
 *
 * if n = 256 and b = 2, min = 129 windows = 258 bits
 */
static constexpr size_t bit_length = 258;
static constexpr size_t quad_length = bit_length / 2;

static std::array<grumpkin::g1::affine_element, num_generators> generators;
static std::vector<std::array<fixed_base_ladder, quad_length>> ladders;
static std::vector<std::array<fixed_base_ladder, quad_length>> hash_ladders;
static std::array<fixed_base_ladder, quad_length> g1_ladder;
static bool inited = false;

/**
 * Precompute ladders and hash ladders
 *
 * `ladders` contains precomputed multiples of a base point
 *
 * Each entry in `ladders` is a `fixed_base_ladder` struct, which contains a pair of points,
 * `one` and `three`
 *
 * e.g. a size-4 `ladder` over a base point `P`, will have the following structure:
 *
 *    ladder[3].one = [P]
 *    ladder[3].three = 3[P]
 *    ladder[2].one = 4[P] + [P]
 *    ladder[2].three = 4[P] + 3[P]
 *    ladder[1].one = 16[P] + [P]
 *    ladder[1].three = 16[P] + 3[P]
 *    ladder[0].one = 64[P] + [P]
 *    ladder[0].three = 64[P] + 3[P]
 *
 * i.e. for a ladder size of `n`, we have the following:
 *
 *                        n - 1 - i
 *    ladder[i].one   = (4           + 1).[P]
 *                        n - 1 - i
 *    ladder[i].three = (4           + 3).[P]
 *
 * When a fixed-base scalar multiplier is decomposed into a size-2 WNAF, each ladder entry represents
 * the positive half of a WNAF table
 *
 * `hash_ladders` are stitched together from two `ladders` objects to preserve the uniqueness of a pedersen hash.
 * If a pedersen hash input is a 256-bit scalar, using a single generator point would mean that multiple inputs would
 * hash to the same output.
 *
 * e.g. if the grumpkin curve order is `n`, then hash(x) = hash(x + n) if we use a single generator
 *
 * A `hash_ladders` first 128 entries, corresponding to the high 256-bits of a scalar, are taken from a ladder table.
 * The next 2 entries, however, are taken from a different ladder table, corresponding to a different generator
 *
 * When using `hash_ladders[i]`, the scalar is split into two segments:
 *
 *  1. The least 4 significant bits use `ladders[2 * i + 1]` (i.e. generator 2 * i + 1)
 *  2. The remaining bits use `ladders[2 * i]` (i.e. generator 2 * i)
 *
 * This is sufficient to create a unique hash for an input string that is < 2^{260}
 **/
const auto init = []() {
    generators = grumpkin::g1::derive_generators<num_generators>();
    ladders.resize(num_generators);
    hash_ladders.resize(num_generators);
    constexpr size_t first_generator_segment = quad_length - 2;
    constexpr size_t second_generator_segment = 2;
    for (size_t i = 0; i < num_generators; ++i) {
        compute_fixed_base_ladder(generators[i], &ladders[i][0]);
    }
    for (size_t i = 0; i < num_generators / 2; ++i) {
        for (size_t j = 0; j < first_generator_segment; ++j) {
            hash_ladders[i][j] = ladders[i * 2][j + (quad_length - first_generator_segment)];
        }

        for (size_t j = 0; j < second_generator_segment; ++j) {
            hash_ladders[i][j + first_generator_segment] =
                ladders[i * 2 + 1][j + (quad_length - second_generator_segment)];
        }
    }

    compute_fixed_base_ladder(grumpkin::g1::one, &g1_ladder[0]);

    inited = true;
    return 1;
};
} // namespace

void compute_fixed_base_ladder(const grumpkin::g1::affine_element& generator, fixed_base_ladder* ladder)
{
    grumpkin::g1::element* ladder_temp =
        static_cast<grumpkin::g1::element*>(aligned_alloc(64, sizeof(grumpkin::g1::element) * (quad_length * 2)));

    grumpkin::g1::element accumulator;
    accumulator = grumpkin::g1::element(generator);
    for (size_t i = 0; i < quad_length; ++i) {
        ladder_temp[i] = accumulator;
        accumulator.self_dbl();
        ladder_temp[quad_length + i] = ladder_temp[i] + accumulator;
        accumulator.self_dbl();
    }
    grumpkin::g1::element::batch_normalize(&ladder_temp[0], quad_length * 2);
    for (size_t i = 0; i < quad_length; ++i) {
        grumpkin::fq::__copy(ladder_temp[i].x, ladder[quad_length - 1 - i].one.x);
        grumpkin::fq::__copy(ladder_temp[i].y, ladder[quad_length - 1 - i].one.y);
        grumpkin::fq::__copy(ladder_temp[quad_length + i].x, ladder[quad_length - 1 - i].three.x);
        grumpkin::fq::__copy(ladder_temp[quad_length + i].y, ladder[quad_length - 1 - i].three.y);
    }

    constexpr grumpkin::fq eight_inverse = grumpkin::fq{ 8, 0, 0, 0 }.to_montgomery_form().invert();
    std::array<grumpkin::fq, quad_length> y_denominators;
    for (size_t i = 0; i < quad_length; ++i) {

        grumpkin::fq x_beta = ladder[i].one.x;
        grumpkin::fq x_gamma = ladder[i].three.x;

        grumpkin::fq y_beta = ladder[i].one.y;
        grumpkin::fq y_gamma = ladder[i].three.y;
        grumpkin::fq x_beta_times_nine = x_beta + x_beta;
        x_beta_times_nine = x_beta_times_nine + x_beta_times_nine;
        x_beta_times_nine = x_beta_times_nine + x_beta_times_nine;
        x_beta_times_nine = x_beta_times_nine + x_beta;

        grumpkin::fq x_alpha_1 = ((x_gamma - x_beta) * eight_inverse);
        grumpkin::fq x_alpha_2 = ((x_beta_times_nine - x_gamma) * eight_inverse);

        grumpkin::fq T0 = x_beta - x_gamma;
        y_denominators[i] = (((T0 + T0) + T0));

        grumpkin::fq y_alpha_1 = ((y_beta + y_beta) + y_beta) - y_gamma;
        grumpkin::fq T1 = x_gamma * y_beta;
        T1 = ((T1 + T1) + T1);
        grumpkin::fq y_alpha_2 = ((x_beta * y_gamma) - T1);

        ladder[i].q_x_1 = x_alpha_1;
        ladder[i].q_x_2 = x_alpha_2;
        ladder[i].q_y_1 = y_alpha_1;
        ladder[i].q_y_2 = y_alpha_2;
    }
    grumpkin::fq::batch_invert(&y_denominators[0], quad_length);
    for (size_t i = 0; i < quad_length; ++i) {
        ladder[i].q_y_1 *= y_denominators[i];
        ladder[i].q_y_2 *= y_denominators[i];
    }
    free(ladder_temp);
}

const fixed_base_ladder* get_ladder_internal(std::array<fixed_base_ladder, quad_length> const& ladder,
                                             const size_t num_bits)
{
    if (!inited) {
        init();
    }
    // find n, such that 2n + 1 >= num_bits
    size_t n;
    if (num_bits == 0) {
        n = 0;
    } else {
        n = (num_bits - 1) >> 1;
        if (((n << 1) + 1) < num_bits) {
            ++n;
        }
    }
    const fixed_base_ladder* result = &ladder[quad_length - n - 1];
    return result;
}

const fixed_base_ladder* get_g1_ladder(const size_t num_bits)
{
    if (!inited) {
        init();
    }
    return get_ladder_internal(g1_ladder, num_bits);
}

const fixed_base_ladder* get_ladder(const size_t generator_index, const size_t num_bits)
{
    if (!inited) {
        init();
    }
    if (generator_index >= num_generators) {
        throw_or_abort(format("Generator index out of range: ", generator_index));
    }
    return get_ladder_internal(ladders[generator_index], num_bits);
}

const fixed_base_ladder* get_hash_ladder(const size_t generator_index, const size_t num_bits)
{
    if (!inited) {
        init();
    }
    if (generator_index >= num_generators) {
        throw_or_abort(format("Generator index out of range: ", generator_index));
    }
    return get_ladder_internal(hash_ladders[generator_index], num_bits);
}

grumpkin::g1::affine_element get_generator(const size_t generator_index)
{
    if (!inited) {
        init();
    }
    if (generator_index >= num_generators) {
        throw_or_abort(format("Generator index out of range: ", generator_index));
    }
    return generators[generator_index];
}

grumpkin::g1::element hash_single(const barretenberg::fr& in, const size_t hash_index)
{
    barretenberg::fr scalar_multiplier = in.from_montgomery_form();

    constexpr size_t num_bits = 254;
    constexpr size_t num_quads_base = (num_bits - 1) >> 1;
    constexpr size_t num_quads = ((num_quads_base << 1) + 1 < num_bits) ? num_quads_base + 1 : num_quads_base;
    constexpr size_t num_wnaf_bits = (num_quads << 1) + 1;

    const crypto::pedersen::fixed_base_ladder* ladder = crypto::pedersen::get_hash_ladder(hash_index, num_bits);

    barretenberg::fr scalar_multiplier_base = scalar_multiplier.to_montgomery_form();
    if ((scalar_multiplier.data[0] & 1) == 0) {
        barretenberg::fr two = barretenberg::fr::one() + barretenberg::fr::one();
        scalar_multiplier_base = scalar_multiplier_base - two;
    }
    scalar_multiplier_base = scalar_multiplier_base.from_montgomery_form();
    uint64_t wnaf_entries[num_quads + 2] = { 0 };
    bool skew = false;
    barretenberg::wnaf::fixed_wnaf<num_wnaf_bits, 1, 2>(&scalar_multiplier_base.data[0], &wnaf_entries[0], skew, 0);

    grumpkin::g1::element accumulator;
    accumulator = grumpkin::g1::element(ladder[0].one);
    if (skew) {
        accumulator += crypto::pedersen::get_generator(hash_index * 2 + 1);
    }

    for (size_t i = 0; i < num_quads; ++i) {
        uint64_t entry = wnaf_entries[i + 1];
        ;
        const grumpkin::g1::affine_element& point_to_add =
            ((entry & 0xffffff) == 1) ? ladder[i + 1].three : ladder[i + 1].one;
        uint64_t predicate = (entry >> 31U) & 1U;
        accumulator.self_mixed_add_or_sub(point_to_add, predicate);
    }
    if (in == barretenberg::fr(0)) {
        accumulator.self_set_infinity();
    }
    return accumulator;
}

grumpkin::fq compress_native(const grumpkin::fq& left, const grumpkin::fq& right, const size_t hash_index)
{
    if (!inited) {
        init();
    }
#ifndef NO_MULTITHREADING
    grumpkin::fq in[2] = { left, right };
    grumpkin::g1::element out[2];
#pragma omp parallel num_threads(2)
    {
        size_t i = (size_t)omp_get_thread_num();
        out[i] = hash_single(in[i], hash_index + i);
    }
    grumpkin::g1::element r;
    r = out[0] + out[1];
    r = r.normalize();
    return r.x;
#else
    grumpkin::g1::element r;
    grumpkin::g1::element first = hash_single(left, hash_index);
    grumpkin::g1::element second = hash_single(right, hash_index + 1);
    r = first + second;
    r = r.normalize();
    return r.x;
#endif
}

grumpkin::g1::affine_element encrypt_native(const std::vector<grumpkin::fq>& inputs, const size_t hash_index)
{
    std::vector<grumpkin::g1::element> out(inputs.size());
    if (!inited) {
        init();
    }
#ifndef NO_MULTITHREADING
#pragma omp parallel for num_threads(inputs.size())
#endif
    for (size_t i = 0; i < inputs.size(); ++i) {
        out[i] = hash_single(inputs[i], i + hash_index);
    }

    grumpkin::g1::element r = out[0];
    for (size_t i = 1; i < inputs.size(); ++i) {
        r = out[i] + r;
    }
    return r.is_point_at_infinity() ? grumpkin::g1::affine_element(0, 0) : grumpkin::g1::affine_element(r);
}

grumpkin::fq compress_native(const std::vector<grumpkin::fq>& inputs, const size_t hash_index)
{
    return encrypt_native(inputs, hash_index).x;
}

grumpkin::fq compress_native_buffer_to_field(const std::vector<uint8_t>& input)
{
    const size_t num_bytes = input.size();
    const size_t bytes_per_element = 31;
    size_t num_elements = (num_bytes % bytes_per_element != 0) + (num_bytes / bytes_per_element);

    const auto slice = [](const std::vector<uint8_t>& data, const size_t start, const size_t slice_size) {
        uint256_t result(0);
        for (size_t i = 0; i < slice_size; ++i) {
            result = (result << uint256_t(8));
            result += uint256_t(data[i + start]);
        }
        return grumpkin::fq(result);
    };

    std::vector<grumpkin::fq> elements;
    for (size_t i = 0; i < num_elements; ++i) {
        size_t bytes_to_slice = 0;
        if (i == num_elements - 1) {
            bytes_to_slice = num_bytes - (i * bytes_per_element);
        } else {
            bytes_to_slice = bytes_per_element;
        }
        grumpkin::fq element = slice(input, i * bytes_per_element, bytes_to_slice);
        elements.emplace_back(element);
    }

    grumpkin::fq result_fq = compress_native(elements);
    return result_fq;
}

std::vector<uint8_t> compress_native(const std::vector<uint8_t>& input)
{
    const auto result_fq = compress_native_buffer_to_field(input);
    uint256_t result_u256(result_fq);
    const size_t num_bytes = input.size();

    bool is_zero = true;
    for (const auto byte : input) {
        is_zero = is_zero && (byte == static_cast<uint8_t>(0));
    }
    if (is_zero) {
        result_u256 = num_bytes;
    }
    std::vector<uint8_t> result_buffer;
    result_buffer.reserve(32);
    for (size_t i = 0; i < 32; ++i) {
        const uint64_t shift = (31 - i) * 8;
        uint256_t shifted = result_u256 >> uint256_t(shift);
        result_buffer.push_back(static_cast<uint8_t>(shifted.data[0]));
    }
    return result_buffer;
}

grumpkin::g1::affine_element compress_to_point_native(const grumpkin::fq& left,
                                                      const grumpkin::fq& right,
                                                      const size_t hash_index)
{
    if (!inited) {
        init();
    }
    grumpkin::g1::element first = hash_single(left, hash_index);
    grumpkin::g1::element second = hash_single(right, hash_index + 1);
    first = first + second;
    first = first.normalize();
    return { first.x, first.y };
}
} // namespace pedersen
} // namespace crypto
