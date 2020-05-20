#pragma once

#include <vector>
#include <array>

#include <ecc/curves/bn254/fr.hpp>

namespace waffle {
enum PLookupBasicTableId {
    XOR,
    AND,
    PEDERSEN,
    AES_SPARSE_MAP,
    AES_SBOX_MAP,
    AES_SPARSE_NORMALIZE,
    SHA256_WITNESS_NORMALIZE,
    SHA256_WITNESS_SLICE_3,
    SHA256_WITNESS_SLICE_7_ROTATE_4,
    SHA256_WITNESS_SLICE_8_ROTATE_7,
    SHA256_WITNESS_SLICE_14_ROTATE_1,
    SHA256_CH_NORMALIZE,
    SHA256_MAJ_NORMALIZE,
    SHA256_BASE28,
    SHA256_BASE28_ROTATE6,
    SHA256_BASE28_ROTATE3,
    SHA256_BASE16,
    SHA256_BASE16_ROTATE2,
    SHA256_BASE16_ROTATE6,
    SHA256_BASE16_ROTATE7,
    SHA256_BASE16_ROTATE8,
    PEDERSEN_1_10,
    PEDERSEN_1_9,
    PEDERSEN_1_8,
    PEDERSEN_1_7,
    PEDERSEN_1_6,
    PEDERSEN_1_5,
    PEDERSEN_1_4,
    PEDERSEN_1_3,
    PEDERSEN_1_2,
    PEDERSEN_1_1,
    PEDERSEN_2_10,
    PEDERSEN_2_9,
    PEDERSEN_2_8,
    PEDERSEN_2_7,
    PEDERSEN_2_6,
    PEDERSEN_2_5,
    PEDERSEN_2_4,
    PEDERSEN_2_3,
    PEDERSEN_2_2,
    PEDERSEN_2_1,
};

enum PLookupMultiTableId {
    SHA256_CH_INPUT = 0,
    SHA256_CH_OUTPUT = 1,
    SHA256_MAJ_INPUT = 2,
    SHA256_MAJ_OUTPUT = 3,
    SHA256_WITNESS_INPUT = 4,
    SHA256_WITNESS_OUTPUT = 5,
    AES_NORMALIZE = 6,
    AES_INPUT = 7,
    AES_SBOX = 8,
    PEDERSEN_1 = 9,
    PEDERSEN_2 = 10,
    NUM_MULTI_TABES = 11,
};

struct PLookupMultiTable {
    std::vector<barretenberg::fr> column_1_coefficients;
    std::vector<barretenberg::fr> column_2_coefficients;
    std::vector<barretenberg::fr> column_3_coefficients;
    PLookupMultiTableId id;
    std::vector<PLookupBasicTableId> lookup_ids;
    std::vector<uint64_t> slice_sizes;
    std::vector<barretenberg::fr> column_1_step_sizes;
    std::vector<barretenberg::fr> column_2_step_sizes;
    std::vector<barretenberg::fr> column_3_step_sizes;
    typedef std::array<barretenberg::fr, 2> table_out;
    typedef std::array<uint64_t, 2> table_in;
    std::vector<table_out (*)(table_in)> get_table_values;

  private:
    void init_step_sizes()
    {
        const size_t num_lookups = column_1_coefficients.size();
        column_1_step_sizes.emplace_back(barretenberg::fr(1));
        column_2_step_sizes.emplace_back(barretenberg::fr(1));
        column_3_step_sizes.emplace_back(barretenberg::fr(1));

        std::vector<barretenberg::fr> coefficient_inverses(column_1_coefficients.begin(), column_1_coefficients.end());
        std::copy(column_2_coefficients.begin(), column_2_coefficients.end(), std::back_inserter(coefficient_inverses));
        std::copy(column_3_coefficients.begin(), column_3_coefficients.end(), std::back_inserter(coefficient_inverses));

        barretenberg::fr::batch_invert(&coefficient_inverses[0], num_lookups * 3);

        for (size_t i = 1; i < num_lookups; ++i) {
            column_1_step_sizes.emplace_back(column_1_coefficients[i] * coefficient_inverses[i - 1]);
            column_2_step_sizes.emplace_back(column_2_coefficients[i] * coefficient_inverses[num_lookups + i - 1]);
            column_3_step_sizes.emplace_back(column_3_coefficients[i] * coefficient_inverses[2 * num_lookups + i - 1]);
        }
    }

  public:
    PLookupMultiTable(const barretenberg::fr& col_1_repeated_coeff,
                      const barretenberg::fr& col_2_repeated_coeff,
                      const barretenberg::fr& col_3_repeated_coeff,
                      const size_t num_lookups)
    {
        column_1_coefficients.emplace_back(1);
        column_2_coefficients.emplace_back(1);
        column_3_coefficients.emplace_back(1);

        for (size_t i = 0; i < num_lookups; ++i) {
            column_1_coefficients.emplace_back(column_1_coefficients.back() * col_1_repeated_coeff);
            column_2_coefficients.emplace_back(column_2_coefficients.back() * col_2_repeated_coeff);
            column_3_coefficients.emplace_back(column_3_coefficients.back() * col_3_repeated_coeff);
        }
        init_step_sizes();
    }
    PLookupMultiTable(const std::vector<barretenberg::fr>& col_1_coeffs,
                      const std::vector<barretenberg::fr>& col_2_coeffs,
                      const std::vector<barretenberg::fr>& col_3_coeffs)
        : column_1_coefficients(col_1_coeffs)
        , column_2_coefficients(col_2_coeffs)
        , column_3_coefficients(col_3_coeffs)
    {
        init_step_sizes();
    }

    PLookupMultiTable(){};
    PLookupMultiTable(const PLookupMultiTable& other) = default;
    PLookupMultiTable(PLookupMultiTable&& other) = default;

    PLookupMultiTable& operator=(const PLookupMultiTable& other) = default;
    PLookupMultiTable& operator=(PLookupMultiTable&& other) = default;
};

// struct PLookupLargeKeyTable {
//     struct KeyEntry {
//         uint256_t key;
//         std::array<barretenberg::fr, 2> value{ barretenberg::fr(0), barretenberg::fr(0) };
//         bool operator<(const KeyEntry& other) const { return key < other.key; }

//         std::array<barretenberg::fr, 3> to_sorted_list_components(const bool use_two_keys) const
//         {
//             return {
//                 key[0],
//                 value[0],
//                 value[1],
//             };
//         }
//     };

//     PLookupBasicTableId id;
//     size_t table_index;
//     size_t size;
//     bool use_twin_keys;

//     barretenberg::fr column_1_step_size = barretenberg::fr(0);
//     barretenberg::fr column_2_step_size = barretenberg::fr(0);
//     barretenberg::fr column_3_step_size = barretenberg::fr(0);
//     std::vector<barretenberg::fr> column_1;
//     std::vector<barretenberg::fr> column_3;
//     std::vector<barretenberg::fr> column_2;
//     std::vector<KeyEntry> lookup_gates;

//     std::array<barretenberg::fr, 2> (*get_values_from_key)(const std::array<uint64_t, 2>);
// };

// struct PLookupFatKeyTable {
//     struct KeyEntry {
//         barretenberg::fr key;
//         std::array<barretenberg::fr, 2> values{ 0, 0 };
//         bool operator<(const KeyEntry& other) const
//         {
//             return (key.from_montgomery_form() < other.key.from_montgomery_form());
//         }

//         std::array<barretenberg::fr, 3> to_sorted_list_components() const { return { key, values[0], values[0] }; }
//     }

//     PLookupBasicTableId id;
//     size_t table_index;
//     size_t size;
//     bool use_twin_keys;

//     barretenberg::fr column_1_step_size = barretenberg::fr(0);
//     barretenberg::fr column_2_step_size = barretenberg::fr(0);
//     barretenberg::fr column_3_step_size = barretenberg::fr(0);
//     std::vector<barretenberg::fr> column_1;
//     std::vector<barretenberg::fr> column_3;
//     std::vector<barretenberg::fr> column_2;
//     std::vector<KeyEntry> lookup_gates;

//     std::array<barretenberg::fr, 2> (*get_values_from_key)(const std::array<uint64_t, 2>);

// }

struct PLookupBasicTable {
    struct KeyEntry {
        std::array<uint256_t, 2> key{ 0, 0 };
        std::array<barretenberg::fr, 2> value{ barretenberg::fr(0), barretenberg::fr(0) };
        bool operator<(const KeyEntry& other) const
        {
            return key[0] < other.key[0] || ((key[0] == other.key[0]) && key[1] < other.key[1]);
        }

        std::array<barretenberg::fr, 3> to_sorted_list_components(const bool use_two_keys) const
        {
            return {
                barretenberg::fr(key[0]),
                use_two_keys ? barretenberg::fr(key[1]) : value[0],
                use_two_keys ? value[0] : value[1],
            };
        }
    };

    PLookupBasicTableId id;
    size_t table_index;
    size_t size;
    bool use_twin_keys;

    barretenberg::fr column_1_step_size = barretenberg::fr(0);
    barretenberg::fr column_2_step_size = barretenberg::fr(0);
    barretenberg::fr column_3_step_size = barretenberg::fr(0);
    std::vector<barretenberg::fr> column_1;
    std::vector<barretenberg::fr> column_3;
    std::vector<barretenberg::fr> column_2;
    std::vector<KeyEntry> lookup_gates;

    std::array<barretenberg::fr, 2> (*get_values_from_key)(const std::array<uint64_t, 2>);
};

struct PLookupReadData {
    std::vector<PLookupBasicTable::KeyEntry> key_entries;

    std::vector<barretenberg::fr> column_1_accumulator_values;
    std::vector<barretenberg::fr> column_2_accumulator_values;
    std::vector<barretenberg::fr> column_3_accumulator_values;
};
} // namespace waffle