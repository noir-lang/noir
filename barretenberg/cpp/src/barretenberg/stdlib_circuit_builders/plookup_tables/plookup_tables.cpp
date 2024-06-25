#include "plookup_tables.hpp"
#include "barretenberg/common/constexpr_utils.hpp"
#include "barretenberg/stdlib_circuit_builders/plookup_tables/aes128.hpp"
#include "barretenberg/stdlib_circuit_builders/plookup_tables/blake2s.hpp"
#include "barretenberg/stdlib_circuit_builders/plookup_tables/dummy.hpp"
#include "barretenberg/stdlib_circuit_builders/plookup_tables/fixed_base/fixed_base.hpp"
#include "barretenberg/stdlib_circuit_builders/plookup_tables/non_native_group_generator.hpp"
#include "barretenberg/stdlib_circuit_builders/plookup_tables/sha256.hpp"
#include "barretenberg/stdlib_circuit_builders/plookup_tables/uint.hpp"

#include "barretenberg/stdlib_circuit_builders/plookup_tables/keccak/keccak_chi.hpp"
#include "barretenberg/stdlib_circuit_builders/plookup_tables/keccak/keccak_input.hpp"
#include "barretenberg/stdlib_circuit_builders/plookup_tables/keccak/keccak_output.hpp"
#include "barretenberg/stdlib_circuit_builders/plookup_tables/keccak/keccak_rho.hpp"
#include "barretenberg/stdlib_circuit_builders/plookup_tables/keccak/keccak_theta.hpp"
#include <mutex>
namespace bb::plookup {

using namespace bb;

namespace {
// TODO(@zac-williamson) convert these into static const members of a struct
// NOLINTNEXTLINE(cppcoreguidelines-avoid-non-const-global-variables)
std::array<MultiTable, MultiTableId::NUM_MULTI_TABLES> MULTI_TABLES;
// NOLINTNEXTLINE(cppcoreguidelines-avoid-non-const-global-variables)
bool initialised = false;
#ifndef NO_MULTITHREADING

// The multitables initialisation procedure is not thread-safe, so we need to make sure only 1 thread gets to initialize
// them.
std::mutex multi_table_mutex;
#endif
void init_multi_tables()
{
#ifndef NO_MULTITHREADING
    std::unique_lock<std::mutex> lock(multi_table_mutex);
#endif
    if (initialised) {
        return;
    }
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
    MULTI_TABLES[MultiTableId::UINT32_XOR] = uint_tables::get_uint32_xor_table(MultiTableId::UINT32_XOR);
    MULTI_TABLES[MultiTableId::UINT32_AND] = uint_tables::get_uint32_and_table(MultiTableId::UINT32_AND);
    MULTI_TABLES[MultiTableId::BN254_XLO] = ecc_generator_tables::ecc_generator_table<bb::g1>::get_xlo_table(
        MultiTableId::BN254_XLO, BasicTableId::BN254_XLO_BASIC);
    MULTI_TABLES[MultiTableId::BN254_XHI] = ecc_generator_tables::ecc_generator_table<bb::g1>::get_xhi_table(
        MultiTableId::BN254_XHI, BasicTableId::BN254_XHI_BASIC);
    MULTI_TABLES[MultiTableId::BN254_YLO] = ecc_generator_tables::ecc_generator_table<bb::g1>::get_ylo_table(
        MultiTableId::BN254_YLO, BasicTableId::BN254_YLO_BASIC);
    MULTI_TABLES[MultiTableId::BN254_YHI] = ecc_generator_tables::ecc_generator_table<bb::g1>::get_yhi_table(
        MultiTableId::BN254_YHI, BasicTableId::BN254_YHI_BASIC);
    MULTI_TABLES[MultiTableId::BN254_XYPRIME] = ecc_generator_tables::ecc_generator_table<bb::g1>::get_xyprime_table(
        MultiTableId::BN254_XYPRIME, BasicTableId::BN254_XYPRIME_BASIC);
    MULTI_TABLES[MultiTableId::BN254_XLO_ENDO] = ecc_generator_tables::ecc_generator_table<bb::g1>::get_xlo_endo_table(
        MultiTableId::BN254_XLO_ENDO, BasicTableId::BN254_XLO_ENDO_BASIC);
    MULTI_TABLES[MultiTableId::BN254_XHI_ENDO] = ecc_generator_tables::ecc_generator_table<bb::g1>::get_xhi_endo_table(
        MultiTableId::BN254_XHI_ENDO, BasicTableId::BN254_XHI_ENDO_BASIC);
    MULTI_TABLES[MultiTableId::BN254_XYPRIME_ENDO] =
        ecc_generator_tables::ecc_generator_table<bb::g1>::get_xyprime_endo_table(
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
    MULTI_TABLES[MultiTableId::KECCAK_FORMAT_INPUT] =
        keccak_tables::KeccakInput::get_keccak_input_table(MultiTableId::KECCAK_FORMAT_INPUT);
    MULTI_TABLES[MultiTableId::KECCAK_THETA_OUTPUT] =
        keccak_tables::Theta::get_theta_output_table(MultiTableId::KECCAK_THETA_OUTPUT);
    MULTI_TABLES[MultiTableId::KECCAK_CHI_OUTPUT] =
        keccak_tables::Chi::get_chi_output_table(MultiTableId::KECCAK_CHI_OUTPUT);
    MULTI_TABLES[MultiTableId::KECCAK_FORMAT_OUTPUT] =
        keccak_tables::KeccakOutput::get_keccak_output_table(MultiTableId::KECCAK_FORMAT_OUTPUT);
    MULTI_TABLES[MultiTableId::FIXED_BASE_LEFT_LO] =
        fixed_base::table::get_fixed_base_table<0, 128>(MultiTableId::FIXED_BASE_LEFT_LO);
    MULTI_TABLES[MultiTableId::FIXED_BASE_LEFT_HI] =
        fixed_base::table::get_fixed_base_table<1, 126>(MultiTableId::FIXED_BASE_LEFT_HI);
    MULTI_TABLES[MultiTableId::FIXED_BASE_RIGHT_LO] =
        fixed_base::table::get_fixed_base_table<2, 128>(MultiTableId::FIXED_BASE_RIGHT_LO);
    MULTI_TABLES[MultiTableId::FIXED_BASE_RIGHT_HI] =
        fixed_base::table::get_fixed_base_table<3, 126>(MultiTableId::FIXED_BASE_RIGHT_HI);

    bb::constexpr_for<0, 25, 1>([&]<size_t i>() {
        MULTI_TABLES[static_cast<size_t>(MultiTableId::KECCAK_NORMALIZE_AND_ROTATE) + i] =
            keccak_tables::Rho<8, i>::get_rho_output_table(MultiTableId::KECCAK_NORMALIZE_AND_ROTATE);
    });
    MULTI_TABLES[MultiTableId::HONK_DUMMY_MULTI] = dummy_tables::get_honk_dummy_multitable();
    initialised = true;
}
} // namespace
/**
 * @brief Return the multitable with the provided ID; construct all MultiTables if not constructed already
 * @details The multitables are relatively light objects (they do not themselves store raw table data) so the first time
 * we use one of them we simply construct all of them (regardless of which of them will actually be used) and store in
 * MULTI_TABLES array.
 *
 * @param id The index of a MultiTable in the MULTI_TABLES array
 * @return const MultiTable&
 */
const MultiTable& get_multitable(const MultiTableId id)
{
    if (!initialised) {
        init_multi_tables();
        initialised = true;
    }
    return MULTI_TABLES[id];
}

/**
 * @brief Given a table ID and the key(s) for a key-value lookup, return the lookup accumulators
 * @details In general the number of bits in original key/value is greater than what can be efficiently supported in
 * lookup tables. For this reason we actually perform lookups on the corresponding limbs. However, since we're
 * interested in the original values and not the limbs, its convenient to structure the witnesses of lookup gates to
 * store the former. This way we don't have to waste gates reaccumulating the limbs to compute the actual value of
 * interest. The way to do this is to populate the wires with 'accumulator' values such that the first gate in the
 * series contains the full accumulated values, and successive gates contain prior stages of the accumulator such that
 * wire_i - r*wire_{i-1} = v_i, where r = num limb bits and v_i is a limb that explicitly appears in one of the lookup
 * tables. See the detailed comment block below for more explanation.
 *
 * @param id
 * @param key_a
 * @param key_b
 * @param is_2_to_1_lookup
 * @return ReadData<bb::fr>
 */
ReadData<bb::fr> get_lookup_accumulators(const MultiTableId id,
                                         const fr& key_a,
                                         const fr& key_b,
                                         const bool is_2_to_1_lookup)
{
    // return multi-table, populating global array of all multi-tables if need be
    const auto& multi_table = get_multitable(id);
    const size_t num_lookups = multi_table.basic_table_ids.size();

    ReadData<bb::fr> lookup;
    const auto key_a_slices = numeric::slice_input_using_variable_bases(key_a, multi_table.slice_sizes);
    const auto key_b_slices = numeric::slice_input_using_variable_bases(key_b, multi_table.slice_sizes);

    std::vector<fr> column_1_raw_values;
    std::vector<fr> column_2_raw_values;
    std::vector<fr> column_3_raw_values;

    for (size_t i = 0; i < num_lookups; ++i) {
        // compute the value(s) corresponding to the key(s) using the i-th basic table query function
        const auto values = multi_table.get_table_values[i]({ key_a_slices[i], key_b_slices[i] });
        // store all query data in raw columns and key entry
        column_1_raw_values.emplace_back(key_a_slices[i]);
        column_2_raw_values.emplace_back(is_2_to_1_lookup ? key_b_slices[i] : values[0]);
        column_3_raw_values.emplace_back(is_2_to_1_lookup ? values[0] : values[1]);

        // Store the lookup entries for use in constructing the sorted table/lookup polynomials later on
        const BasicTable::LookupEntry lookup_entry{ { key_a_slices[i], key_b_slices[i] }, values };
        lookup.lookup_entries.emplace_back(lookup_entry);
    }

    lookup[C1].resize(num_lookups);
    lookup[C2].resize(num_lookups);
    lookup[C3].resize(num_lookups);

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
     * each basic table, here, s1, s2, ..., s6). In other words, every lookup query adds L lookup gates to the program.
     * For example, to look up the XOR of 32-bit inputs, we actually perform 6 individual lookups on the 6-bit XOR basic
     * table. Let the input slices/keys be (a1, b1), (a2, b2), ..., (a6, b6). The lookup gate structure is as follows:
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
     * reconstructing the original inputs/outputs. I.e. the output value at the 0th index in the above table is the
     * actual value we were interested in computing in the first place. Importantly, the structure of the remaining rows
     * is such that row_i - r*row_{i+1} produces an entry {a_j, b_j, s_j} that exactly corresponds to an entry in a
     * BasicTable. This is what gives rise to the wire_i - scalar*wire_i_shift structure in the lookup relation. Here,
     * (p, q, r) are referred to as column coefficients/step sizes. In the next few lines, we compute these accumulating
     * sums from raw column values (a1, ..., a6), (b1, ..., b6), (s1, ..., s6) and column coefficients (p, q, r).
     *
     * For more details: see
     * https://app.gitbook.com/o/-LgCgJ8TCO7eGlBr34fj/s/-MEwtqp3H6YhHUTQ_pVJ/plookup-gates-for-ultraplonk/lookup-table-structures
     *
     */
    lookup[C1][num_lookups - 1] = column_1_raw_values[num_lookups - 1];
    lookup[C2][num_lookups - 1] = column_2_raw_values[num_lookups - 1];
    lookup[C3][num_lookups - 1] = column_3_raw_values[num_lookups - 1];

    for (size_t i = num_lookups - 1; i > 0; --i) {
        lookup[C1][i - 1] = column_1_raw_values[i - 1] + lookup[C1][i] * multi_table.column_1_step_sizes[i];
        lookup[C2][i - 1] = column_2_raw_values[i - 1] + lookup[C2][i] * multi_table.column_2_step_sizes[i];
        lookup[C3][i - 1] = column_3_raw_values[i - 1] + lookup[C3][i] * multi_table.column_3_step_sizes[i];
    }
    return lookup;
}

BasicTable create_basic_table(const BasicTableId id, const size_t index)
{
    // we have >50 basic fixed base tables so we match with some logic instead of a switch statement
    auto id_var = static_cast<size_t>(id);
    if (id_var >= static_cast<size_t>(FIXED_BASE_0_0) && id_var < static_cast<size_t>(FIXED_BASE_1_0)) {
        return fixed_base::table::generate_basic_fixed_base_table<0>(
            id, index, id_var - static_cast<size_t>(FIXED_BASE_0_0));
    }
    if (id_var >= static_cast<size_t>(FIXED_BASE_1_0) && id_var < static_cast<size_t>(FIXED_BASE_2_0)) {
        return fixed_base::table::generate_basic_fixed_base_table<1>(
            id, index, id_var - static_cast<size_t>(FIXED_BASE_1_0));
    }
    if (id_var >= static_cast<size_t>(FIXED_BASE_2_0) && id_var < static_cast<size_t>(FIXED_BASE_3_0)) {
        return fixed_base::table::generate_basic_fixed_base_table<2>(
            id, index, id_var - static_cast<size_t>(FIXED_BASE_2_0));
    }
    if (id_var >= static_cast<size_t>(FIXED_BASE_3_0) && id_var < static_cast<size_t>(HONK_DUMMY_BASIC1)) {
        return fixed_base::table::generate_basic_fixed_base_table<3>(
            id, index, id_var - static_cast<size_t>(FIXED_BASE_3_0));
    }
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
    case UINT_XOR_ROTATE0: {
        return uint_tables::generate_xor_rotate_table<6, 0>(UINT_XOR_ROTATE0, index);
    }
    case UINT_AND_ROTATE0: {
        return uint_tables::generate_and_rotate_table<6, 0>(UINT_AND_ROTATE0, index);
    }
    case BN254_XLO_BASIC: {
        return ecc_generator_tables::ecc_generator_table<bb::g1>::generate_xlo_table(BN254_XLO_BASIC, index);
    }
    case BN254_XHI_BASIC: {
        return ecc_generator_tables::ecc_generator_table<bb::g1>::generate_xhi_table(BN254_XHI_BASIC, index);
    }
    case BN254_YLO_BASIC: {
        return ecc_generator_tables::ecc_generator_table<bb::g1>::generate_ylo_table(BN254_YLO_BASIC, index);
    }
    case BN254_YHI_BASIC: {
        return ecc_generator_tables::ecc_generator_table<bb::g1>::generate_yhi_table(BN254_YHI_BASIC, index);
    }
    case BN254_XYPRIME_BASIC: {
        return ecc_generator_tables::ecc_generator_table<bb::g1>::generate_xyprime_table(BN254_XYPRIME_BASIC, index);
    }
    case BN254_XLO_ENDO_BASIC: {
        return ecc_generator_tables::ecc_generator_table<bb::g1>::generate_xlo_endo_table(BN254_XLO_ENDO_BASIC, index);
    }
    case BN254_XHI_ENDO_BASIC: {
        return ecc_generator_tables::ecc_generator_table<bb::g1>::generate_xhi_endo_table(BN254_XHI_ENDO_BASIC, index);
    }
    case BN254_XYPRIME_ENDO_BASIC: {
        return ecc_generator_tables::ecc_generator_table<bb::g1>::generate_xyprime_endo_table(BN254_XYPRIME_ENDO_BASIC,
                                                                                              index);
    }
    case SECP256K1_XLO_BASIC: {
        return ecc_generator_tables::ecc_generator_table<secp256k1::g1>::generate_xlo_table(SECP256K1_XLO_BASIC, index);
    }
    case SECP256K1_XHI_BASIC: {
        return ecc_generator_tables::ecc_generator_table<secp256k1::g1>::generate_xhi_table(SECP256K1_XHI_BASIC, index);
    }
    case SECP256K1_YLO_BASIC: {
        return ecc_generator_tables::ecc_generator_table<secp256k1::g1>::generate_ylo_table(SECP256K1_YLO_BASIC, index);
    }
    case SECP256K1_YHI_BASIC: {
        return ecc_generator_tables::ecc_generator_table<secp256k1::g1>::generate_yhi_table(SECP256K1_YHI_BASIC, index);
    }
    case SECP256K1_XYPRIME_BASIC: {
        return ecc_generator_tables::ecc_generator_table<secp256k1::g1>::generate_xyprime_table(SECP256K1_XYPRIME_BASIC,
                                                                                                index);
    }
    case SECP256K1_XLO_ENDO_BASIC: {
        return ecc_generator_tables::ecc_generator_table<secp256k1::g1>::generate_xlo_endo_table(
            SECP256K1_XLO_ENDO_BASIC, index);
    }
    case SECP256K1_XHI_ENDO_BASIC: {
        return ecc_generator_tables::ecc_generator_table<secp256k1::g1>::generate_xhi_endo_table(
            SECP256K1_XHI_ENDO_BASIC, index);
    }
    case SECP256K1_XYPRIME_ENDO_BASIC: {
        return ecc_generator_tables::ecc_generator_table<secp256k1::g1>::generate_xyprime_endo_table(
            SECP256K1_XYPRIME_ENDO_BASIC, index);
    }
    case BLAKE_XOR_ROTATE0: {
        return blake2s_tables::generate_xor_rotate_table<6, 0>(BLAKE_XOR_ROTATE0, index);
    }
    case BLAKE_XOR_ROTATE0_SLICE5_MOD4: {
        return blake2s_tables::generate_xor_rotate_table<5, 0, true>(BLAKE_XOR_ROTATE0_SLICE5_MOD4, index);
    }
    case BLAKE_XOR_ROTATE2: {
        return blake2s_tables::generate_xor_rotate_table<6, 2>(BLAKE_XOR_ROTATE2, index);
    }
    case BLAKE_XOR_ROTATE1: {
        return blake2s_tables::generate_xor_rotate_table<6, 1>(BLAKE_XOR_ROTATE1, index);
    }
    case BLAKE_XOR_ROTATE4: {
        return blake2s_tables::generate_xor_rotate_table<6, 4>(BLAKE_XOR_ROTATE4, index);
    }
    case HONK_DUMMY_BASIC1: {
        return dummy_tables::generate_honk_dummy_table<HONK_DUMMY_BASIC1>(HONK_DUMMY_BASIC1, index);
    }
    case HONK_DUMMY_BASIC2: {
        return dummy_tables::generate_honk_dummy_table<HONK_DUMMY_BASIC2>(HONK_DUMMY_BASIC2, index);
    }
    case KECCAK_INPUT: {
        return keccak_tables::KeccakInput::generate_keccak_input_table(KECCAK_INPUT, index);
    }
    case KECCAK_THETA: {
        return keccak_tables::Theta::generate_theta_renormalization_table(KECCAK_THETA, index);
    }
    case KECCAK_CHI: {
        return keccak_tables::Chi::generate_chi_renormalization_table(KECCAK_CHI, index);
    }
    case KECCAK_OUTPUT: {
        return keccak_tables::KeccakOutput::generate_keccak_output_table(KECCAK_OUTPUT, index);
    }
    case KECCAK_RHO_1: {
        return keccak_tables::Rho<1>::generate_rho_renormalization_table(KECCAK_RHO_1, index);
    }
    case KECCAK_RHO_2: {
        return keccak_tables::Rho<2>::generate_rho_renormalization_table(KECCAK_RHO_2, index);
    }
    case KECCAK_RHO_3: {
        return keccak_tables::Rho<3>::generate_rho_renormalization_table(KECCAK_RHO_3, index);
    }
    case KECCAK_RHO_4: {
        return keccak_tables::Rho<4>::generate_rho_renormalization_table(KECCAK_RHO_4, index);
    }
    case KECCAK_RHO_5: {
        return keccak_tables::Rho<5>::generate_rho_renormalization_table(KECCAK_RHO_5, index);
    }
    case KECCAK_RHO_6: {
        return keccak_tables::Rho<6>::generate_rho_renormalization_table(KECCAK_RHO_6, index);
    }
    case KECCAK_RHO_7: {
        return keccak_tables::Rho<7>::generate_rho_renormalization_table(KECCAK_RHO_7, index);
    }
    case KECCAK_RHO_8: {
        return keccak_tables::Rho<8>::generate_rho_renormalization_table(KECCAK_RHO_8, index);
    }
    default: {
        throw_or_abort("table id does not exist");
        return sparse_tables::generate_sparse_table_with_rotation<9, 8, 0>(AES_SPARSE_MAP, index);
    }
    }
}
} // namespace bb::plookup
