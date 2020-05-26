#include "./sidon_pedersen.hpp"

#include "./sidon_set/sidon_set.hpp"

#include <ecc/curves/grumpkin/grumpkin.hpp>

namespace crypto {
namespace pedersen {
namespace sidon {
namespace {

static std::vector<uint64_t> sidon_set;
static std::array<std::vector<grumpkin::g1::affine_element>, NUM_PEDERSEN_TABLES> sidon_pedersen_tables;
static std::array<grumpkin::g1::affine_element, NUM_PEDERSEN_TABLES> generators;
static bool inited = false;

void init_single_sidon_lookup_table(const size_t index)
{
    std::vector<grumpkin::g1::element> temp;
    temp.reserve(PEDERSEN_TABLE_SIZE);
    sidon_pedersen_tables[index].reserve(PEDERSEN_TABLE_SIZE);

    const auto& generator = generators[index];
    for (size_t i = 0; i < PEDERSEN_TABLE_SIZE; ++i) {
        temp.emplace_back(generator * grumpkin::fr(sidon_set[i]));
    }
    grumpkin::g1::element::batch_normalize(&temp[0], PEDERSEN_TABLE_SIZE);

    for (const auto& element : temp) {
        sidon_pedersen_tables[index].emplace_back(element);
    }
}

void init()
{
    if (inited) {
        return;
    }
    sidon_set = compute_sidon_set<PEDERSEN_TABLE_SIZE>();
    generators = grumpkin::g1::derive_generators<NUM_PEDERSEN_TABLES>();
    for (size_t i = 0; i < NUM_PEDERSEN_TABLES; ++i) {
        init_single_sidon_lookup_table(i);
    }
    inited = true;
}
} // namespace

grumpkin::g1::affine_element get_table_generator(const size_t table_index)
{
    ASSERT(table_index < NUM_PEDERSEN_TABLES);
    init();
    return generators[table_index];
}

const std::vector<uint64_t>& get_sidon_set()
{
    init();
    return sidon_set;
}

const std::vector<grumpkin::g1::affine_element>& get_table(const size_t table_index)
{
    init();
    return sidon_pedersen_tables[table_index];
}

grumpkin::g1::element compress_single(const grumpkin::fq& input, const bool parity)
{
    init();
    uint256_t bits(input);

    // N.B. NUM_PEDERSEN_TABLES must be divisible by 2 for this to work as-is.
    constexpr size_t num_rounds = NUM_PEDERSEN_TABLES / 2;

    constexpr uint64_t table_mask = PEDERSEN_TABLE_SIZE - 1;

    size_t table_index_offset = parity ? (NUM_PEDERSEN_TABLES / 2) : 0;

    std::array<grumpkin::g1::element, 3> accumulators;
    for (size_t i = 0; i < num_rounds; ++i) {
        const uint64_t slice_a = (bits.data[0] & table_mask);
        bits >>= BITS_PER_TABLE;
        const uint64_t slice_b = (bits.data[0] & table_mask);
        bits >>= BITS_PER_TABLE;
        const uint64_t slice_c = (bits.data[0] & table_mask);

        // P = g * a + g * (b * lambda) + g * (c * (lambda + 1))

        const size_t index = table_index_offset + i;
        if (i == 0) {
            accumulators = {
                sidon_pedersen_tables[index][slice_a],
                sidon_pedersen_tables[index][slice_b],
                sidon_pedersen_tables[index][slice_c],
            };
        } else {
            accumulators[0] += sidon_pedersen_tables[index][slice_a];
            accumulators[1] += sidon_pedersen_tables[index][slice_b];
            accumulators[2] += sidon_pedersen_tables[index][slice_c];
        }
        bits >>= (BITS_PER_TABLE);
    }

    accumulators[1].x *= grumpkin::fq::beta();
    accumulators[2].x *= grumpkin::fq::beta().sqr();
    accumulators[2].y = -accumulators[2].y;

    return accumulators[0] + accumulators[1] + accumulators[2];
}

grumpkin::fq compress(const grumpkin::fq& left, const grumpkin::fq& right)
{
    grumpkin::g1::affine_element result =
        grumpkin::g1::affine_element(compress_single(left, false) + compress_single(right, true));
    return result.x;
}

} // namespace sidon
} // namespace pedersen
} // namespace crypto