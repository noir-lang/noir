#pragma once

#include <vector>
#include <array>

#include <ecc/curves/bn254/fr.hpp>

namespace plookup {

enum BasicTableId {
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
    UINT_XOR_ROTATE0,
    UINT_AND_ROTATE0,
    BN254_XLO_BASIC,
    BN254_XHI_BASIC,
    BN254_YLO_BASIC,
    BN254_YHI_BASIC,
    BN254_XYPRIME_BASIC,
    BN254_XLO_ENDO_BASIC,
    BN254_XHI_ENDO_BASIC,
    BN254_XYPRIME_ENDO_BASIC,
    SECP256K1_XLO_BASIC,
    SECP256K1_XHI_BASIC,
    SECP256K1_YLO_BASIC,
    SECP256K1_YHI_BASIC,
    SECP256K1_XYPRIME_BASIC,
    SECP256K1_XLO_ENDO_BASIC,
    SECP256K1_XHI_ENDO_BASIC,
    SECP256K1_XYPRIME_ENDO_BASIC,
    BLAKE_XOR_ROTATE0,
    BLAKE_XOR_ROTATE0_SLICE5_MOD4,
    BLAKE_XOR_ROTATE1,
    BLAKE_XOR_ROTATE2,
    BLAKE_XOR_ROTATE4,
    PEDERSEN_29_SMALL,
    PEDERSEN_28,
    PEDERSEN_27,
    PEDERSEN_26,
    PEDERSEN_25,
    PEDERSEN_24,
    PEDERSEN_23,
    PEDERSEN_22,
    PEDERSEN_21,
    PEDERSEN_20,
    PEDERSEN_19,
    PEDERSEN_18,
    PEDERSEN_17,
    PEDERSEN_16,
    PEDERSEN_15,
    PEDERSEN_14_SMALL,
    PEDERSEN_13,
    PEDERSEN_12,
    PEDERSEN_11,
    PEDERSEN_10,
    PEDERSEN_9,
    PEDERSEN_8,
    PEDERSEN_7,
    PEDERSEN_6,
    PEDERSEN_5,
    PEDERSEN_4,
    PEDERSEN_3,
    PEDERSEN_2,
    PEDERSEN_1,
    PEDERSEN_0,
    PEDERSEN_IV_BASE,
};

enum MultiTableId {
    SHA256_CH_INPUT,
    SHA256_CH_OUTPUT,
    SHA256_MAJ_INPUT,
    SHA256_MAJ_OUTPUT,
    SHA256_WITNESS_INPUT,
    SHA256_WITNESS_OUTPUT,
    AES_NORMALIZE,
    AES_INPUT,
    AES_SBOX,
    PEDERSEN_LEFT_HI,
    PEDERSEN_LEFT_LO,
    PEDERSEN_RIGHT_HI,
    PEDERSEN_RIGHT_LO,
    UINT32_XOR,
    UINT32_AND,
    BN254_XLO,
    BN254_XHI,
    BN254_YLO,
    BN254_YHI,
    BN254_XYPRIME,
    BN254_XLO_ENDO,
    BN254_XHI_ENDO,
    BN254_XYPRIME_ENDO,
    SECP256K1_XLO,
    SECP256K1_XHI,
    SECP256K1_YLO,
    SECP256K1_YHI,
    SECP256K1_XYPRIME,
    SECP256K1_XLO_ENDO,
    SECP256K1_XHI_ENDO,
    SECP256K1_XYPRIME_ENDO,
    BLAKE_XOR,
    BLAKE_XOR_ROTATE_16,
    BLAKE_XOR_ROTATE_8,
    BLAKE_XOR_ROTATE_7,
    PEDERSEN_IV,
    NUM_MULTI_TABLES,
};

struct MultiTable {
    std::vector<barretenberg::fr> column_1_coefficients;
    std::vector<barretenberg::fr> column_2_coefficients;
    std::vector<barretenberg::fr> column_3_coefficients;
    MultiTableId id;
    std::vector<BasicTableId> lookup_ids;
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
    MultiTable(const barretenberg::fr& col_1_repeated_coeff,
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
    MultiTable(const std::vector<barretenberg::fr>& col_1_coeffs,
               const std::vector<barretenberg::fr>& col_2_coeffs,
               const std::vector<barretenberg::fr>& col_3_coeffs)
        : column_1_coefficients(col_1_coeffs)
        , column_2_coefficients(col_2_coeffs)
        , column_3_coefficients(col_3_coeffs)
    {
        init_step_sizes();
    }

    MultiTable(){};
    MultiTable(const MultiTable& other) = default;
    MultiTable(MultiTable&& other) = default;

    MultiTable& operator=(const MultiTable& other) = default;
    MultiTable& operator=(MultiTable&& other) = default;
};

// struct PlookupLargeKeyTable {
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

//     BasicTableId id;
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

// struct PlookupFatKeyTable {
//     struct KeyEntry {
//         barretenberg::fr key;
//         std::array<barretenberg::fr, 2> values{ 0, 0 };
//         bool operator<(const KeyEntry& other) const
//         {
//             return (key.from_montgomery_form() < other.key.from_montgomery_form());
//         }

//         std::array<barretenberg::fr, 3> to_sorted_list_components() const { return { key, values[0], values[0] }; }
//     }

//     BasicTableId id;
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

/**
 * @brief The structure contains the most basic table serving one function (for, example an xor table)
 *
 * @details You can find initialization example at ../ultra_composer.cpp#UltraComposer::initialize_precomputed_table(..)
 *
 */
struct BasicTable {
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

    // Unique id of the table which is used to look it up, when we need its functionality. One of BasicTableId enum
    BasicTableId id;
    size_t table_index;
    // The size of the table
    size_t size;
    // This means that we are using two inputs to look up stuff, not translate a single entry into another one.
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

enum ColumnIdx { C1, C2, C3 };

/**
 * @brief Container type for lookup table reads.
 *
 * @tparam DataType: a native or stdlib field type, or the witness index type uint32_t
 *
 * @details We us this approach to indexing, using enums, rather than to make member variables column_i, to minimize
 * code changes; both non-const and const versions are in use.
 *
 * The inner index, i.e., the index of each vector v in the array `columns`, could also be treated as an enum, but that
 * might be messier. Note that v[0] represents a full accumulated sum, v[1] represents one step before that,
 * and so on. See the documentation of the native version of get_lookup_accumulators.
 *
 */
template <class DataType> class ReadData {
  public:
    ReadData() = default;
    std::vector<DataType>& operator[](ColumnIdx idx) { return columns[static_cast<size_t>(idx)]; };
    const std::vector<DataType>& operator[](ColumnIdx idx) const { return columns[static_cast<size_t>(idx)]; };

    std::vector<BasicTable::KeyEntry> key_entries;

  private:
    std::array<std::vector<DataType>, 3> columns;
};

} // namespace plookup