#pragma once

#include "types.hpp"
#include "sha256.hpp"
#include "aes128.hpp"
#include "sparse.hpp"
#include "pedersen.hpp"
#include "uint.hpp"

namespace waffle {
namespace plookup {

const PlookupMultiTable& create_table(const PlookupMultiTableId id);

PlookupReadData get_table_values(const PlookupMultiTableId id,
                                 const barretenberg::fr& key_a,
                                 const barretenberg::fr& key_b = 0,
                                 const bool is_2_to_1_map = false);

inline PlookupBasicTable create_basic_table(const PlookupBasicTableId id, const size_t index)
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
    case PEDERSEN_0: {
        return pedersen_tables::generate_sidon_pedersen_table<0>(PEDERSEN_0, index);
    }
    case PEDERSEN_1: {
        return pedersen_tables::generate_sidon_pedersen_table<1>(PEDERSEN_1, index);
    }
    case PEDERSEN_2: {
        return pedersen_tables::generate_sidon_pedersen_table<2>(PEDERSEN_2, index);
    }
    case PEDERSEN_3: {
        return pedersen_tables::generate_sidon_pedersen_table<3>(PEDERSEN_3, index);
    }
    case PEDERSEN_4: {
        return pedersen_tables::generate_sidon_pedersen_table<4>(PEDERSEN_4, index);
    }
    case PEDERSEN_5: {
        return pedersen_tables::generate_sidon_pedersen_table<5>(PEDERSEN_5, index);
    }
    case PEDERSEN_6: {
        return pedersen_tables::generate_sidon_pedersen_table<6>(PEDERSEN_6, index);
    }
    case PEDERSEN_7: {
        return pedersen_tables::generate_sidon_pedersen_table<7>(PEDERSEN_7, index);
    }
    case PEDERSEN_8: {
        return pedersen_tables::generate_sidon_pedersen_table<8>(PEDERSEN_8, index);
    }
    case PEDERSEN_9: {
        return pedersen_tables::generate_sidon_pedersen_table<9>(PEDERSEN_9, index);
    }
    case PEDERSEN_10: {
        return pedersen_tables::generate_sidon_pedersen_table<10>(PEDERSEN_10, index);
    }
    case PEDERSEN_11: {
        return pedersen_tables::generate_sidon_pedersen_table<11>(PEDERSEN_11, index);
    }
    case PEDERSEN_12: {
        return pedersen_tables::generate_sidon_pedersen_table<12>(PEDERSEN_12, index);
    }
    case PEDERSEN_13: {
        return pedersen_tables::generate_sidon_pedersen_table<13>(PEDERSEN_13, index);
    }
    case PEDERSEN_14: {
        return pedersen_tables::generate_sidon_pedersen_table<14>(PEDERSEN_14, index);
    }
    case PEDERSEN_15: {
        return pedersen_tables::generate_sidon_pedersen_table<15>(PEDERSEN_15, index);
    }
    case PEDERSEN_16: {
        return pedersen_tables::generate_sidon_pedersen_table<16>(PEDERSEN_16, index);
    }
    case PEDERSEN_17: {
        return pedersen_tables::generate_sidon_pedersen_table<17>(PEDERSEN_17, index);
    }
    case UINT_XOR_ROTATE0: {
        return uint_tables::generate_xor_rotate_table<6, 0>(UINT_XOR_ROTATE0, index);
    }
    case UINT_AND_ROTATE0: {
        return uint_tables::generate_and_rotate_table<6, 0>(UINT_AND_ROTATE0, index);
    }
    default: {
        barretenberg::errors::throw_or_abort("table id does not exist");
        return sparse_tables::generate_sparse_table_with_rotation<9, 8, 0>(AES_SPARSE_MAP, index);
    }
    }
}
} // namespace plookup
} // namespace waffle