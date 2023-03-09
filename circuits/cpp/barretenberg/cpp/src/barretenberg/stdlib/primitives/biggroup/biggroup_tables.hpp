#pragma once
namespace plonk {
namespace stdlib {

template <typename C, class Fq, class Fr, class G>
template <size_t num_elements, typename>
std::array<twin_rom_table<C>, 5> element<C, Fq, Fr, G>::create_group_element_rom_tables(
    const std::array<element, num_elements>& rom_data)
{

    std::vector<std::array<field_t<C>, 2>> x_lo_limbs;
    std::vector<std::array<field_t<C>, 2>> x_hi_limbs;
    std::vector<std::array<field_t<C>, 2>> y_lo_limbs;
    std::vector<std::array<field_t<C>, 2>> y_hi_limbs;
    std::vector<std::array<field_t<C>, 2>> prime_limbs;

    for (size_t i = 0; i < num_elements; ++i) {
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
    const std::array<twin_rom_table<C>, 5>& tables, const field_t<C>& index)
{
    const auto xlo = tables[0][index];
    const auto xhi = tables[1][index];
    const auto ylo = tables[2][index];
    const auto yhi = tables[3][index];
    const auto xyprime = tables[4][index];

    Fq x_fq(xlo[0], xlo[1], xhi[0], xhi[1], xyprime[0]);
    Fq y_fq(ylo[0], ylo[1], yhi[0], yhi[1], xyprime[1]);
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
        element_table[i] = (-element_table[15 - i]).reduce();
    }

    coordinates = create_group_element_rom_tables<16>(element_table);
}

template <typename C, class Fq, class Fr, class G>
template <typename X>
element<C, Fq, Fr, G> element<C, Fq, Fr, G>::four_bit_table_plookup<X>::operator[](const field_t<C>& index) const
{
    return read_group_element_rom_tables<16>(coordinates, index);
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

    const auto xlo = plookup_read::read_pair_from_table(tags[0], index);
    const auto xhi = plookup_read::read_pair_from_table(tags[1], index);
    const auto ylo = plookup_read::read_pair_from_table(tags[2], index);
    const auto yhi = plookup_read::read_pair_from_table(tags[3], index);
    const auto xyprime = plookup_read::read_pair_from_table(tags[4], index);

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
        element_table[0] = inputs[1] + inputs[0];
        element_table[1] = inputs[1] - inputs[0];
    } else if constexpr (length == 3) {
        element R0 = inputs[1] + inputs[0];
        element R1 = inputs[1] - inputs[0];
        element_table[0] = inputs[2] + R0; // C + B + A
        element_table[1] = inputs[2] + R1; // C + B - A
        element_table[2] = inputs[2] - R1; // C - B + A
        element_table[3] = inputs[2] - R0; // C - B - A
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
    } else if constexpr (length == 5) {
        element A0 = inputs[1] + inputs[0]; // B + A
        element A1 = inputs[1] - inputs[0]; // B - A

        element T2 = inputs[3] + inputs[2]; // D + C
        element T3 = inputs[3] - inputs[2]; // D - C

        element E0 = inputs[4] + T2; // E + D + C // 0 0 0
        element E1 = inputs[4] + T3; // E + D - C // 0 0 1
        element E2 = inputs[4] - T3; // E - D + C // 0 1 0
        element E3 = inputs[4] - T2; // E - D - C // 0 1 1

        element_table[0] = E0 + A0;  // E + D + C + B + A // 0 0 0 0 0
        element_table[1] = E0 + A1;  // E + D + C + B - A // 0 0 0 0 1
        element_table[2] = E0 - A1;  // E + D + C - B + A // 0 0 0 1 0
        element_table[3] = E0 - A0;  // E + D + C - B - A // 0 0 0 1 1
        element_table[4] = E1 + A0;  // E + D - C + B + A // 0 0 1 0 0
        element_table[5] = E1 + A1;  // E + D - C + B - A // 0 0 1 0 1
        element_table[6] = E1 - A1;  // E + D - C - B + A // 0 0 1 1 0
        element_table[7] = E1 - A0;  // E + D - C - B - A // 0 0 1 1 1
        element_table[8] = E2 + A0;  // E - D + C + B + A // 0 1 0 0 0
        element_table[9] = E2 + A1;  // E - D + C + B - A // 0 1 0 0 1
        element_table[10] = E2 - A1; // E - D + C - B + A // 0 1 0 1 0
        element_table[11] = E2 - A0; // E - D - C - B - A // 0 1 0 1 1
        element_table[12] = E3 + A0; // E - D - C + B + A // 0 1 1 0 0
        element_table[13] = E3 + A1; // E - D - C + B - A // 0 1 1 0 1
        element_table[14] = E3 - A1; // E - D - C - B + A // 0 1 1 1 0
        element_table[15] = E3 - A0; // E - D - C - B - A // 0 1 1 1 1
    } else if constexpr (length == 6) {
        // 44 adds! Only use this if it saves us adding another table to a multi-scalar-multiplication
        element A0 = inputs[1] + inputs[0]; // B + A
        element A1 = inputs[1] - inputs[0]; // B - A
        element E0 = inputs[4] + inputs[3]; // E + D
        element E1 = inputs[4] - inputs[3]; // E - D

        element C0 = inputs[2] + A0; //  C + B + A
        element C1 = inputs[2] + A1; //  C + B - A
        element C2 = inputs[2] - A1; //  C - B + A
        element C3 = inputs[2] - A0; //  C - B - A

        element F0 = inputs[5] + E0; // F + E + D
        element F1 = inputs[5] + E1; // F + E - D
        element F2 = inputs[5] - E1; // F - E + D
        element F3 = inputs[5] - E0; // F - E - E

        element_table[0] = F0 + C0;
        element_table[1] = F0 + C1;
        element_table[2] = F0 + C2;
        element_table[3] = F0 + C3;
        element_table[4] = F0 - C3;
        element_table[5] = F0 - C2;
        element_table[6] = F0 - C1;
        element_table[7] = F0 - C0;

        element_table[8] = F1 + C0;
        element_table[9] = F1 + C1;
        element_table[10] = F1 + C2;
        element_table[11] = F1 + C3;
        element_table[12] = F1 - C3;
        element_table[13] = F1 - C2;
        element_table[14] = F1 - C1;
        element_table[15] = F1 - C0;

        element_table[16] = F2 + C0;
        element_table[17] = F2 + C1;
        element_table[18] = F2 + C2;
        element_table[19] = F2 + C3;
        element_table[20] = F2 - C3;
        element_table[21] = F2 - C2;
        element_table[22] = F2 - C1;
        element_table[23] = F2 - C0;

        element_table[24] = F3 + C0;
        element_table[25] = F3 + C1;
        element_table[26] = F3 + C2;
        element_table[27] = F3 + C3;
        element_table[28] = F3 - C3;
        element_table[29] = F3 - C2;
        element_table[30] = F3 - C1;
        element_table[31] = F3 - C0;
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
        element_table[i + table_size / 2] = (-element_table[table_size / 2 - 1 - i]).reduce();
    }
    coordinates = create_group_element_rom_tables<table_size>(element_table);
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
    return read_group_element_rom_tables<table_size>(coordinates, index);
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
} // namespace plonk
