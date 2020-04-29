#pragma once

#include <vector>
#include <array>

#include <ecc/curves/bn254/fr.hpp>

namespace waffle {
enum PLookupTableId {
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
};

enum PLookupMultiTableId {
    SHA256_CH_INPUT,
    SHA256_CH_OUTPUT,
    SHA256_MAJ_INPUT,
    SHA256_MAJ_OUTPUT,
    SHA256_WITNESS_INPUT,
    SHA256_WITNESS_OUTPUT,
};

struct PLookupMultiTable {
    std::vector<PLookupTableId> lookup_ids;
    std::vector<barretenberg::fr> column_1_coefficients;
    std::vector<barretenberg::fr> column_2_coefficients;
    std::vector<barretenberg::fr> column_3_coefficients;
    std::vector<uint64_t> slice_sizes;
    PLookupMultiTableId id;
};

struct PLookupTable {
    struct KeyEntry {
        std::array<uint64_t, 2> key{ 0, 0 };
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

    PLookupTableId id;
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
    std::vector<PLookupTable::KeyEntry> key_entries;

    std::vector<barretenberg::fr> column_1_step_sizes;
    std::vector<barretenberg::fr> column_2_step_sizes;
    std::vector<barretenberg::fr> column_3_step_sizes;

    std::vector<barretenberg::fr> column_1_accumulator_values;
    std::vector<barretenberg::fr> column_2_accumulator_values;
    std::vector<barretenberg::fr> column_3_accumulator_values;
};
} // namespace waffle