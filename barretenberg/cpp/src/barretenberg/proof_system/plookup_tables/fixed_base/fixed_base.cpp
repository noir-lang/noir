
#include "./fixed_base.hpp"

#include "barretenberg/common/constexpr_utils.hpp"
#include "barretenberg/crypto/pedersen_hash/pedersen.hpp"
#include "barretenberg/numeric/bitop/pow.hpp"
#include "barretenberg/numeric/bitop/rotate.hpp"
#include "barretenberg/numeric/bitop/sparse_form.hpp"
namespace bb::plookup::fixed_base {

/**
 * @brief Given a base_point [P] and an offset_generator [G], compute a lookup table of MAX_TABLE_SIZE that contains the
 * following terms:
 *
 * { [G] + 0.[P] , [G] + 1.[P], ..., [G] + (MAX_TABLE_SIZE - 1).[P] }
 *
 * @param base_point
 * @param offset_generator
 * @return table::single_lookup_table
 */
table::single_lookup_table table::generate_single_lookup_table(const affine_element& base_point,
                                                               const affine_element& offset_generator)
{
    std::vector<element> table_raw(MAX_TABLE_SIZE);

    element accumulator = offset_generator;
    for (size_t i = 0; i < MAX_TABLE_SIZE; ++i) {
        table_raw[i] = accumulator;
        accumulator += base_point;
    }
    element::batch_normalize(&table_raw[0], MAX_TABLE_SIZE);
    single_lookup_table table(MAX_TABLE_SIZE);
    for (size_t i = 0; i < table_raw.size(); ++i) {
        table[i] = affine_element{ table_raw[i].x, table_raw[i].y };
    }
    return table;
}

/**
 * @brief For a given base point [P], compute the lookup tables required to traverse a `num_bits` sized lookup
 *
 * i.e. call `generate_single_lookup_table` for the following base points:
 *
 * { [P], [P] * (1 << BITS_PER_TABLE), [P] * (1 << BITS_PER_TABLE * 2), ..., [P] * (1 << BITS_PER_TABLE * (NUM_TABLES -
 * 1)) }
 *
 * @tparam num_bits
 * @param input
 * @return table::fixed_base_scalar_mul_tables
 */
template <size_t num_bits> table::fixed_base_scalar_mul_tables table::generate_tables(const affine_element& input)
{
    constexpr size_t NUM_TABLES = get_num_tables_per_multi_table<num_bits>();

    fixed_base_scalar_mul_tables result;
    result.reserve(NUM_TABLES);

    std::vector<uint8_t> input_buf;
    serialize::write(input_buf, input);
    const auto offset_generators = grumpkin::g1::derive_generators(input_buf, NUM_TABLES);

    grumpkin::g1::element accumulator = input;
    for (size_t i = 0; i < NUM_TABLES; ++i) {
        result.emplace_back(generate_single_lookup_table(accumulator, offset_generators[i]));
        for (size_t j = 0; j < BITS_PER_TABLE; ++j) {
            accumulator = accumulator.dbl();
        }
    }
    return result;
}

/**
 * @brief For a fixed-base lookup of size `num_table_bits` and an input base point `input`,
 *        return the total contrbution in the scalar multiplication output from the offset generators in the lookup
 * tables.
 *
 * @note We need the base point as an input parameter because we derive the offset generator using our hash-to-curve
 * algorithm, where the base point is used as the domain separator. Ensures generator points cannot collide with base
 * points w/o solving the dlog problem
 * @tparam num_table_bits
 * @param input
 * @return grumpkin::g1::affine_element
 */
template <size_t num_table_bits>
grumpkin::g1::affine_element table::generate_generator_offset(const grumpkin::g1::affine_element& input)
{
    constexpr size_t NUM_TABLES = get_num_tables_per_multi_table<num_table_bits>();

    std::vector<uint8_t> input_buf;
    serialize::write(input_buf, input);
    const auto offset_generators = grumpkin::g1::derive_generators(input_buf, NUM_TABLES);
    grumpkin::g1::element acc = grumpkin::g1::point_at_infinity;
    for (const auto& gen : offset_generators) {
        acc += gen;
    }
    return acc;
}

/**
 * @brief Given a point, do we have a precomputed lookup table for this point?
 *
 * @param input
 * @return true
 * @return false
 */
bool table::lookup_table_exists_for_point(const affine_element& input)
{
    return (input == LHS_GENERATOR_POINT || input == RHS_GENERATOR_POINT);
}

/**
 * @brief Given a point, return (if it exists) the 2 MultiTableId's that correspond to the LO_SCALAR, HI_SCALAR
 * MultiTables
 *
 * @param input
 * @return std::optional<std::array<MultiTableId, 2>>
 */
std::optional<std::array<MultiTableId, 2>> table::get_lookup_table_ids_for_point(
    const grumpkin::g1::affine_element& input)
{
    if (input == LHS_GENERATOR_POINT) {
        return { { FIXED_BASE_LEFT_LO, FIXED_BASE_LEFT_HI } };
    }
    if (input == RHS_GENERATOR_POINT) {
        return { { FIXED_BASE_RIGHT_LO, FIXED_BASE_RIGHT_HI } };
    }
    return {};
}

/**
 * @brief Given a table id, return the offset generator term that will be present in the final scalar mul output.
 *
 * Return value is std::optional in case the table_id is not a fixed-base table.
 *
 * @param table_id
 * @return std::optional<affine_element>
 */
std::optional<grumpkin::g1::affine_element> table::get_generator_offset_for_table_id(const MultiTableId table_id)
{
    if (table_id == FIXED_BASE_LEFT_LO) {
        return fixed_base_table_offset_generators[0];
    }
    if (table_id == FIXED_BASE_LEFT_HI) {
        return fixed_base_table_offset_generators[1];
    }
    if (table_id == FIXED_BASE_RIGHT_LO) {
        return fixed_base_table_offset_generators[2];
    }
    if (table_id == FIXED_BASE_RIGHT_HI) {
        return fixed_base_table_offset_generators[3];
    }
    return std::nullopt;
}

using function_ptr = std::array<bb::fr, 2> (*)(const std::array<uint64_t, 2>);
using function_ptr_table =
    std::array<std::array<function_ptr, table::MAX_NUM_TABLES_IN_MULTITABLE>, table::NUM_FIXED_BASE_MULTI_TABLES>;
/**
 * @brief create a compile-time static 2D array of all our required `get_basic_fixed_base_table_values` function
 * pointers, so that we can specify the function pointer required for this method call using runtime variables
 * `multitable_index`, `table_index`. (downstream code becomes a lot simpler if `table_index` is not compile time,
 * particularly the init code in `plookup_tables.cpp`)
 * @return constexpr function_ptr_table
 */
constexpr function_ptr_table make_function_pointer_table()
{
    function_ptr_table table;
    bb::constexpr_for<0, table::NUM_FIXED_BASE_MULTI_TABLES, 1>([&]<size_t i>() {
        bb::constexpr_for<0, table::MAX_NUM_TABLES_IN_MULTITABLE, 1>(
            [&]<size_t j>() { table[i][j] = &table::get_basic_fixed_base_table_values<i, j>; });
    });
    return table;
};

/**
 * @brief Generate a single fixed-base-scalar-mul plookup table
 *
 * @tparam multitable_index , which of our 4 multitables is this basic table a part of?
 * @param id the BasicTableId
 * @param basic_table_index plookup table index
 * @param table_index This index describes which bit-slice the basic table corresponds to. i.e. table_index = 0 maps to
 *                    the least significant bit slice
 * @return BasicTable
 */
template <size_t multitable_index>
BasicTable table::generate_basic_fixed_base_table(BasicTableId id, size_t basic_table_index, size_t table_index)
{
    static_assert(multitable_index < NUM_FIXED_BASE_MULTI_TABLES);
    ASSERT(table_index < MAX_NUM_TABLES_IN_MULTITABLE);

    const size_t multitable_bits = get_num_bits_of_multi_table(multitable_index);
    const size_t bits_covered_by_previous_tables_in_multitable = BITS_PER_TABLE * table_index;
    const bool is_small_table = (multitable_bits - bits_covered_by_previous_tables_in_multitable) < BITS_PER_TABLE;
    const size_t table_bits =
        is_small_table ? multitable_bits - bits_covered_by_previous_tables_in_multitable : BITS_PER_TABLE;
    const auto table_size = static_cast<size_t>(1ULL << table_bits);
    BasicTable table;
    table.id = id;
    table.table_index = basic_table_index;
    table.size = table_size;
    table.use_twin_keys = false;

    const auto& basic_table = fixed_base_tables[multitable_index][table_index];

    for (size_t i = 0; i < table.size; ++i) {
        table.column_1.emplace_back(i);
        table.column_2.emplace_back(basic_table[i].x);
        table.column_3.emplace_back(basic_table[i].y);
    }
    table.get_values_from_key = nullptr;

    constexpr function_ptr_table get_values_from_key_table = make_function_pointer_table();
    table.get_values_from_key = get_values_from_key_table[multitable_index][table_index];

    ASSERT(table.get_values_from_key != nullptr);
    table.column_1_step_size = table.size;
    table.column_2_step_size = 0;
    table.column_3_step_size = 0;

    return table;
}

/**
 * @brief Generate a multi-table that describes the lookups required to cover a fixed-base-scalar-mul of `num_bits`
 *
 * @tparam multitable_index , which one of our 4 multitables are we generating?
 * @tparam num_bits , this will be either `BITS_PER_LO_SCALAR` or `BITS_PER_HI_SCALAR`
 * @param id
 * @return MultiTable
 */
template <size_t multitable_index, size_t num_bits> MultiTable table::get_fixed_base_table(const MultiTableId id)
{
    static_assert(num_bits == BITS_PER_LO_SCALAR || num_bits == BITS_PER_HI_SCALAR);
    constexpr size_t NUM_TABLES = get_num_tables_per_multi_table<num_bits>();
    constexpr std::array<BasicTableId, NUM_FIXED_BASE_MULTI_TABLES> basic_table_ids{
        FIXED_BASE_0_0,
        FIXED_BASE_1_0,
        FIXED_BASE_2_0,
        FIXED_BASE_3_0,
    };
    constexpr function_ptr_table get_values_from_key_table = make_function_pointer_table();

    MultiTable table(MAX_TABLE_SIZE, 0, 0, NUM_TABLES);
    table.id = id;
    table.get_table_values.resize(NUM_TABLES);
    table.lookup_ids.resize(NUM_TABLES);
    for (size_t i = 0; i < NUM_TABLES; ++i) {
        table.slice_sizes.emplace_back(MAX_TABLE_SIZE);
        table.get_table_values[i] = get_values_from_key_table[multitable_index][i];
        static_assert(multitable_index < NUM_FIXED_BASE_MULTI_TABLES);
        size_t idx = i + static_cast<size_t>(basic_table_ids[multitable_index]);
        table.lookup_ids[i] = static_cast<plookup::BasicTableId>(idx);
    }
    return table;
}

template grumpkin::g1::affine_element table::generate_generator_offset<table::BITS_PER_LO_SCALAR>(
    const grumpkin::g1::affine_element& input);
template grumpkin::g1::affine_element table::generate_generator_offset<table::BITS_PER_HI_SCALAR>(
    const grumpkin::g1::affine_element& input);
template table::fixed_base_scalar_mul_tables table::generate_tables<table::BITS_PER_LO_SCALAR>(
    const table::affine_element& input);
template table::fixed_base_scalar_mul_tables table::generate_tables<table::BITS_PER_HI_SCALAR>(
    const table::affine_element& input);

template BasicTable table::generate_basic_fixed_base_table<0>(BasicTableId, size_t, size_t);
template BasicTable table::generate_basic_fixed_base_table<1>(BasicTableId, size_t, size_t);
template BasicTable table::generate_basic_fixed_base_table<2>(BasicTableId, size_t, size_t);
template BasicTable table::generate_basic_fixed_base_table<3>(BasicTableId, size_t, size_t);
template MultiTable table::get_fixed_base_table<0, table::BITS_PER_LO_SCALAR>(MultiTableId);
template MultiTable table::get_fixed_base_table<1, table::BITS_PER_HI_SCALAR>(MultiTableId);
template MultiTable table::get_fixed_base_table<2, table::BITS_PER_LO_SCALAR>(MultiTableId);
template MultiTable table::get_fixed_base_table<3, table::BITS_PER_HI_SCALAR>(MultiTableId);

const table::all_multi_tables table::fixed_base_tables = {
    table::generate_tables<BITS_PER_LO_SCALAR>(lhs_base_point_lo),
    table::generate_tables<BITS_PER_HI_SCALAR>(lhs_base_point_hi),
    table::generate_tables<BITS_PER_LO_SCALAR>(rhs_base_point_lo),
    table::generate_tables<BITS_PER_HI_SCALAR>(rhs_base_point_hi),
};

const std::array<table::affine_element, table::NUM_FIXED_BASE_MULTI_TABLES>
    table::fixed_base_table_offset_generators = {
        table::generate_generator_offset<BITS_PER_LO_SCALAR>(lhs_base_point_lo),
        table::generate_generator_offset<BITS_PER_HI_SCALAR>(lhs_base_point_hi),
        table::generate_generator_offset<BITS_PER_LO_SCALAR>(rhs_base_point_lo),
        table::generate_generator_offset<BITS_PER_HI_SCALAR>(rhs_base_point_hi),
    };

} // namespace bb::plookup::fixed_base