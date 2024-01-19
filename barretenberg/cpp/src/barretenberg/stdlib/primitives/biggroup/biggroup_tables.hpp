#pragma once
#include "barretenberg/proof_system/plookup_tables/types.hpp"
namespace bb::plonk {
namespace stdlib {

using plookup::MultiTableId;

/**
 * @brief Constructs a ROM table to look up linear combinations of group elements
 *
 * @tparam C
 * @tparam Fq
 * @tparam Fr
 * @tparam G
 * @tparam num_elements
 * @tparam typename
 * @param rom_data the ROM table we are writing into
 * @param limb_max the maximum size of each limb in the ROM table.
 *
 * @details When reading a group element *out* of the ROM table, we must know the maximum value of each coordinate's
 * limbs. We take this value to be the maximum of the maximum values of the input limbs into the table!
 * @return std::array<twin_rom_table<C>, 5>
 */
template <typename C, class Fq, class Fr, class G>
template <size_t num_elements, typename>
std::array<twin_rom_table<C>, 5> element<C, Fq, Fr, G>::create_group_element_rom_tables(
    const std::array<element, num_elements>& rom_data, std::array<uint256_t, 8>& limb_max)
{
    std::vector<std::array<field_t<C>, 2>> x_lo_limbs;
    std::vector<std::array<field_t<C>, 2>> x_hi_limbs;
    std::vector<std::array<field_t<C>, 2>> y_lo_limbs;
    std::vector<std::array<field_t<C>, 2>> y_hi_limbs;
    std::vector<std::array<field_t<C>, 2>> prime_limbs;

    for (size_t i = 0; i < num_elements; ++i) {
        limb_max[0] = std::max(limb_max[0], rom_data[i].x.binary_basis_limbs[0].maximum_value);
        limb_max[1] = std::max(limb_max[1], rom_data[i].x.binary_basis_limbs[1].maximum_value);
        limb_max[2] = std::max(limb_max[2], rom_data[i].x.binary_basis_limbs[2].maximum_value);
        limb_max[3] = std::max(limb_max[3], rom_data[i].x.binary_basis_limbs[3].maximum_value);
        limb_max[4] = std::max(limb_max[4], rom_data[i].y.binary_basis_limbs[0].maximum_value);
        limb_max[5] = std::max(limb_max[5], rom_data[i].y.binary_basis_limbs[1].maximum_value);
        limb_max[6] = std::max(limb_max[6], rom_data[i].y.binary_basis_limbs[2].maximum_value);
        limb_max[7] = std::max(limb_max[7], rom_data[i].y.binary_basis_limbs[3].maximum_value);

        x_lo_limbs.emplace_back(std::array<field_t<C>, 2>{ rom_data[i].x.binary_basis_limbs[0].element,
                                                           rom_data[i].x.binary_basis_limbs[1].element });
        x_hi_limbs.emplace_back(std::array<field_t<C>, 2>{ rom_data[i].x.binary_basis_limbs[2].element,
                                                           rom_data[i].x.binary_basis_limbs[3].element });
        y_lo_limbs.emplace_back(std::array<field_t<C>, 2>{ rom_data[i].y.binary_basis_limbs[0].element,
                                                           rom_data[i].y.binary_basis_limbs[1].element });
        y_hi_limbs.emplace_back(std::array<field_t<C>, 2>{ rom_data[i].y.binary_basis_limbs[2].element,
                                                           rom_data[i].y.binary_basis_limbs[3].element });
        prime_limbs.emplace_back(
            std::array<field_t<C>, 2>{ rom_data[i].x.prime_basis_limb, rom_data[i].y.prime_basis_limb });
    }
    std::array<twin_rom_table<C>, 5> output_tables;
    output_tables[0] = twin_rom_table<C>(x_lo_limbs);
    output_tables[1] = twin_rom_table<C>(x_hi_limbs);
    output_tables[2] = twin_rom_table<C>(y_lo_limbs);
    output_tables[3] = twin_rom_table<C>(y_hi_limbs);
    output_tables[4] = twin_rom_table<C>(prime_limbs);
    return output_tables;
}

template <typename C, class Fq, class Fr, class G>
template <size_t, typename>
element<C, Fq, Fr, G> element<C, Fq, Fr, G>::read_group_element_rom_tables(
    const std::array<twin_rom_table<C>, 5>& tables, const field_t<C>& index, const std::array<uint256_t, 8>& limb_max)
{
    const auto xlo = tables[0][index];
    const auto xhi = tables[1][index];
    const auto ylo = tables[2][index];
    const auto yhi = tables[3][index];
    const auto xyprime = tables[4][index];

    Fq x_fq(xlo[0], xlo[1], xhi[0], xhi[1], xyprime[0]);
    Fq y_fq(ylo[0], ylo[1], yhi[0], yhi[1], xyprime[1]);
    x_fq.binary_basis_limbs[0].maximum_value = limb_max[0];
    x_fq.binary_basis_limbs[1].maximum_value = limb_max[1];
    x_fq.binary_basis_limbs[2].maximum_value = limb_max[2];
    x_fq.binary_basis_limbs[3].maximum_value = limb_max[3];
    y_fq.binary_basis_limbs[0].maximum_value = limb_max[4];
    y_fq.binary_basis_limbs[1].maximum_value = limb_max[5];
    y_fq.binary_basis_limbs[2].maximum_value = limb_max[6];
    y_fq.binary_basis_limbs[3].maximum_value = limb_max[7];

    const auto output = element(x_fq, y_fq);
    return output;
}

template <typename C, class Fq, class Fr, class G>
template <typename X>
element<C, Fq, Fr, G>::four_bit_table_plookup<X>::four_bit_table_plookup(const element& input)
{
    element d2 = input.dbl();

    element_table[8] = input;
    for (size_t i = 9; i < 16; ++i) {
        element_table[i] = element_table[i - 1] + d2;
    }
    for (size_t i = 0; i < 8; ++i) {
        element_table[i] = (-element_table[15 - i]);
    }

    coordinates = create_group_element_rom_tables<16>(element_table, limb_max);
}

template <typename C, class Fq, class Fr, class G>
template <typename X>
element<C, Fq, Fr, G> element<C, Fq, Fr, G>::four_bit_table_plookup<X>::operator[](const field_t<C>& index) const
{
    return read_group_element_rom_tables<16>(coordinates, index, limb_max);
}

template <class C, class Fq, class Fr, class G>
template <typename X>
element<C, Fq, Fr, G> element<C, Fq, Fr, G>::eight_bit_fixed_base_table<X>::operator[](const field_t<C>& index) const
{
    const auto get_plookup_tags = [this]() {
        switch (curve_type) {
        case CurveType::SECP256K1: {
            return std::array<MultiTableId, 5>{
                use_endomorphism ? MultiTableId::SECP256K1_XLO_ENDO : MultiTableId::SECP256K1_XLO,
                use_endomorphism ? MultiTableId::SECP256K1_XHI_ENDO : MultiTableId::SECP256K1_XHI,
                MultiTableId::SECP256K1_YLO,
                MultiTableId::SECP256K1_YHI,
                use_endomorphism ? MultiTableId::SECP256K1_XYPRIME_ENDO : MultiTableId::SECP256K1_XYPRIME,
            };
        }
        case CurveType::BN254: {
            return std::array<MultiTableId, 5>{
                use_endomorphism ? MultiTableId::BN254_XLO_ENDO : MultiTableId::BN254_XLO,
                use_endomorphism ? MultiTableId::BN254_XHI_ENDO : MultiTableId::BN254_XHI,
                MultiTableId::BN254_YLO,
                MultiTableId::BN254_YHI,
                use_endomorphism ? MultiTableId::BN254_XYPRIME_ENDO : MultiTableId::BN254_XYPRIME,
            };
        }
        default: {
            return std::array<MultiTableId, 5>{
                use_endomorphism ? MultiTableId::BN254_XLO_ENDO : MultiTableId::BN254_XLO,
                use_endomorphism ? MultiTableId::BN254_XHI_ENDO : MultiTableId::BN254_XHI,
                MultiTableId::BN254_YLO,
                MultiTableId::BN254_YHI,
                use_endomorphism ? MultiTableId::BN254_XYPRIME_ENDO : MultiTableId::BN254_XYPRIME,
            };
        }
        }
    };

    const auto tags = get_plookup_tags();

    const auto xlo = plookup_read<C>::read_pair_from_table(tags[0], index);
    const auto xhi = plookup_read<C>::read_pair_from_table(tags[1], index);
    const auto ylo = plookup_read<C>::read_pair_from_table(tags[2], index);
    const auto yhi = plookup_read<C>::read_pair_from_table(tags[3], index);
    const auto xyprime = plookup_read<C>::read_pair_from_table(tags[4], index);

    Fq x = Fq(xlo.first, xlo.second, xhi.first, xhi.second, xyprime.first);
    Fq y = Fq(ylo.first, ylo.second, yhi.first, yhi.second, xyprime.second);

    if (use_endomorphism) {
        y = -y;
    }

    return element(x, y);
}

template <typename C, class Fq, class Fr, class G>
template <typename X>
element<C, Fq, Fr, G> element<C, Fq, Fr, G>::eight_bit_fixed_base_table<X>::operator[](const size_t index) const
{
    return operator[](field_t<C>(index));
}

/**
 * lookup_table_plookup
 **/
template <typename C, class Fq, class Fr, class G>
template <size_t length, typename X>
element<C, Fq, Fr, G>::lookup_table_plookup<length, X>::lookup_table_plookup(const std::array<element, length>& inputs)
{
    if constexpr (length == 2) {
        auto [A0, A1] = inputs[1].add_sub(inputs[0]);
        element_table[0] = A0;
        element_table[1] = A1;
    } else if constexpr (length == 3) {
        auto [R0, R1] = inputs[1].add_sub(inputs[0]); // B ± A

        auto [T0, T1] = inputs[2].add_sub(R0); // C ± (B + A)
        auto [T2, T3] = inputs[2].add_sub(R1); // C ± (B - A)

        element_table[0] = T0;
        element_table[1] = T2;
        element_table[2] = T3;
        element_table[3] = T1;
    } else if constexpr (length == 4) {
        auto [T0, T1] = inputs[1].add_sub(inputs[0]); // B ± A
        auto [T2, T3] = inputs[3].add_sub(inputs[2]); // D ± C

        auto [F0, F3] = T2.add_sub(T0); // (D + C) ± (B + A)
        auto [F1, F2] = T2.add_sub(T1); // (D + C) ± (B - A)
        auto [F4, F7] = T3.add_sub(T0); // (D - C) ± (B + A)
        auto [F5, F6] = T3.add_sub(T1); // (D - C) ± (B - A)

        element_table[0] = F0;
        element_table[1] = F1;
        element_table[2] = F2;
        element_table[3] = F3;
        element_table[4] = F4;
        element_table[5] = F5;
        element_table[6] = F6;
        element_table[7] = F7;
    } else if constexpr (length == 5) {
        auto [A0, A1] = inputs[1].add_sub(inputs[0]); // B ± A
        auto [T2, T3] = inputs[3].add_sub(inputs[2]); // D ± C

        auto [E0, E3] = inputs[4].add_sub(T2); // E ± (D + C)
        auto [E1, E2] = inputs[4].add_sub(T3); // E ± (D - C)

        auto [F0, F3] = E0.add_sub(A0);
        auto [F1, F2] = E0.add_sub(A1);
        auto [F4, F7] = E1.add_sub(A0);
        auto [F5, F6] = E1.add_sub(A1);
        auto [F8, F11] = E2.add_sub(A0);
        auto [F9, F10] = E2.add_sub(A1);
        auto [F12, F15] = E3.add_sub(A0);
        auto [F13, F14] = E3.add_sub(A1);

        element_table[0] = F0;
        element_table[1] = F1;
        element_table[2] = F2;
        element_table[3] = F3;
        element_table[4] = F4;
        element_table[5] = F5;
        element_table[6] = F6;
        element_table[7] = F7;
        element_table[8] = F8;
        element_table[9] = F9;
        element_table[10] = F10;
        element_table[11] = F11;
        element_table[12] = F12;
        element_table[13] = F13;
        element_table[14] = F14;
        element_table[15] = F15;
    } else if constexpr (length == 6) {
        // 44 adds! Only use this if it saves us adding another table to a multi-scalar-multiplication

        auto [A0, A1] = inputs[1].add_sub(inputs[0]);
        auto [E0, E1] = inputs[4].add_sub(inputs[3]);
        auto [C0, C3] = inputs[2].add_sub(A0);
        auto [C1, C2] = inputs[2].add_sub(A1);

        auto [F0, F3] = inputs[5].add_sub(E0);
        auto [F1, F2] = inputs[5].add_sub(E1);

        auto [R0, R7] = F0.add_sub(C0);
        auto [R1, R6] = F0.add_sub(C1);
        auto [R2, R5] = F0.add_sub(C2);
        auto [R3, R4] = F0.add_sub(C3);

        auto [S0, S7] = F1.add_sub(C0);
        auto [S1, S6] = F1.add_sub(C1);
        auto [S2, S5] = F1.add_sub(C2);
        auto [S3, S4] = F1.add_sub(C3);

        auto [U0, U7] = F2.add_sub(C0);
        auto [U1, U6] = F2.add_sub(C1);
        auto [U2, U5] = F2.add_sub(C2);
        auto [U3, U4] = F2.add_sub(C3);

        auto [W0, W7] = F3.add_sub(C0);
        auto [W1, W6] = F3.add_sub(C1);
        auto [W2, W5] = F3.add_sub(C2);
        auto [W3, W4] = F3.add_sub(C3);

        element_table[0] = R0;
        element_table[1] = R1;
        element_table[2] = R2;
        element_table[3] = R3;
        element_table[4] = R4;
        element_table[5] = R5;
        element_table[6] = R6;
        element_table[7] = R7;

        element_table[8] = S0;
        element_table[9] = S1;
        element_table[10] = S2;
        element_table[11] = S3;
        element_table[12] = S4;
        element_table[13] = S5;
        element_table[14] = S6;
        element_table[15] = S7;

        element_table[16] = U0;
        element_table[17] = U1;
        element_table[18] = U2;
        element_table[19] = U3;
        element_table[20] = U4;
        element_table[21] = U5;
        element_table[22] = U6;
        element_table[23] = U7;

        element_table[24] = W0;
        element_table[25] = W1;
        element_table[26] = W2;
        element_table[27] = W3;
        element_table[28] = W4;
        element_table[29] = W5;
        element_table[30] = W6;
        element_table[31] = W7;
    } else if constexpr (length == 7) {
        // 82 adds! This one is not worth using...

        element A0 = inputs[1] + inputs[0]; // B + A
        element A1 = inputs[1] - inputs[0]; // B - A

        element D0 = inputs[3] + inputs[2]; // D + C
        element D1 = inputs[3] - inputs[2]; // D - C

        element E0 = D0 + A0; // D + C + B + A
        element E1 = D0 + A1; // D + C + B - A
        element E2 = D0 - A1; // D + C - B + A
        element E3 = D0 - A0; // D + C - B - A
        element E4 = D1 + A0; // D - C + B + A
        element E5 = D1 + A1; // D - C + B - A
        element E6 = D1 - A1; // D - C - B + A
        element E7 = D1 - A0; // D - C - B - A

        element F0 = inputs[5] + inputs[4]; // F + E
        element F1 = inputs[5] - inputs[4]; // F - E

        element G0 = inputs[6] + F0; // G + F + E
        element G1 = inputs[6] + F1; // G + F - E
        element G2 = inputs[6] - F1; // G - F + E
        element G3 = inputs[6] - F0; // G - F - E

        element_table[0] = G0 + E0;
        element_table[1] = G0 + E1;
        element_table[2] = G0 + E2;
        element_table[3] = G0 + E3;
        element_table[4] = G0 + E4;
        element_table[5] = G0 + E5;
        element_table[6] = G0 + E6;
        element_table[7] = G0 + E7;
        element_table[8] = G0 - E7;
        element_table[9] = G0 - E6;
        element_table[10] = G0 - E5;
        element_table[11] = G0 - E4;
        element_table[12] = G0 - E3;
        element_table[13] = G0 - E2;
        element_table[14] = G0 - E1;
        element_table[15] = G0 - E0;
        element_table[16] = G1 + E0;
        element_table[17] = G1 + E1;
        element_table[18] = G1 + E2;
        element_table[19] = G1 + E3;
        element_table[20] = G1 + E4;
        element_table[21] = G1 + E5;
        element_table[22] = G1 + E6;
        element_table[23] = G1 + E7;
        element_table[24] = G1 - E7;
        element_table[25] = G1 - E6;
        element_table[26] = G1 - E5;
        element_table[27] = G1 - E4;
        element_table[28] = G1 - E3;
        element_table[29] = G1 - E2;
        element_table[30] = G1 - E1;
        element_table[31] = G1 - E0;
        element_table[32] = G2 + E0;
        element_table[33] = G2 + E1;
        element_table[34] = G2 + E2;
        element_table[35] = G2 + E3;
        element_table[36] = G2 + E4;
        element_table[37] = G2 + E5;
        element_table[38] = G2 + E6;
        element_table[39] = G2 + E7;
        element_table[40] = G2 - E7;
        element_table[41] = G2 - E6;
        element_table[42] = G2 - E5;
        element_table[43] = G2 - E4;
        element_table[44] = G2 - E3;
        element_table[45] = G2 - E2;
        element_table[46] = G2 - E1;
        element_table[47] = G2 - E0;
        element_table[48] = G3 + E0;
        element_table[49] = G3 + E1;
        element_table[50] = G3 + E2;
        element_table[51] = G3 + E3;
        element_table[52] = G3 + E4;
        element_table[53] = G3 + E5;
        element_table[54] = G3 + E6;
        element_table[55] = G3 + E7;
        element_table[56] = G3 - E7;
        element_table[57] = G3 - E6;
        element_table[58] = G3 - E5;
        element_table[59] = G3 - E4;
        element_table[60] = G3 - E3;
        element_table[61] = G3 - E2;
        element_table[62] = G3 - E1;
        element_table[63] = G3 - E0;
    }
    for (size_t i = 0; i < table_size / 2; ++i) {
        element_table[i + table_size / 2] = (-element_table[table_size / 2 - 1 - i]);
    }
    coordinates = create_group_element_rom_tables<table_size>(element_table, limb_max);
}

template <typename C, class Fq, class Fr, class G>
template <size_t length, typename X>
element<C, Fq, Fr, G> element<C, Fq, Fr, G>::lookup_table_plookup<length, X>::get(
    const std::array<bool_t<C>, length>& bits) const
{
    std::vector<field_t<C>> accumulators;
    for (size_t i = 0; i < length; ++i) {
        accumulators.emplace_back(field_t<C>(bits[i]) * (1ULL << i));
    }
    field_t<C> index = field_t<C>::accumulate(accumulators);
    return read_group_element_rom_tables<table_size>(coordinates, index, limb_max);
}

/**
 * lookup_table_base
 **/
template <typename C, class Fq, class Fr, class G>
template <size_t length>
element<C, Fq, Fr, G>::lookup_table_base<length>::lookup_table_base(const std::array<element, length>& inputs)
{
    static_assert(length <= 4 && length >= 2);
    if constexpr (length == 2) {
        twin0 = inputs[1] + inputs[0];
        twin1 = inputs[1] - inputs[0];
        element_table[0] = twin0;
        element_table[1] = twin1;
    } else if constexpr (length == 3) {
        element T0 = inputs[1] + inputs[0];
        element T1 = inputs[1] - inputs[0];
        element_table[0] = inputs[2] + T0; // C + B + A
        element_table[1] = inputs[2] + T1; // C + B - A
        element_table[2] = inputs[2] - T1; // C - B + A
        element_table[3] = inputs[2] - T0; // C - B - A

        x_b0_table = field_t<C>::preprocess_two_bit_table(element_table[0].x.binary_basis_limbs[0].element,
                                                          element_table[1].x.binary_basis_limbs[0].element,
                                                          element_table[2].x.binary_basis_limbs[0].element,
                                                          element_table[3].x.binary_basis_limbs[0].element);
        x_b1_table = field_t<C>::preprocess_two_bit_table(element_table[0].x.binary_basis_limbs[1].element,
                                                          element_table[1].x.binary_basis_limbs[1].element,
                                                          element_table[2].x.binary_basis_limbs[1].element,
                                                          element_table[3].x.binary_basis_limbs[1].element);
        x_b2_table = field_t<C>::preprocess_two_bit_table(element_table[0].x.binary_basis_limbs[2].element,
                                                          element_table[1].x.binary_basis_limbs[2].element,
                                                          element_table[2].x.binary_basis_limbs[2].element,
                                                          element_table[3].x.binary_basis_limbs[2].element);
        x_b3_table = field_t<C>::preprocess_two_bit_table(element_table[0].x.binary_basis_limbs[3].element,
                                                          element_table[1].x.binary_basis_limbs[3].element,
                                                          element_table[2].x.binary_basis_limbs[3].element,
                                                          element_table[3].x.binary_basis_limbs[3].element);

        y_b0_table = field_t<C>::preprocess_two_bit_table(element_table[0].y.binary_basis_limbs[0].element,
                                                          element_table[1].y.binary_basis_limbs[0].element,
                                                          element_table[2].y.binary_basis_limbs[0].element,
                                                          element_table[3].y.binary_basis_limbs[0].element);
        y_b1_table = field_t<C>::preprocess_two_bit_table(element_table[0].y.binary_basis_limbs[1].element,
                                                          element_table[1].y.binary_basis_limbs[1].element,
                                                          element_table[2].y.binary_basis_limbs[1].element,
                                                          element_table[3].y.binary_basis_limbs[1].element);
        y_b2_table = field_t<C>::preprocess_two_bit_table(element_table[0].y.binary_basis_limbs[2].element,
                                                          element_table[1].y.binary_basis_limbs[2].element,
                                                          element_table[2].y.binary_basis_limbs[2].element,
                                                          element_table[3].y.binary_basis_limbs[2].element);
        y_b3_table = field_t<C>::preprocess_two_bit_table(element_table[0].y.binary_basis_limbs[3].element,
                                                          element_table[1].y.binary_basis_limbs[3].element,
                                                          element_table[2].y.binary_basis_limbs[3].element,
                                                          element_table[3].y.binary_basis_limbs[3].element);
    } else if constexpr (length == 4) {
        element T0 = inputs[1] + inputs[0];
        element T1 = inputs[1] - inputs[0];
        element T2 = inputs[3] + inputs[2];
        element T3 = inputs[3] - inputs[2];

        element_table[0] = T2 + T0; // D + C + B + A
        element_table[1] = T2 + T1; // D + C + B - A
        element_table[2] = T2 - T1; // D + C - B + A
        element_table[3] = T2 - T0; // D + C - B - A
        element_table[4] = T3 + T0; // D - C + B + A
        element_table[5] = T3 + T1; // D - C + B - A
        element_table[6] = T3 - T1; // D - C - B + A
        element_table[7] = T3 - T0; // D - C - B - A

        x_b0_table = field_t<C>::preprocess_three_bit_table(element_table[0].x.binary_basis_limbs[0].element,
                                                            element_table[1].x.binary_basis_limbs[0].element,
                                                            element_table[2].x.binary_basis_limbs[0].element,
                                                            element_table[3].x.binary_basis_limbs[0].element,
                                                            element_table[4].x.binary_basis_limbs[0].element,
                                                            element_table[5].x.binary_basis_limbs[0].element,
                                                            element_table[6].x.binary_basis_limbs[0].element,
                                                            element_table[7].x.binary_basis_limbs[0].element);
        x_b1_table = field_t<C>::preprocess_three_bit_table(element_table[0].x.binary_basis_limbs[1].element,
                                                            element_table[1].x.binary_basis_limbs[1].element,
                                                            element_table[2].x.binary_basis_limbs[1].element,
                                                            element_table[3].x.binary_basis_limbs[1].element,
                                                            element_table[4].x.binary_basis_limbs[1].element,
                                                            element_table[5].x.binary_basis_limbs[1].element,
                                                            element_table[6].x.binary_basis_limbs[1].element,
                                                            element_table[7].x.binary_basis_limbs[1].element);
        x_b2_table = field_t<C>::preprocess_three_bit_table(element_table[0].x.binary_basis_limbs[2].element,
                                                            element_table[1].x.binary_basis_limbs[2].element,
                                                            element_table[2].x.binary_basis_limbs[2].element,
                                                            element_table[3].x.binary_basis_limbs[2].element,
                                                            element_table[4].x.binary_basis_limbs[2].element,
                                                            element_table[5].x.binary_basis_limbs[2].element,
                                                            element_table[6].x.binary_basis_limbs[2].element,
                                                            element_table[7].x.binary_basis_limbs[2].element);
        x_b3_table = field_t<C>::preprocess_three_bit_table(element_table[0].x.binary_basis_limbs[3].element,
                                                            element_table[1].x.binary_basis_limbs[3].element,
                                                            element_table[2].x.binary_basis_limbs[3].element,
                                                            element_table[3].x.binary_basis_limbs[3].element,
                                                            element_table[4].x.binary_basis_limbs[3].element,
                                                            element_table[5].x.binary_basis_limbs[3].element,
                                                            element_table[6].x.binary_basis_limbs[3].element,
                                                            element_table[7].x.binary_basis_limbs[3].element);

        y_b0_table = field_t<C>::preprocess_three_bit_table(element_table[0].y.binary_basis_limbs[0].element,
                                                            element_table[1].y.binary_basis_limbs[0].element,
                                                            element_table[2].y.binary_basis_limbs[0].element,
                                                            element_table[3].y.binary_basis_limbs[0].element,
                                                            element_table[4].y.binary_basis_limbs[0].element,
                                                            element_table[5].y.binary_basis_limbs[0].element,
                                                            element_table[6].y.binary_basis_limbs[0].element,
                                                            element_table[7].y.binary_basis_limbs[0].element);
        y_b1_table = field_t<C>::preprocess_three_bit_table(element_table[0].y.binary_basis_limbs[1].element,
                                                            element_table[1].y.binary_basis_limbs[1].element,
                                                            element_table[2].y.binary_basis_limbs[1].element,
                                                            element_table[3].y.binary_basis_limbs[1].element,
                                                            element_table[4].y.binary_basis_limbs[1].element,
                                                            element_table[5].y.binary_basis_limbs[1].element,
                                                            element_table[6].y.binary_basis_limbs[1].element,
                                                            element_table[7].y.binary_basis_limbs[1].element);
        y_b2_table = field_t<C>::preprocess_three_bit_table(element_table[0].y.binary_basis_limbs[2].element,
                                                            element_table[1].y.binary_basis_limbs[2].element,
                                                            element_table[2].y.binary_basis_limbs[2].element,
                                                            element_table[3].y.binary_basis_limbs[2].element,
                                                            element_table[4].y.binary_basis_limbs[2].element,
                                                            element_table[5].y.binary_basis_limbs[2].element,
                                                            element_table[6].y.binary_basis_limbs[2].element,
                                                            element_table[7].y.binary_basis_limbs[2].element);
        y_b3_table = field_t<C>::preprocess_three_bit_table(element_table[0].y.binary_basis_limbs[3].element,
                                                            element_table[1].y.binary_basis_limbs[3].element,
                                                            element_table[2].y.binary_basis_limbs[3].element,
                                                            element_table[3].y.binary_basis_limbs[3].element,
                                                            element_table[4].y.binary_basis_limbs[3].element,
                                                            element_table[5].y.binary_basis_limbs[3].element,
                                                            element_table[6].y.binary_basis_limbs[3].element,
                                                            element_table[7].y.binary_basis_limbs[3].element);
    }
}

template <typename C, class Fq, class Fr, class G>
template <size_t length>
element<C, Fq, Fr, G> element<C, Fq, Fr, G>::lookup_table_base<length>::get(
    const std::array<bool_t<C>, length>& bits) const
{
    static_assert(length <= 4 && length >= 2);

    if constexpr (length == 2) {
        bool_t<C> table_selector = bits[0] ^ bits[1];
        bool_t<C> sign_selector = bits[1];
        Fq to_add_x = twin0.x.conditional_select(twin1.x, table_selector);
        Fq to_add_y = twin0.y.conditional_select(twin1.y, table_selector);
        element to_add(to_add_x, to_add_y.conditional_negate(sign_selector));
        return to_add;
    } else if constexpr (length == 3) {
        bool_t<C> t0 = bits[2] ^ bits[0];
        bool_t<C> t1 = bits[2] ^ bits[1];

        field_t<C> x_b0 = field_t<C>::select_from_two_bit_table(x_b0_table, t1, t0);
        field_t<C> x_b1 = field_t<C>::select_from_two_bit_table(x_b1_table, t1, t0);
        field_t<C> x_b2 = field_t<C>::select_from_two_bit_table(x_b2_table, t1, t0);
        field_t<C> x_b3 = field_t<C>::select_from_two_bit_table(x_b3_table, t1, t0);

        field_t<C> y_b0 = field_t<C>::select_from_two_bit_table(y_b0_table, t1, t0);
        field_t<C> y_b1 = field_t<C>::select_from_two_bit_table(y_b1_table, t1, t0);
        field_t<C> y_b2 = field_t<C>::select_from_two_bit_table(y_b2_table, t1, t0);
        field_t<C> y_b3 = field_t<C>::select_from_two_bit_table(y_b3_table, t1, t0);

        Fq to_add_x;
        Fq to_add_y;
        to_add_x.binary_basis_limbs[0] = typename Fq::Limb(x_b0, Fq::DEFAULT_MAXIMUM_LIMB);
        to_add_x.binary_basis_limbs[1] = typename Fq::Limb(x_b1, Fq::DEFAULT_MAXIMUM_LIMB);
        to_add_x.binary_basis_limbs[2] = typename Fq::Limb(x_b2, Fq::DEFAULT_MAXIMUM_LIMB);
        to_add_x.binary_basis_limbs[3] = typename Fq::Limb(x_b3, Fq::DEFAULT_MAXIMUM_MOST_SIGNIFICANT_LIMB);
        to_add_x.prime_basis_limb = to_add_x.binary_basis_limbs[0].element.add_two(
            to_add_x.binary_basis_limbs[1].element * Fq::shift_1, to_add_x.binary_basis_limbs[2].element * Fq::shift_2);
        to_add_x.prime_basis_limb += to_add_x.binary_basis_limbs[3].element * Fq::shift_3;

        to_add_y.binary_basis_limbs[0] = typename Fq::Limb(y_b0, Fq::DEFAULT_MAXIMUM_LIMB);
        to_add_y.binary_basis_limbs[1] = typename Fq::Limb(y_b1, Fq::DEFAULT_MAXIMUM_LIMB);
        to_add_y.binary_basis_limbs[2] = typename Fq::Limb(y_b2, Fq::DEFAULT_MAXIMUM_LIMB);
        to_add_y.binary_basis_limbs[3] = typename Fq::Limb(y_b3, Fq::DEFAULT_MAXIMUM_MOST_SIGNIFICANT_LIMB);
        to_add_y.prime_basis_limb = to_add_y.binary_basis_limbs[0].element.add_two(
            to_add_y.binary_basis_limbs[1].element * Fq::shift_1, to_add_y.binary_basis_limbs[2].element * Fq::shift_2);
        to_add_y.prime_basis_limb += to_add_y.binary_basis_limbs[3].element * Fq::shift_3;
        element to_add(to_add_x, to_add_y.conditional_negate(bits[2]));

        return to_add;
    } else if constexpr (length == 4) {
        bool_t<C> t0 = bits[3] ^ bits[0];
        bool_t<C> t1 = bits[3] ^ bits[1];
        bool_t<C> t2 = bits[3] ^ bits[2];

        field_t<C> x_b0 = field_t<C>::select_from_three_bit_table(x_b0_table, t2, t1, t0);
        field_t<C> x_b1 = field_t<C>::select_from_three_bit_table(x_b1_table, t2, t1, t0);
        field_t<C> x_b2 = field_t<C>::select_from_three_bit_table(x_b2_table, t2, t1, t0);
        field_t<C> x_b3 = field_t<C>::select_from_three_bit_table(x_b3_table, t2, t1, t0);

        field_t<C> y_b0 = field_t<C>::select_from_three_bit_table(y_b0_table, t2, t1, t0);
        field_t<C> y_b1 = field_t<C>::select_from_three_bit_table(y_b1_table, t2, t1, t0);
        field_t<C> y_b2 = field_t<C>::select_from_three_bit_table(y_b2_table, t2, t1, t0);
        field_t<C> y_b3 = field_t<C>::select_from_three_bit_table(y_b3_table, t2, t1, t0);

        Fq to_add_x;
        Fq to_add_y;
        to_add_x.binary_basis_limbs[0] = typename Fq::Limb(x_b0, Fq::DEFAULT_MAXIMUM_LIMB);
        to_add_x.binary_basis_limbs[1] = typename Fq::Limb(x_b1, Fq::DEFAULT_MAXIMUM_LIMB);
        to_add_x.binary_basis_limbs[2] = typename Fq::Limb(x_b2, Fq::DEFAULT_MAXIMUM_LIMB);
        to_add_x.binary_basis_limbs[3] = typename Fq::Limb(x_b3, Fq::DEFAULT_MAXIMUM_MOST_SIGNIFICANT_LIMB);
        to_add_x.prime_basis_limb = to_add_x.binary_basis_limbs[0].element.add_two(
            to_add_x.binary_basis_limbs[1].element * Fq::shift_1, to_add_x.binary_basis_limbs[2].element * Fq::shift_2);
        to_add_x.prime_basis_limb += to_add_x.binary_basis_limbs[3].element * Fq::shift_3;

        to_add_y.binary_basis_limbs[0] = typename Fq::Limb(y_b0, Fq::DEFAULT_MAXIMUM_LIMB);
        to_add_y.binary_basis_limbs[1] = typename Fq::Limb(y_b1, Fq::DEFAULT_MAXIMUM_LIMB);
        to_add_y.binary_basis_limbs[2] = typename Fq::Limb(y_b2, Fq::DEFAULT_MAXIMUM_LIMB);
        to_add_y.binary_basis_limbs[3] = typename Fq::Limb(y_b3, Fq::DEFAULT_MAXIMUM_MOST_SIGNIFICANT_LIMB);
        to_add_y.prime_basis_limb = to_add_y.binary_basis_limbs[0].element.add_two(
            to_add_y.binary_basis_limbs[1].element * Fq::shift_1, to_add_y.binary_basis_limbs[2].element * Fq::shift_2);
        to_add_y.prime_basis_limb += to_add_y.binary_basis_limbs[3].element * Fq::shift_3;

        element to_add(to_add_x, to_add_y.conditional_negate(bits[3]));

        return to_add;
    }
    return element::one(bits[0].get_context());
}
} // namespace stdlib
} // namespace bb::plonk
