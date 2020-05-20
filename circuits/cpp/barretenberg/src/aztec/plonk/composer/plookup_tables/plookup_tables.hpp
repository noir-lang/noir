#pragma once

#include "types.hpp"
#include "sha256.hpp"
#include "aes128.hpp"
#include "sparse.hpp"
#include "pedersen.hpp"

namespace waffle {
namespace plookup {

const PLookupMultiTable& create_table(const PLookupMultiTableId id);

PLookupReadData get_table_values(const PLookupMultiTableId id, const barretenberg::fr& key);

inline PLookupBasicTable create_basic_table(const PLookupBasicTableId id, const size_t index)
{
    switch (id) {
    case AES_SPARSE_MAP: {
        return sparse_tables::generate_sparse_table_with_rotation<9, 8, 0>(AES_SPARSE_MAP, index);
    }
    case AES_SBOX_MAP: {
        return aes128_tables::generate_aes_sbox_table(AES_SBOX_MAP, index);
    }
    case AES_SPARSE_NORMALIZE: {
        return aes128_tables::generate_aes_sparse_normalization_table(AES_SPARSE_NORMALIZE, index);
    }
    case SHA256_WITNESS_NORMALIZE: {
        return sha256_tables::generate_witness_extension_normalization_table(SHA256_WITNESS_NORMALIZE, index);
    }
    case SHA256_WITNESS_SLICE_3: {
        return sparse_tables::generate_sparse_table_with_rotation<16, 3, 0>(SHA256_WITNESS_SLICE_3, index);
    }
    case SHA256_WITNESS_SLICE_7_ROTATE_4: {
        return sparse_tables::generate_sparse_table_with_rotation<16, 7, 4>(SHA256_WITNESS_SLICE_7_ROTATE_4, index);
    }
    case SHA256_WITNESS_SLICE_8_ROTATE_7: {
        return sparse_tables::generate_sparse_table_with_rotation<16, 8, 7>(SHA256_WITNESS_SLICE_8_ROTATE_7, index);
    }
    case SHA256_WITNESS_SLICE_14_ROTATE_1: {
        return sparse_tables::generate_sparse_table_with_rotation<16, 14, 1>(SHA256_WITNESS_SLICE_14_ROTATE_1, index);
    }
    case SHA256_CH_NORMALIZE: {
        return sha256_tables::generate_choose_normalization_table(SHA256_CH_NORMALIZE, index);
    }
    case SHA256_MAJ_NORMALIZE: {
        return sha256_tables::generate_majority_normalization_table(SHA256_MAJ_NORMALIZE, index);
    }
    case SHA256_BASE28: {
        return sparse_tables::generate_sparse_table_with_rotation<28, 11, 0>(SHA256_BASE28, index);
    }
    case SHA256_BASE28_ROTATE6: {
        return sparse_tables::generate_sparse_table_with_rotation<28, 11, 6>(SHA256_BASE28_ROTATE6, index);
    }
    case SHA256_BASE28_ROTATE3: {
        return sparse_tables::generate_sparse_table_with_rotation<28, 11, 3>(SHA256_BASE28_ROTATE3, index);
    }
    case SHA256_BASE16: {
        return sparse_tables::generate_sparse_table_with_rotation<16, 11, 0>(SHA256_BASE16, index);
    }
    case SHA256_BASE16_ROTATE2: {
        return sparse_tables::generate_sparse_table_with_rotation<16, 11, 2>(SHA256_BASE16_ROTATE2, index);
    }
    case PEDERSEN_1_10: {
        return pedersen_tables::generate_pedersen_table(0, 9, PEDERSEN_1_10, index);
    }
    case PEDERSEN_1_9: {
        return pedersen_tables::generate_pedersen_table(0, 8, PEDERSEN_1_9, index);
    }
    case PEDERSEN_1_8: {
        return pedersen_tables::generate_pedersen_table(0, 7, PEDERSEN_1_8, index);
    }
    case PEDERSEN_1_7: {
        return pedersen_tables::generate_pedersen_table(0, 6, PEDERSEN_1_7, index);
    }
    case PEDERSEN_1_6: {
        return pedersen_tables::generate_pedersen_table(0, 5, PEDERSEN_1_6, index);
    }
    case PEDERSEN_1_5: {
        return pedersen_tables::generate_pedersen_table(0, 4, PEDERSEN_1_5, index);
    }
    case PEDERSEN_1_4: {
        return pedersen_tables::generate_pedersen_table(0, 3, PEDERSEN_1_4, index);
    }
    case PEDERSEN_1_3: {
        return pedersen_tables::generate_pedersen_table(0, 2, PEDERSEN_1_3, index);
    }
    case PEDERSEN_1_2: {
        return pedersen_tables::generate_pedersen_table(0, 1, PEDERSEN_1_2, index);
    }
    case PEDERSEN_1_1: {
        return pedersen_tables::generate_pedersen_table(0, 0, PEDERSEN_1_1, index);
    }
    case PEDERSEN_2_10: {
        return pedersen_tables::generate_pedersen_table(1, 9, PEDERSEN_2_10, index);
    }
    case PEDERSEN_2_9: {
        return pedersen_tables::generate_pedersen_table(1, 8, PEDERSEN_2_9, index);
    }
    case PEDERSEN_2_8: {
        return pedersen_tables::generate_pedersen_table(1, 7, PEDERSEN_2_8, index);
    }
    case PEDERSEN_2_7: {
        return pedersen_tables::generate_pedersen_table(1, 6, PEDERSEN_2_7, index);
    }
    case PEDERSEN_2_6: {
        return pedersen_tables::generate_pedersen_table(1, 5, PEDERSEN_2_6, index);
    }
    case PEDERSEN_2_5: {
        return pedersen_tables::generate_pedersen_table(1, 4, PEDERSEN_2_5, index);
    }
    case PEDERSEN_2_4: {
        return pedersen_tables::generate_pedersen_table(1, 3, PEDERSEN_2_4, index);
    }
    case PEDERSEN_2_3: {
        return pedersen_tables::generate_pedersen_table(1, 2, PEDERSEN_2_3, index);
    }
    case PEDERSEN_2_2: {
        return pedersen_tables::generate_pedersen_table(1, 1, PEDERSEN_2_2, index);
    }
    case PEDERSEN_2_1: {
        return pedersen_tables::generate_pedersen_table(1, 0, PEDERSEN_2_1, index);
    }
    default: {
        throw;
    }
    }
}
} // namespace plookup
} // namespace waffle