#pragma once

#include <array>
#include <vector>

#include "./fixed_base/fixed_base_params.hpp"
#include "barretenberg/ecc/curves/bn254/fr.hpp"

namespace bb::plookup {

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
    FIXED_BASE_0_0,
    FIXED_BASE_1_0 = FIXED_BASE_0_0 + FixedBaseParams::NUM_TABLES_PER_LO_MULTITABLE,
    FIXED_BASE_2_0 = FIXED_BASE_1_0 + FixedBaseParams::NUM_TABLES_PER_HI_MULTITABLE,
    FIXED_BASE_3_0 = FIXED_BASE_2_0 + FixedBaseParams::NUM_TABLES_PER_LO_MULTITABLE,
    HONK_DUMMY_BASIC1 = FIXED_BASE_3_0 + FixedBaseParams::NUM_TABLES_PER_HI_MULTITABLE,
    HONK_DUMMY_BASIC2,
    KECCAK_INPUT,
    KECCAK_THETA,
    KECCAK_RHO,
    KECCAK_CHI,
    KECCAK_OUTPUT,
    KECCAK_RHO_1,
    KECCAK_RHO_2,
    KECCAK_RHO_3,
    KECCAK_RHO_4,
    KECCAK_RHO_5,
    KECCAK_RHO_6,
    KECCAK_RHO_7,
    KECCAK_RHO_8,
    KECCAK_RHO_9,
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
    FIXED_BASE_LEFT_LO,
    FIXED_BASE_LEFT_HI,
    FIXED_BASE_RIGHT_LO,
    FIXED_BASE_RIGHT_HI,
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
    HONK_DUMMY_MULTI,
    KECCAK_THETA_OUTPUT,
    KECCAK_CHI_OUTPUT,
    KECCAK_FORMAT_INPUT,
    KECCAK_FORMAT_OUTPUT,
    KECCAK_NORMALIZE_AND_ROTATE,
    NUM_MULTI_TABLES = KECCAK_NORMALIZE_AND_ROTATE + 25,
};

struct MultiTable {
    // Coefficients are accumulated products of corresponding step sizes until that point
    std::vector<bb::fr> column_1_coefficients;
    std::vector<bb::fr> column_2_coefficients;
    std::vector<bb::fr> column_3_coefficients;
    MultiTableId id;
    std::vector<BasicTableId> lookup_ids;
    std::vector<uint64_t> slice_sizes;
    std::vector<bb::fr> column_1_step_sizes;
    std::vector<bb::fr> column_2_step_sizes;
    std::vector<bb::fr> column_3_step_sizes;
    typedef std::array<bb::fr, 2> table_out;
    typedef std::array<uint64_t, 2> table_in;
    std::vector<table_out (*)(table_in)> get_table_values;

  private:
    void init_step_sizes()
    {
        const size_t num_lookups = column_1_coefficients.size();
        column_1_step_sizes.emplace_back(bb::fr(1));
        column_2_step_sizes.emplace_back(bb::fr(1));
        column_3_step_sizes.emplace_back(bb::fr(1));

        std::vector<bb::fr> coefficient_inverses(column_1_coefficients.begin(), column_1_coefficients.end());
        std::copy(column_2_coefficients.begin(), column_2_coefficients.end(), std::back_inserter(coefficient_inverses));
        std::copy(column_3_coefficients.begin(), column_3_coefficients.end(), std::back_inserter(coefficient_inverses));

        bb::fr::batch_invert(&coefficient_inverses[0], num_lookups * 3);

        for (size_t i = 1; i < num_lookups; ++i) {
            column_1_step_sizes.emplace_back(column_1_coefficients[i] * coefficient_inverses[i - 1]);
            column_2_step_sizes.emplace_back(column_2_coefficients[i] * coefficient_inverses[num_lookups + i - 1]);
            column_3_step_sizes.emplace_back(column_3_coefficients[i] * coefficient_inverses[2 * num_lookups + i - 1]);
        }
    }

  public:
    MultiTable(const bb::fr& col_1_repeated_coeff,
               const bb::fr& col_2_repeated_coeff,
               const bb::fr& col_3_repeated_coeff,
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
    MultiTable(const std::vector<bb::fr>& col_1_coeffs,
               const std::vector<bb::fr>& col_2_coeffs,
               const std::vector<bb::fr>& col_3_coeffs)
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
//         std::array<bb::fr, 2> value{ bb::fr(0), bb::fr(0) };
//         bool operator<(const KeyEntry& other) const { return key < other.key; }

//         std::array<bb::fr, 3> to_sorted_list_components(const bool use_two_keys) const
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

//     bb::fr column_1_step_size = bb::fr(0);
//     bb::fr column_2_step_size = bb::fr(0);
//     bb::fr column_3_step_size = bb::fr(0);
//     std::vector<bb::fr> column_1;
//     std::vector<bb::fr> column_3;
//     std::vector<bb::fr> column_2;
//     std::vector<KeyEntry> lookup_gates;

//     std::array<bb::fr, 2> (*get_values_from_key)(const std::array<uint64_t, 2>);
// };

// struct PlookupFatKeyTable {
//     struct KeyEntry {
//         bb::fr key;
//         std::array<bb::fr, 2> values{ 0, 0 };
//         bool operator<(const KeyEntry& other) const
//         {
//             return (key.from_montgomery_form() < other.key.from_montgomery_form());
//         }

//         std::array<bb::fr, 3> to_sorted_list_components() const { return { key, values[0], values[0] }; }
//     }

//     BasicTableId id;
//     size_t table_index;
//     size_t size;
//     bool use_twin_keys;

//     bb::fr column_1_step_size = bb::fr(0);
//     bb::fr column_2_step_size = bb::fr(0);
//     bb::fr column_3_step_size = bb::fr(0);
//     std::vector<bb::fr> column_1;
//     std::vector<bb::fr> column_3;
//     std::vector<bb::fr> column_2;
//     std::vector<KeyEntry> lookup_gates;

//     std::array<bb::fr, 2> (*get_values_from_key)(const std::array<uint64_t, 2>);

// }

/**
 * @brief The structure contains the most basic table serving one function (for, example an xor table)
 *
 * @details You can find initialization example at
 * ../ultra_plonk_composer.cpp#UltraPlonkComposer::initialize_precomputed_table(..)
 *
 */
struct BasicTable {
    struct KeyEntry {
        std::array<uint256_t, 2> key{ 0, 0 };
        std::array<bb::fr, 2> value{ bb::fr(0), bb::fr(0) };
        bool operator<(const KeyEntry& other) const
        {
            return key[0] < other.key[0] || ((key[0] == other.key[0]) && key[1] < other.key[1]);
        }

        std::array<bb::fr, 3> to_sorted_list_components(const bool use_two_keys) const
        {
            return {
                bb::fr(key[0]),
                use_two_keys ? bb::fr(key[1]) : value[0],
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

    bb::fr column_1_step_size = bb::fr(0);
    bb::fr column_2_step_size = bb::fr(0);
    bb::fr column_3_step_size = bb::fr(0);
    std::vector<bb::fr> column_1;
    std::vector<bb::fr> column_3;
    std::vector<bb::fr> column_2;
    std::vector<KeyEntry> lookup_gates;

    std::array<bb::fr, 2> (*get_values_from_key)(const std::array<uint64_t, 2>);
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

} // namespace bb::plookup
