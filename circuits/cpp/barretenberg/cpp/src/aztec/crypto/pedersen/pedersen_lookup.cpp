#include "./pedersen_lookup.hpp"
#include "./convert_buffer_to_field.hpp"

#include <ecc/curves/grumpkin/grumpkin.hpp>

namespace crypto {
namespace pedersen {
namespace lookup {
namespace {

static std::array<std::vector<grumpkin::g1::affine_element>, NUM_PEDERSEN_TABLES> pedersen_tables;
static std::vector<grumpkin::g1::affine_element> pedersen_iv_table;
static std::array<grumpkin::g1::affine_element, NUM_PEDERSEN_TABLES> generators;
static bool inited = false;

void init_single_lookup_table(const size_t index)
{
    std::vector<grumpkin::g1::element> temp;
    temp.reserve(PEDERSEN_TABLE_SIZE);
    pedersen_tables[index].reserve(PEDERSEN_TABLE_SIZE);

    const auto& generator = generators[index];
    for (size_t i = 0; i < PEDERSEN_TABLE_SIZE; ++i) {
        temp.emplace_back(generator * grumpkin::fr(i + 1));
    }
    grumpkin::g1::element::batch_normalize(&temp[0], PEDERSEN_TABLE_SIZE);

    for (const auto& element : temp) {
        pedersen_tables[index].emplace_back(element);
    }
}

void init_small_lookup_table(const size_t index)
{
    std::vector<grumpkin::g1::element> temp;
    temp.reserve(PEDERSEN_SMALL_TABLE_SIZE);
    pedersen_tables[index].reserve(PEDERSEN_SMALL_TABLE_SIZE);

    const auto& generator = generators[index];
    for (size_t i = 0; i < PEDERSEN_SMALL_TABLE_SIZE; ++i) {
        temp.emplace_back(generator * grumpkin::fr(i + 1));
    }
    grumpkin::g1::element::batch_normalize(&temp[0], PEDERSEN_SMALL_TABLE_SIZE);

    for (const auto& element : temp) {
        pedersen_tables[index].emplace_back(element);
    }
}

void init_iv_lookup_table()
{
    std::vector<grumpkin::g1::element> temp;
    temp.reserve(PEDERSEN_IV_TABLE_SIZE);
    pedersen_iv_table.reserve(PEDERSEN_IV_TABLE_SIZE);

    for (size_t i = 0; i < PEDERSEN_IV_TABLE_SIZE; ++i) {
        temp.emplace_back(grumpkin::g1::affine_one * grumpkin::fr(i + 1));
    }
    grumpkin::g1::element::batch_normalize(&temp[0], PEDERSEN_IV_TABLE_SIZE);

    for (const auto& element : temp) {
        pedersen_iv_table.emplace_back(element);
    }
}

void init()
{
    ASSERT(BITS_PER_TABLE < BITS_OF_BETA);
    ASSERT(BITS_PER_TABLE + BITS_OF_BETA < BITS_ON_CURVE);
    if (inited) {
        return;
    }
    generators = grumpkin::g1::derive_generators<NUM_PEDERSEN_TABLES>();
    const size_t first_half = (NUM_PEDERSEN_TABLES >> 1) - 1;
    for (size_t i = 0; i < first_half; ++i) {
        init_single_lookup_table(i);
    }
    init_small_lookup_table(first_half);
    for (size_t i = 0; i < first_half; ++i) {
        init_single_lookup_table(i + first_half + 1);
    }
    init_small_lookup_table(2 * first_half + 1);
    init_iv_lookup_table();
    inited = true;
}
} // namespace

grumpkin::g1::affine_element get_table_generator(const size_t table_index)
{
    ASSERT(table_index < NUM_PEDERSEN_TABLES);
    init();
    return generators[table_index];
}

const std::vector<grumpkin::g1::affine_element>& get_table(const size_t table_index)
{
    init();
    return pedersen_tables[table_index];
}

const std::vector<grumpkin::g1::affine_element>& get_iv_table()
{
    init();
    return pedersen_iv_table;
}

grumpkin::g1::element hash_single(const grumpkin::fq& input, const bool parity)
{
    init();
    uint256_t bits(input);

    // N.B. NUM_PEDERSEN_TABLES must be divisible by 2 for this to work as-is.
    constexpr size_t num_rounds = NUM_PEDERSEN_TABLES / 2;
    constexpr uint64_t table_mask = PEDERSEN_TABLE_SIZE - 1;
    size_t table_index_offset = parity ? (NUM_PEDERSEN_TABLES / 2) : 0;

    std::array<grumpkin::g1::element, 2> accumulators;
    for (size_t i = 0; i < num_rounds; ++i) {
        const uint64_t slice_a = (bits.data[0] & table_mask);
        bits >>= BITS_PER_TABLE;
        const uint64_t slice_b = (bits.data[0] & table_mask);

        // P = g * (b) + g * (a * lambda)
        const size_t index = table_index_offset + i;
        if (i == 0) {
            accumulators = {
                pedersen_tables[index][static_cast<size_t>(slice_a)],
                pedersen_tables[index][static_cast<size_t>(slice_b)],
            };
        } else {
            accumulators[0] += pedersen_tables[index][static_cast<size_t>(slice_a)];
            if (i < (num_rounds - 1)) {
                accumulators[1] += pedersen_tables[index][static_cast<size_t>(slice_b)];
            }
        }
        bits >>= (BITS_PER_TABLE);
    }

    grumpkin::fq beta = grumpkin::fq::cube_root_of_unity();
    accumulators[0].x *= beta;

    return accumulators[0] + accumulators[1];
}

grumpkin::fq hash_pair(const grumpkin::fq& left, const grumpkin::fq& right)
{
    grumpkin::g1::affine_element result =
        grumpkin::g1::affine_element(hash_single(left, false) + hash_single(right, true));
    return result.x;
}

grumpkin::g1::element merkle_damgard_compress(const std::vector<grumpkin::fq>& inputs, const size_t iv)
{
    if (inputs.size() == 0) {
        auto result = grumpkin::g1::affine_one;
        result.self_set_infinity();
        return result;
    }
    init();
    const size_t num_inputs = inputs.size();

    grumpkin::fq result = (pedersen_iv_table[iv]).x;
    for (size_t i = 0; i < num_inputs; i++) {
        result = hash_pair(result, inputs[i]);
    }

    return (hash_single(result, false) + hash_single(grumpkin::fq(num_inputs), true));
}

grumpkin::g1::affine_element commit_native(const std::vector<grumpkin::fq>& inputs, const size_t hash_index)
{
    return grumpkin::g1::affine_element(merkle_damgard_compress(inputs, hash_index));
}

grumpkin::fq compress_native(const std::vector<grumpkin::fq>& inputs, const size_t hash_index)
{
    return commit_native(inputs, hash_index).x;
}

grumpkin::fq compress_native_buffer_to_field(const std::vector<uint8_t>& input)
{
    const auto elements = convert_buffer_to_field(input);
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
} // namespace lookup
} // namespace pedersen
} // namespace crypto