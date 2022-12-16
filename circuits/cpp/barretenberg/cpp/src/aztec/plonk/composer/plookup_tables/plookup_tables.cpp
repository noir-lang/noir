#include "plookup_tables.hpp"

namespace plookup {

using namespace barretenberg;

namespace {
static std::array<MultiTable, MultiTableId::NUM_MULTI_TABLES> MULTI_TABLES;
static bool inited = false;

void init_multi_tables()
{
    MULTI_TABLES[MultiTableId::SHA256_CH_INPUT] = sha256_tables::get_choose_input_table(MultiTableId::SHA256_CH_INPUT);
    MULTI_TABLES[MultiTableId::SHA256_MAJ_INPUT] =
        sha256_tables::get_majority_input_table(MultiTableId::SHA256_MAJ_INPUT);
    MULTI_TABLES[MultiTableId::SHA256_WITNESS_INPUT] =
        sha256_tables::get_witness_extension_input_table(MultiTableId::SHA256_WITNESS_INPUT);
    MULTI_TABLES[MultiTableId::SHA256_CH_OUTPUT] =
        sha256_tables::get_choose_output_table(MultiTableId::SHA256_CH_OUTPUT);
    MULTI_TABLES[MultiTableId::SHA256_MAJ_OUTPUT] =
        sha256_tables::get_majority_output_table(MultiTableId::SHA256_MAJ_OUTPUT);
    MULTI_TABLES[MultiTableId::SHA256_WITNESS_OUTPUT] =
        sha256_tables::get_witness_extension_output_table(MultiTableId::SHA256_WITNESS_OUTPUT);
    MULTI_TABLES[MultiTableId::AES_NORMALIZE] = aes128_tables::get_aes_normalization_table(MultiTableId::AES_NORMALIZE);
    MULTI_TABLES[MultiTableId::AES_INPUT] = aes128_tables::get_aes_input_table(MultiTableId::AES_INPUT);
    MULTI_TABLES[MultiTableId::AES_SBOX] = aes128_tables::get_aes_sbox_table(MultiTableId::AES_SBOX);
    MULTI_TABLES[MultiTableId::PEDERSEN_LEFT_HI] =
        pedersen_tables::basic::get_pedersen_left_hi_table(MultiTableId::PEDERSEN_LEFT_HI);
    MULTI_TABLES[MultiTableId::PEDERSEN_LEFT_LO] =
        pedersen_tables::basic::get_pedersen_left_lo_table(MultiTableId::PEDERSEN_LEFT_LO);
    MULTI_TABLES[MultiTableId::PEDERSEN_RIGHT_HI] =
        pedersen_tables::basic::get_pedersen_right_hi_table(MultiTableId::PEDERSEN_RIGHT_HI);
    MULTI_TABLES[MultiTableId::PEDERSEN_RIGHT_LO] =
        pedersen_tables::basic::get_pedersen_right_lo_table(MultiTableId::PEDERSEN_RIGHT_LO);
    MULTI_TABLES[MultiTableId::PEDERSEN_IV] = pedersen_tables::basic::get_pedersen_iv_table(MultiTableId::PEDERSEN_IV);
    MULTI_TABLES[MultiTableId::UINT32_XOR] = uint_tables::get_uint32_xor_table(MultiTableId::UINT32_XOR);
    MULTI_TABLES[MultiTableId::UINT32_AND] = uint_tables::get_uint32_and_table(MultiTableId::UINT32_AND);
    MULTI_TABLES[MultiTableId::BN254_XLO] = ecc_generator_tables::ecc_generator_table<barretenberg::g1>::get_xlo_table(
        MultiTableId::BN254_XLO, BasicTableId::BN254_XLO_BASIC);
    MULTI_TABLES[MultiTableId::BN254_XHI] = ecc_generator_tables::ecc_generator_table<barretenberg::g1>::get_xhi_table(
        MultiTableId::BN254_XHI, BasicTableId::BN254_XHI_BASIC);
    MULTI_TABLES[MultiTableId::BN254_YLO] = ecc_generator_tables::ecc_generator_table<barretenberg::g1>::get_ylo_table(
        MultiTableId::BN254_YLO, BasicTableId::BN254_YLO_BASIC);
    MULTI_TABLES[MultiTableId::BN254_YHI] = ecc_generator_tables::ecc_generator_table<barretenberg::g1>::get_yhi_table(
        MultiTableId::BN254_YHI, BasicTableId::BN254_YHI_BASIC);
    MULTI_TABLES[MultiTableId::BN254_XYPRIME] =
        ecc_generator_tables::ecc_generator_table<barretenberg::g1>::get_xyprime_table(
            MultiTableId::BN254_XYPRIME, BasicTableId::BN254_XYPRIME_BASIC);
    MULTI_TABLES[MultiTableId::BN254_XLO_ENDO] =
        ecc_generator_tables::ecc_generator_table<barretenberg::g1>::get_xlo_endo_table(
            MultiTableId::BN254_XLO_ENDO, BasicTableId::BN254_XLO_ENDO_BASIC);
    MULTI_TABLES[MultiTableId::BN254_XHI_ENDO] =
        ecc_generator_tables::ecc_generator_table<barretenberg::g1>::get_xhi_endo_table(
            MultiTableId::BN254_XHI_ENDO, BasicTableId::BN254_XHI_ENDO_BASIC);
    MULTI_TABLES[MultiTableId::BN254_XYPRIME_ENDO] =
        ecc_generator_tables::ecc_generator_table<barretenberg::g1>::get_xyprime_endo_table(
            MultiTableId::BN254_XYPRIME_ENDO, BasicTableId::BN254_XYPRIME_ENDO_BASIC);
    MULTI_TABLES[MultiTableId::SECP256K1_XLO] = ecc_generator_tables::ecc_generator_table<secp256k1::g1>::get_xlo_table(
        MultiTableId::SECP256K1_XLO, BasicTableId::SECP256K1_XLO_BASIC);
    MULTI_TABLES[MultiTableId::SECP256K1_XHI] = ecc_generator_tables::ecc_generator_table<secp256k1::g1>::get_xhi_table(
        MultiTableId::SECP256K1_XHI, BasicTableId::SECP256K1_XHI_BASIC);
    MULTI_TABLES[MultiTableId::SECP256K1_YLO] = ecc_generator_tables::ecc_generator_table<secp256k1::g1>::get_ylo_table(
        MultiTableId::SECP256K1_YLO, BasicTableId::SECP256K1_YLO_BASIC);
    MULTI_TABLES[MultiTableId::SECP256K1_YHI] = ecc_generator_tables::ecc_generator_table<secp256k1::g1>::get_yhi_table(
        MultiTableId::SECP256K1_YHI, BasicTableId::SECP256K1_YHI_BASIC);
    MULTI_TABLES[MultiTableId::SECP256K1_XYPRIME] =
        ecc_generator_tables::ecc_generator_table<secp256k1::g1>::get_xyprime_table(
            MultiTableId::SECP256K1_XYPRIME, BasicTableId::SECP256K1_XYPRIME_BASIC);
    MULTI_TABLES[MultiTableId::SECP256K1_XLO_ENDO] =
        ecc_generator_tables::ecc_generator_table<secp256k1::g1>::get_xlo_endo_table(
            MultiTableId::SECP256K1_XLO_ENDO, BasicTableId::SECP256K1_XLO_ENDO_BASIC);
    MULTI_TABLES[MultiTableId::SECP256K1_XHI_ENDO] =
        ecc_generator_tables::ecc_generator_table<secp256k1::g1>::get_xhi_endo_table(
            MultiTableId::SECP256K1_XHI_ENDO, BasicTableId::SECP256K1_XHI_ENDO_BASIC);
    MULTI_TABLES[MultiTableId::SECP256K1_XYPRIME_ENDO] =
        ecc_generator_tables::ecc_generator_table<secp256k1::g1>::get_xyprime_endo_table(
            MultiTableId::SECP256K1_XYPRIME_ENDO, BasicTableId::SECP256K1_XYPRIME_ENDO_BASIC);
    MULTI_TABLES[MultiTableId::BLAKE_XOR] = blake2s_tables::get_blake2s_xor_table(MultiTableId::BLAKE_XOR);
    MULTI_TABLES[MultiTableId::BLAKE_XOR_ROTATE_16] =
        blake2s_tables::get_blake2s_xor_rotate_16_table(MultiTableId::BLAKE_XOR_ROTATE_16);
    MULTI_TABLES[MultiTableId::BLAKE_XOR_ROTATE_8] =
        blake2s_tables::get_blake2s_xor_rotate_8_table(MultiTableId::BLAKE_XOR_ROTATE_8);
    MULTI_TABLES[MultiTableId::BLAKE_XOR_ROTATE_7] =
        blake2s_tables::get_blake2s_xor_rotate_7_table(MultiTableId::BLAKE_XOR_ROTATE_7);
}
} // namespace

const MultiTable& create_table(const MultiTableId id)
{
    if (!inited) {
        init_multi_tables();
        inited = true;
    }
    return MULTI_TABLES[id];
}

ReadData<barretenberg::fr> get_lookup_accumulators(const MultiTableId id,
                                                   const fr& key_a,
                                                   const fr& key_b,
                                                   const bool is_2_to_1_lookup)
{
    // return multi-table, populating global array of all multi-tables if need be
    const auto& multi_table = create_table(id);
    const size_t num_lookups = multi_table.lookup_ids.size();

    ReadData<barretenberg::fr> lookup;

    const auto key_a_slices = numeric::slice_input_using_variable_bases(key_a, multi_table.slice_sizes);
    const auto key_b_slices = numeric::slice_input_using_variable_bases(key_b, multi_table.slice_sizes);

    std::vector<fr> column_1_raw_values;
    std::vector<fr> column_2_raw_values;
    std::vector<fr> column_3_raw_values;

    for (size_t i = 0; i < num_lookups; ++i) {
        // get i-th table query function and then submit query
        const auto values = multi_table.get_table_values[i]({ key_a_slices[i], key_b_slices[i] });
        // store all query data in raw columns and key entry
        column_1_raw_values.emplace_back(key_a_slices[i]);
        column_2_raw_values.emplace_back(is_2_to_1_lookup ? key_b_slices[i] : values[0]);
        column_3_raw_values.emplace_back(is_2_to_1_lookup ? values[0] : values[1]);

        // Question: why are we storing the key slices twice?
        const BasicTable::KeyEntry key_entry{ { key_a_slices[i], key_b_slices[i] }, values };
        lookup.key_entries.emplace_back(key_entry);
    }
    lookup[ColumnIdx::C1].resize(num_lookups);
    lookup[ColumnIdx::C2].resize(num_lookups);
    lookup[ColumnIdx::C3].resize(num_lookups);

    /**
     * A multi-table consists of multiple basic tables (say L = 6).
     *
     *             [      ]                [      ]
     *     [      ]|      |[      ][      ]|      |[      ]
     * M ≡ |  B1  ||  B2  ||  B3  ||  B4  ||  B5  ||  B6  |
     *     [      ]|      |[      ][      ]|      |[      ]
     *             [      ]                [      ]
     *        |̐       |̐       |̐       |̐       |̐       |̐
     *        s1      s2      s3      s4      s5      s6
     *
     * Note that different basic tables can be of different sizes. Every lookup query generates L output slices (one for
     * each basic table, here, s1, s2, ..., s6). In other words, every lookup query add L lookup gates to the program.
     * Let the input slices/keys be (a1, b1), (a2, b2), ..., (a6, b6). The lookup gate structure is as follows:
     *
     * +---+-----------------------------------+----------------------------------+-----------------------------------+
     * | s | key_a                             | key_b                            | output                            |
     * |---+-----------------------------------+----------------------------------+-----------------------------------|
     * | 6 | a6 + p.a5 + p^2.a4 + ... + p^5.a1 | b6 + q.b5 + qq.b4 + ... + q^5.b1 | s6 + r.s5 + r^2.s4 + ... + r^5.s1 |
     * | 5 | a5 + p.a4 + ...... + p^4.a1       | b5 + q.b4 + ...... + q^4.b1      | s5 + r.s4 + ...... + r^4.s1       |
     * | 4 | a4 + p.a3 + ... + p^3.a1          | b4 + q.b3 + ... + q^3.b1         | s4 + r.s3 + ... + r^3.s1          |
     * | 3 | a3 + p.a2 + p^2.a1                | b3 + q.b2 + q^2.b1               | s3 + r.s2 + r^2.s1                |
     * | 2 | a2 + p.a1                         | b2 + q.b1                        | s2 + r.a1                         |
     * | 1 | a1                                | b1                               | s1                                |
     * +---+-----------------------------------+----------------------------------+-----------------------------------+
     *
     * Note that we compute the accumulating sums of the slices so as to avoid using additonal gates for the purpose of
     * reconstructing inputs/outputs. Here, (p, q, r) are referred to as column coefficients/step sizes.
     * In the next few lines, we compute these accumulating sums from raw column values (a1, ..., a6), (b1, ..., b6),
     * (s1, ..., s6) and column coefficients (p, q, r).
     *
     * For more details: see
     * https://app.gitbook.com/o/-LgCgJ8TCO7eGlBr34fj/s/-MEwtqp3H6YhHUTQ_pVJ/plookup-gates-for-ultraplonk/lookup-table-structures
     *
     */
    lookup[ColumnIdx::C1][num_lookups - 1] = column_1_raw_values[num_lookups - 1];
    lookup[ColumnIdx::C2][num_lookups - 1] = column_2_raw_values[num_lookups - 1];
    lookup[ColumnIdx::C3][num_lookups - 1] = column_3_raw_values[num_lookups - 1];

    for (size_t i = 1; i < num_lookups; ++i) {
        const auto& previous_1 = lookup[ColumnIdx::C1][num_lookups - i];
        const auto& previous_2 = lookup[ColumnIdx::C2][num_lookups - i];
        const auto& previous_3 = lookup[ColumnIdx::C3][num_lookups - i];

        auto& current_1 = lookup[ColumnIdx::C1][num_lookups - 1 - i];
        auto& current_2 = lookup[ColumnIdx::C2][num_lookups - 1 - i];
        auto& current_3 = lookup[ColumnIdx::C3][num_lookups - 1 - i];

        const auto& raw_1 = column_1_raw_values[num_lookups - 1 - i];
        const auto& raw_2 = column_2_raw_values[num_lookups - 1 - i];
        const auto& raw_3 = column_3_raw_values[num_lookups - 1 - i];

        current_1 = raw_1 + previous_1 * multi_table.column_1_step_sizes[num_lookups - i];
        current_2 = raw_2 + previous_2 * multi_table.column_2_step_sizes[num_lookups - i];
        current_3 = raw_3 + previous_3 * multi_table.column_3_step_sizes[num_lookups - i];
    }
    return lookup;
}

} // namespace plookup