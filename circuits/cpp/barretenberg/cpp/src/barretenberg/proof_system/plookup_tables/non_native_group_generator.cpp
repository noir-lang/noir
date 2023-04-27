#include "non_native_group_generator.hpp"

namespace plookup {
namespace ecc_generator_tables {

/**
 * Init 8-bit generator lookup tables
 * The 8-bit wNAF is structured so that entries are in the range [0, ..., 255]
 *
 * The actual scalar value = (wNAF * 2) - 255
 *
 * scalar values are from the values [-255, -253, ..., -3, -1, 1, 3, ..., 253, 255]
 **/
template <typename G1> void ecc_generator_table<G1>::init_generator_tables()
{
    if (init) {
        return;
    }
    element base_point = G1::one;

    auto d2 = base_point.dbl();
    std::array<element, 256> point_table;
    point_table[128] = base_point;
    for (size_t i = 1; i < 128; ++i) {
        point_table[i + 128] = point_table[i + 127] + d2;
    }
    for (size_t i = 0; i < 128; ++i) {
        point_table[127 - i] = -point_table[128 + i];
    }
    element::batch_normalize(&point_table[0], 256);

    auto beta = G1::Fq::cube_root_of_unity();
    for (size_t i = 0; i < 256; ++i) {
        uint256_t endo_x = static_cast<uint256_t>(point_table[i].x * beta);
        uint256_t x = static_cast<uint256_t>(point_table[i].x);
        uint256_t y = static_cast<uint256_t>(point_table[i].y);

        const uint256_t SHIFT = uint256_t(1) << 68;
        const uint256_t MASK = SHIFT - 1;
        uint256_t x0 = x & MASK;
        x = x >> 68;
        uint256_t x1 = x & MASK;
        x = x >> 68;
        uint256_t x2 = x & MASK;
        x = x >> 68;
        uint256_t x3 = x & MASK;

        uint256_t endox0 = endo_x & MASK;
        endo_x = endo_x >> 68;
        uint256_t endox1 = endo_x & MASK;
        endo_x = endo_x >> 68;
        uint256_t endox2 = endo_x & MASK;
        endo_x = endo_x >> 68;
        uint256_t endox3 = endo_x & MASK;

        uint256_t y0 = y & MASK;
        y = y >> 68;
        uint256_t y1 = y & MASK;
        y = y >> 68;
        uint256_t y2 = y & MASK;
        y = y >> 68;
        uint256_t y3 = y & MASK;
        ecc_generator_table<G1>::generator_xlo_table[i] = std::make_pair<barretenberg::fr, barretenberg::fr>(x0, x1);
        ecc_generator_table<G1>::generator_xhi_table[i] = std::make_pair<barretenberg::fr, barretenberg::fr>(x2, x3);
        ecc_generator_table<G1>::generator_endo_xlo_table[i] =
            std::make_pair<barretenberg::fr, barretenberg::fr>(endox0, endox1);
        ecc_generator_table<G1>::generator_endo_xhi_table[i] =
            std::make_pair<barretenberg::fr, barretenberg::fr>(endox2, endox3);
        ecc_generator_table<G1>::generator_ylo_table[i] = std::make_pair<barretenberg::fr, barretenberg::fr>(y0, y1);
        ecc_generator_table<G1>::generator_yhi_table[i] = std::make_pair<barretenberg::fr, barretenberg::fr>(y2, y3);
        ecc_generator_table<G1>::generator_xyprime_table[i] = std::make_pair<barretenberg::fr, barretenberg::fr>(
            barretenberg::fr(uint256_t(point_table[i].x)), barretenberg::fr(uint256_t(point_table[i].y)));
        ecc_generator_table<G1>::generator_endo_xyprime_table[i] = std::make_pair<barretenberg::fr, barretenberg::fr>(
            barretenberg::fr(uint256_t(point_table[i].x * beta)), barretenberg::fr(uint256_t(point_table[i].y)));
    }
    init = true;
}

// map 0 to 255 into 0 to 510 in steps of two
// actual naf value = (position * 2) - 255
template <typename G1> size_t ecc_generator_table<G1>::convert_position_to_shifted_naf(const size_t position)
{
    return (position * 2);
}

template <typename G1> size_t ecc_generator_table<G1>::convert_shifted_naf_to_position(const size_t shifted_naf)
{
    return shifted_naf / 2;
}

/**
 * Get 2 low 68-bit limbs of x-coordinate
 **/
template <typename G1>
std::array<barretenberg::fr, 2> ecc_generator_table<G1>::get_xlo_values(const std::array<uint64_t, 2> key)
{
    init_generator_tables();
    const size_t index = static_cast<size_t>(key[0]);
    return { ecc_generator_table<G1>::generator_xlo_table[index].first,
             ecc_generator_table<G1>::generator_xlo_table[index].second };
}

/**
 * Get 2 high 68-bit limbs of x-coordinate
 **/
template <typename G1>
std::array<barretenberg::fr, 2> ecc_generator_table<G1>::get_xhi_values(const std::array<uint64_t, 2> key)
{
    init_generator_tables();
    const size_t index = static_cast<size_t>(key[0]);
    return { ecc_generator_table<G1>::generator_xhi_table[index].first,
             ecc_generator_table<G1>::generator_xhi_table[index].second };
}

/**
 * Get 2 low 68-bit limbs of x-coordinate (for endomorphism point \lambda.[P])
 **/
template <typename G1>
std::array<barretenberg::fr, 2> ecc_generator_table<G1>::get_xlo_endo_values(const std::array<uint64_t, 2> key)
{
    init_generator_tables();
    const size_t index = static_cast<size_t>(key[0]);
    return { ecc_generator_table<G1>::generator_endo_xlo_table[index].first,
             ecc_generator_table<G1>::generator_endo_xlo_table[index].second };
}

/**
 * Get 2 high 68-bit limbs of x-coordinate (for endomorphism point \lambda.[1])
 **/
template <typename G1>
std::array<barretenberg::fr, 2> ecc_generator_table<G1>::get_xhi_endo_values(const std::array<uint64_t, 2> key)
{
    init_generator_tables();
    const size_t index = static_cast<size_t>(key[0]);
    return { ecc_generator_table<G1>::generator_endo_xhi_table[index].first,
             ecc_generator_table<G1>::generator_endo_xhi_table[index].second };
}

/**
 * Get 2 low 68-bit limbs of y-coordinate
 **/
template <typename G1>
std::array<barretenberg::fr, 2> ecc_generator_table<G1>::get_ylo_values(const std::array<uint64_t, 2> key)
{
    init_generator_tables();
    const size_t index = static_cast<size_t>(key[0]);
    return { ecc_generator_table<G1>::generator_ylo_table[index].first,
             ecc_generator_table<G1>::generator_ylo_table[index].second };
}

/**
 * Get 2 high 68-bit limbs of y-coordinate
 **/
template <typename G1>
std::array<barretenberg::fr, 2> ecc_generator_table<G1>::get_yhi_values(const std::array<uint64_t, 2> key)
{
    init_generator_tables();
    const size_t index = static_cast<size_t>(key[0]);
    return { ecc_generator_table<G1>::generator_yhi_table[index].first,
             ecc_generator_table<G1>::generator_yhi_table[index].second };
}

/**
 * Get the prime basis limbs for the x and y coordinates
 **/
template <typename G1>
std::array<barretenberg::fr, 2> ecc_generator_table<G1>::get_xyprime_values(const std::array<uint64_t, 2> key)
{
    init_generator_tables();
    const size_t index = static_cast<size_t>(key[0]);
    return { ecc_generator_table<G1>::generator_xyprime_table[index].first,
             ecc_generator_table<G1>::generator_xyprime_table[index].second };
}

/**
 * Get the prime basis limbs for the x and y coordinates (endomorphism version for \lambda.[1])
 **/
template <typename G1>
std::array<barretenberg::fr, 2> ecc_generator_table<G1>::get_xyprime_endo_values(const std::array<uint64_t, 2> key)
{
    init_generator_tables();
    const size_t index = static_cast<size_t>(key[0]);
    return { ecc_generator_table<G1>::generator_endo_xyprime_table[index].first,
             ecc_generator_table<G1>::generator_endo_xyprime_table[index].second };
}

template <typename G1> BasicTable ecc_generator_table<G1>::generate_xlo_table(BasicTableId id, const size_t table_index)
{
    BasicTable table;
    table.id = id;
    table.table_index = table_index;
    table.size = 256;
    table.use_twin_keys = false;

    for (size_t i = 0; i < table.size; ++i) {
        table.column_1.emplace_back((i));
        table.column_2.emplace_back(ecc_generator_table<G1>::generator_xlo_table[i].first);
        table.column_3.emplace_back(ecc_generator_table<G1>::generator_xlo_table[i].second);
    }

    table.get_values_from_key = &get_xlo_values;

    table.column_1_step_size = 0;
    table.column_2_step_size = 0;
    table.column_3_step_size = 0;

    return table;
}

template <typename G1> BasicTable ecc_generator_table<G1>::generate_xhi_table(BasicTableId id, const size_t table_index)
{
    BasicTable table;
    table.id = id;
    table.table_index = table_index;
    table.size = 256;
    table.use_twin_keys = false;

    for (size_t i = 0; i < table.size; ++i) {
        table.column_1.emplace_back((i));
        table.column_2.emplace_back(ecc_generator_table<G1>::generator_xhi_table[i].first);
        table.column_3.emplace_back(ecc_generator_table<G1>::generator_xhi_table[i].second);
    }

    table.get_values_from_key = &get_xhi_values;

    table.column_1_step_size = 0;
    table.column_2_step_size = 0;
    table.column_3_step_size = 0;

    return table;
}

template <typename G1>
BasicTable ecc_generator_table<G1>::generate_xlo_endo_table(BasicTableId id, const size_t table_index)
{
    BasicTable table;
    table.id = id;
    table.table_index = table_index;
    table.size = 256;
    table.use_twin_keys = false;

    for (size_t i = 0; i < table.size; ++i) {
        table.column_1.emplace_back((i));
        table.column_2.emplace_back(ecc_generator_table<G1>::generator_endo_xlo_table[i].first);
        table.column_3.emplace_back(ecc_generator_table<G1>::generator_endo_xlo_table[i].second);
    }

    table.get_values_from_key = &get_xlo_endo_values;

    table.column_1_step_size = 0;
    table.column_2_step_size = 0;
    table.column_3_step_size = 0;

    return table;
}

template <typename G1>
BasicTable ecc_generator_table<G1>::generate_xhi_endo_table(BasicTableId id, const size_t table_index)
{
    BasicTable table;
    table.id = id;
    table.table_index = table_index;
    table.size = 256;
    table.use_twin_keys = false;

    for (size_t i = 0; i < table.size; ++i) {
        table.column_1.emplace_back((i));
        table.column_2.emplace_back(ecc_generator_table<G1>::generator_endo_xhi_table[i].first);
        table.column_3.emplace_back(ecc_generator_table<G1>::generator_endo_xhi_table[i].second);
    }

    table.get_values_from_key = &get_xhi_endo_values;

    table.column_1_step_size = 0;
    table.column_2_step_size = 0;
    table.column_3_step_size = 0;

    return table;
}

template <typename G1> BasicTable ecc_generator_table<G1>::generate_ylo_table(BasicTableId id, const size_t table_index)
{
    BasicTable table;
    table.id = id;
    table.table_index = table_index;
    table.size = 256;
    table.use_twin_keys = false;

    for (size_t i = 0; i < table.size; ++i) {
        table.column_1.emplace_back((i));
        table.column_2.emplace_back(ecc_generator_table<G1>::generator_ylo_table[i].first);
        table.column_3.emplace_back(ecc_generator_table<G1>::generator_ylo_table[i].second);
    }

    table.get_values_from_key = &get_ylo_values;

    table.column_1_step_size = 0;
    table.column_2_step_size = 0;
    table.column_3_step_size = 0;

    return table;
}

template <typename G1> BasicTable ecc_generator_table<G1>::generate_yhi_table(BasicTableId id, const size_t table_index)
{
    BasicTable table;
    table.id = id;
    table.table_index = table_index;
    table.size = 256;
    table.use_twin_keys = false;

    for (size_t i = 0; i < table.size; ++i) {
        table.column_1.emplace_back((i));
        table.column_2.emplace_back(ecc_generator_table<G1>::generator_yhi_table[i].first);
        table.column_3.emplace_back(ecc_generator_table<G1>::generator_yhi_table[i].second);
    }

    table.get_values_from_key = &get_yhi_values;

    table.column_1_step_size = 0;
    table.column_2_step_size = 0;
    table.column_3_step_size = 0;

    return table;
}

template <typename G1>
BasicTable ecc_generator_table<G1>::generate_xyprime_table(BasicTableId id, const size_t table_index)
{
    BasicTable table;
    table.id = id;
    table.table_index = table_index;
    table.size = 256;
    table.use_twin_keys = false;

    for (size_t i = 0; i < table.size; ++i) {
        table.column_1.emplace_back((i));
        table.column_2.emplace_back(ecc_generator_table<G1>::generator_xyprime_table[i].first);
        table.column_3.emplace_back(ecc_generator_table<G1>::generator_xyprime_table[i].second);
    }

    table.get_values_from_key = &get_xyprime_values;

    table.column_1_step_size = 0;
    table.column_2_step_size = 0;
    table.column_3_step_size = 0;

    return table;
}

template <typename G1>
BasicTable ecc_generator_table<G1>::generate_xyprime_endo_table(BasicTableId id, const size_t table_index)
{
    BasicTable table;
    table.id = id;
    table.table_index = table_index;
    table.size = 256;
    table.use_twin_keys = false;

    for (size_t i = 0; i < table.size; ++i) {
        table.column_1.emplace_back((i));
        table.column_2.emplace_back(ecc_generator_table<G1>::generator_endo_xyprime_table[i].first);
        table.column_3.emplace_back(ecc_generator_table<G1>::generator_endo_xyprime_table[i].second);
    }

    table.get_values_from_key = &get_xyprime_endo_values;

    table.column_1_step_size = 0;
    table.column_2_step_size = 0;
    table.column_3_step_size = 0;

    return table;
}

template <typename G1>
MultiTable ecc_generator_table<G1>::get_xlo_table(const MultiTableId id, const BasicTableId basic_id)
{
    const size_t num_entries = 1;
    MultiTable table(256, 0, 0, 1);

    table.id = id;
    for (size_t i = 0; i < num_entries; ++i) {
        table.slice_sizes.emplace_back(512);
        table.lookup_ids.emplace_back(basic_id);
        table.get_table_values.emplace_back(&get_xlo_values);
    }
    return table;
}

template <typename G1>
MultiTable ecc_generator_table<G1>::get_xhi_table(const MultiTableId id, const BasicTableId basic_id)
{
    const size_t num_entries = 1;
    MultiTable table(256, 0, 0, 1);

    table.id = id;
    for (size_t i = 0; i < num_entries; ++i) {
        table.slice_sizes.emplace_back(512);
        table.lookup_ids.emplace_back(basic_id);
        table.get_table_values.emplace_back(&get_xhi_values);
    }
    return table;
}

template <typename G1>
MultiTable ecc_generator_table<G1>::get_xlo_endo_table(const MultiTableId id, const BasicTableId basic_id)
{
    const size_t num_entries = 1;
    MultiTable table(256, 0, 0, 1);

    table.id = id;
    for (size_t i = 0; i < num_entries; ++i) {
        table.slice_sizes.emplace_back(512);
        table.lookup_ids.emplace_back(basic_id);
        table.get_table_values.emplace_back(&get_xlo_endo_values);
    }
    return table;
}

template <typename G1>
MultiTable ecc_generator_table<G1>::get_xhi_endo_table(const MultiTableId id, const BasicTableId basic_id)
{
    const size_t num_entries = 1;
    MultiTable table(256, 0, 0, 1);

    table.id = id;
    for (size_t i = 0; i < num_entries; ++i) {
        table.slice_sizes.emplace_back(512);
        table.lookup_ids.emplace_back(basic_id);
        table.get_table_values.emplace_back(&get_xhi_endo_values);
    }
    return table;
}

template <typename G1>
MultiTable ecc_generator_table<G1>::get_ylo_table(const MultiTableId id, const BasicTableId basic_id)
{
    const size_t num_entries = 1;
    MultiTable table(256, 0, 0, 1);

    table.id = id;
    for (size_t i = 0; i < num_entries; ++i) {
        table.slice_sizes.emplace_back(512);
        table.lookup_ids.emplace_back(basic_id);
        table.get_table_values.emplace_back(&get_ylo_values);
    }
    return table;
}

template <typename G1>
MultiTable ecc_generator_table<G1>::get_yhi_table(const MultiTableId id, const BasicTableId basic_id)
{
    const size_t num_entries = 1;
    MultiTable table(256, 0, 0, 1);

    table.id = id;
    for (size_t i = 0; i < num_entries; ++i) {
        table.slice_sizes.emplace_back(512);
        table.lookup_ids.emplace_back(basic_id);
        table.get_table_values.emplace_back(&get_yhi_values);
    }
    return table;
}

template <typename G1>
MultiTable ecc_generator_table<G1>::get_xyprime_table(const MultiTableId id, const BasicTableId basic_id)
{
    const size_t num_entries = 1;
    MultiTable table(256, 0, 0, 1);

    table.id = id;
    for (size_t i = 0; i < num_entries; ++i) {
        table.slice_sizes.emplace_back(512);
        table.lookup_ids.emplace_back(basic_id);
        table.get_table_values.emplace_back(&get_xyprime_values);
    }
    return table;
}

template <typename G1>
MultiTable ecc_generator_table<G1>::get_xyprime_endo_table(const MultiTableId id, const BasicTableId basic_id)
{
    const size_t num_entries = 1;
    MultiTable table(256, 0, 0, 1);

    table.id = id;
    for (size_t i = 0; i < num_entries; ++i) {
        table.slice_sizes.emplace_back(512);
        table.lookup_ids.emplace_back(basic_id);
        table.get_table_values.emplace_back(&get_xyprime_endo_values);
    }
    return table;
}
template class ecc_generator_table<barretenberg::g1>;
template class ecc_generator_table<secp256k1::g1>;

} // namespace ecc_generator_tables
} // namespace plookup