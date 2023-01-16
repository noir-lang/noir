#pragma once
#include <ecc/curves/secp256k1/secp256k1.hpp>

namespace plonk {
namespace stdlib {

/**
 * Split a secp256k1 Fr element into two 129 bit scalars `klo, khi`, where `scalar = klo + \lambda * khi mod n`
 *   where `\lambda` is the cube root of unity mod n, and `n` is the secp256k1 Fr modulus
 *
 * We return the wnaf representation of the two 129-bit scalars
 *
 * The wnaf representation includes `positive_skew` and `negative_skew` components,
 * because for both `klo, khi` EITHER `k < 2^{129}` OR `-k mod n < 2^{129}`.
 * If we have to negate the short scalar, the wnaf skew component flips sign.
 *
 * Outline of algorithm:
 *
 * We will use our wnaf elements to index a ROM table. ROM index values act like regular array indices,
 * i.e. start at 0, increase by 1 per index.
 * We need the wnaf format to follow the same structure.
 *
 * The mapping from wnaf value to lookup table point is as follows (example is 4-bit WNAF):
 *
 *  | wnaf witness value | wnaf real value | point representation |
 *  |--------------------|-----------------|----------------------|
 *  |                  0 |             -15 |              -15.[P] |
 *  |                  1 |             -13 |              -13.[P] |
 *  |                  2 |             -11 |              -11.[P] |
 *  |                  3 |              -9 |               -9.[P] |
 *  |                  4 |              -7 |               -7.[P] |
 *  |                  5 |              -5 |               -5.[P] |
 *  |                  6 |              -3 |               -3.[P] |
 *  |                  7 |              -1 |               -1.[P] |
 *  |                  8 |               1 |                1.[P] |
 *  |                  9 |               3 |                3.[P] |
 *  |                 10 |               5 |                5.[P] |
 *  |                 11 |               7 |                7.[P] |
 *  |                 12 |               9 |                9.[P] |
 *  |                 13 |              11 |               11.[P] |
 *  |                 14 |              13 |               13.[P] |
 *  |                 15 |              15 |               15.[P] |
 *  |--------------------|-----------------|----------------------|
 *
 * The transformation between the wnaf witness value `w` and the wnaf real value `v` is, for an `s`-bit window:
 *
 *                      s
 *          v = 2.w - (2 - 1)
 *
 * To reconstruct the 129-bit scalar multiplier `x` from wnaf values `w` (starting with most significant slice):
 *
 *                                                        m
 *                                                       ___
 *                                                      \     /          s      \    s.(m - i - 1)
 *           x =  positive_skew - negative_skew +        |    | 2.w  - (2  - 1) | . 2
 *                                                      /___  \    i            /
 *                                                       i=0
 *
 * N.B. `m` = number of rounds = (129 + s - 1) / s
 *
 * We can split the RHS into positive and negative components that are strictly positive:
 *
 *                                          m
 *                                         ___
 *                                        \     /    \    s.(m - i - 1)
 *                x_pos = positive_skew +  |    |2.w | . 2
 *                                        /___  \   i/
 *                                         i=0
 *
 *                                          m
 *                                         ___
 *                                        \     /  s     \    s.(m - i - 1)
 *                x_neg = negative_skew +  |    |(2  - 1)| . 2
 *                                        /___  \        /
 *                                         i=0
 *
 * By independently constructing `x_pos`, `x_neg`, we ensure we never underflow the native circuit modulus
 *
 * To reconstruct our wnaf components into a scalar, we perform the following (for each 129-bit slice klo, khi):
 *
 *      1. Compute the wnaf entries and range constrain each entry to be < 2^s
 *      2. Construct `x_pos`
 *      3. Construct `x_neg`
 *      4. Cast `x_pos, x_neg` into two Fr elements and compute `Fr reconstructed = Fr(x_pos) - Fr(x_neg)`
 *
 * This ensures that the only negation in performed in the Fr representation, removing the risk of underflow errors
 *
 * Once `klo, khi` have been reconstructed as Fr elements, we validate the following:
 *
 *      1. `scalar == Fr(klo) - Fr(khi) * Fr(\lambda)
 *
 * Finally, we return the wnaf representations of klo, khi including the skew
 **/
template <typename C, class Fq, class Fr, class G>
template <size_t wnaf_size, size_t lo_stagger, size_t hi_stagger>
typename element<C, Fq, Fr, G>::secp256k1_wnaf_pair element<C, Fq, Fr, G>::compute_secp256k1_endo_wnaf(const Fr& scalar)
{
    /**
     * The staggered offset describes the number of bits we want to remove from the input scalar before computing our
     * wnaf slices. This is to enable us to make repeated calls to the montgomery ladder algo when computing a
     * multi-scalar multiplication e.g. Consider an example with 2 points (A, B), using a 2-bit WNAF The typical
     * approach would be to perfomr a double-and-add algorithm, adding points into an accumulator ACC:
     *
     * ACC = ACC.dbl()
     * ACC = ACC.dbl()
     * ACC = ACC.add(A)
     * ACC = ACC.add(B)
     *
     * However, if the A and B WNAFs are offset by 1 bit each, we can perform the following:
     *
     * ACC = ACC.dbl()
     * ACC = ACC.add(A)
     * ACC = ACC.dbl()
     * ACC = ACC.add(B)
     *
     * which we can reduce to:
     *
     * ACC = ACC.montgomery_ladder(A)
     * ACC = ACC.montgomery_ladder(B)
     *
     * This is more efficient than the non-staggered approach as we save 1 non-native field multiplication when we
     * replace a DBL, ADD subroutine with a call to the montgomery ladder
     */
    C* ctx = scalar.context;

    constexpr size_t num_bits = 129;

    /**
     * @brief Compute WNAF of a single 129-bit scalar
     *
     * @param k Scalar
     * @param stagger The number of bits that are used in "staggering"
     * @param is_negative If it should be subtracted
     * @param is_lo True if it's the low scalar
     */
    const auto compute_single_wnaf = [ctx](const secp256k1::fr& k,
                                           const auto stagger,
                                           const bool is_negative,
                                           const bool is_lo = false) {
        // The number of rounds is the minimal required to cover the whole scalar with wnaf_size windows
        constexpr size_t num_rounds = ((num_bits + wnaf_size - 1) / wnaf_size);
        // Stagger mask is needed to retrieve the lowest bits that will not be used in montgomery ladder directly
        const uint64_t stagger_mask = (1ULL << stagger) - 1;
        // Stagger scalar represents the lower "staggered" bits that are not used in the ladder
        const uint64_t stagger_scalar = k.data[0] & stagger_mask;

        uint64_t wnaf_values[num_rounds] = { 0 };
        bool skew_without_stagger;
        uint256_t k_u256{ k.data[0], k.data[1], k.data[2], k.data[3] };
        k_u256 = k_u256 >> stagger;
        if (is_lo) {
            barretenberg::wnaf::fixed_wnaf<num_bits - lo_stagger, 1, wnaf_size>(
                &k_u256.data[0], &wnaf_values[0], skew_without_stagger, 0);
        } else {
            barretenberg::wnaf::fixed_wnaf<num_bits - hi_stagger, 1, wnaf_size>(
                &k_u256.data[0], &wnaf_values[0], skew_without_stagger, 0);
        }

        // Number of rounds that are needed to reconstruct the scalar without staggered bits
        const size_t num_rounds_excluding_stagger_bits = ((num_bits + wnaf_size - 1 - stagger) / wnaf_size);

        /**
         * @brief Compute the stagger-related part of WNAF and the final skew
         *
         * @param fragment_u64 Stagger-masked lower bits of the skalar
         * @param stagger The number of staggering bits
         * @param is_negative If the initial scalar is supposed to be subtracted
         * @param wnaf_skew The skew of the stagger-right-shifted part of the skalar
         *
         */
        const auto compute_staggered_wnaf_fragment =
            [](const uint64_t fragment_u64, const uint64_t stagger, bool is_negative, bool wnaf_skew) {
                // If there is not stagger then there is no need to change anyhing
                if (stagger == 0) {
                    return std::make_pair<uint64_t, bool>((uint64_t)0, (bool)wnaf_skew);
                }
                int fragment = static_cast<int>(fragment_u64);
                // Inverse the fragment if it's negative
                if (is_negative) {
                    fragment = -fragment;
                }
                // If the value is positive and there is a skew in wnaf, subtract 2ˢᵗᵃᵍᵍᵉʳ. If negative and there is
                // skew, then add
                if (!is_negative && wnaf_skew) {
                    fragment -= (1 << stagger);
                } else if (is_negative && wnaf_skew) {
                    fragment += (1 << stagger);
                }
                // If the lowest bit is zero, then set final skew to 1 and add 1 to the absolute value of the fragment
                bool output_skew = (fragment_u64 % 2) == 0;
                if (!is_negative && output_skew) {
                    fragment += 1;
                } else if (is_negative && output_skew) {
                    fragment -= 1;
                }

                uint64_t output_fragment;
                if (fragment < 0) {
                    output_fragment = static_cast<uint64_t>((int)((1ULL << (wnaf_size - 1))) + (fragment / 2 - 1));
                } else {
                    output_fragment = static_cast<uint64_t>((1ULL << (wnaf_size - 1)) - 1ULL +
                                                            (uint64_t)((uint64_t)fragment / 2 + 1));
                }

                return std::make_pair<uint64_t, bool>((uint64_t)output_fragment, (bool)output_skew);
            };

        // Compute the lowest fragment and final skew
        const auto [first_fragment, skew] =
            compute_staggered_wnaf_fragment(stagger_scalar, stagger, is_negative, skew_without_stagger);

        constexpr uint64_t wnaf_window_size = (1ULL << (wnaf_size - 1));
        /**
         * @brief Compute wnaf values, convert them into witness field elements and range constrain them
         *
         */
        const auto get_wnaf_wires = [ctx](uint64_t* wnaf_values, bool is_negative, size_t rounds) {
            std::vector<field_t<C>> wnaf_entries;
            for (size_t i = 0; i < rounds; ++i) {
                // Predicate == sign of current wnaf value
                bool predicate = bool((wnaf_values[i] >> 31U) & 1U);
                uint64_t offset_entry;
                // If the signs of current entry and the whole scalar are the same, then add the lowest bits of current
                // wnaf value to the windows size to form an entry. Otherwise, subract the lowest bits along with 1
                if ((!predicate && !is_negative) || (predicate && is_negative)) {
                    // TODO: Why is this mask fixed?
                    offset_entry = wnaf_window_size + (wnaf_values[i] & 0xffffff);
                } else {
                    offset_entry = wnaf_window_size - 1 - (wnaf_values[i] & 0xffffff);
                }
                field_t<C> entry(witness_t<C>(ctx, offset_entry));

                // TODO: Do these need to be range constrained? we use these witnesses
                // to index a size-16 ROM lookup table, which performs an implicit range constraint
                entry.create_range_constraint(wnaf_size);
                wnaf_entries.emplace_back(entry);
            }
            return wnaf_entries;
        };

        // Get wnaf witnesses
        std::vector<field_t<C>> wnaf = get_wnaf_wires(&wnaf_values[0], is_negative, num_rounds_excluding_stagger_bits);
        // Compute and constrain skews
        field_t<C> negative_skew = witness_t<C>(ctx, is_negative ? 0 : skew);
        field_t<C> positive_skew = witness_t<C>(ctx, is_negative ? skew : 0);
        negative_skew.create_range_constraint(1);
        positive_skew.create_range_constraint(1);
        (negative_skew + positive_skew).create_range_constraint(1);

        const auto reconstruct_bigfield_from_wnaf = [ctx](const std::vector<field_t<C>>& wnaf,
                                                          const field_t<C>& positive_skew,
                                                          const field_t<C>& stagger_fragment,
                                                          const size_t stagger,
                                                          const size_t rounds) {
            std::vector<field_t<C>> accumulator;
            // Collect positive wnaf entries for accumulation
            for (size_t i = 0; i < rounds; ++i) {
                field_t<C> entry = wnaf[rounds - 1 - i];
                entry *= static_cast<field_t<C>>(uint256_t(1) << (i * wnaf_size));
                accumulator.emplace_back(entry);
            }
            // Accumulate entries, shift by stagger and add the stagger itself
            field_t<C> sum = field_t<C>::accumulate(accumulator);
            sum = sum * field_t<C>(barretenberg::fr(1ULL << stagger));
            sum += (stagger_fragment);
            sum = sum.normalize();
            // TODO: improve efficiency by creating a constructor that does NOT require us to range constrain
            //       limbs (we already know (sum < 2^{130}))
            // Convert this value to bigfield element
            Fr reconstructed = Fr(sum, field_t<C>::from_witness_index(ctx, ctx->zero_idx), false);
            // Double the final value and add the skew
            reconstructed = (reconstructed + reconstructed).add_to_lower_limb(positive_skew, uint256_t(1));
            return reconstructed;
        };

        // Initialize stagger witness
        field_t<C> stagger_fragment = witness_t<C>(ctx, first_fragment);

        // Reconstruct bigfield x_pos
        Fr wnaf_sum = reconstruct_bigfield_from_wnaf(
            wnaf, positive_skew, stagger_fragment, stagger, num_rounds_excluding_stagger_bits);

        // Start reconstructing x_neg
        uint256_t negative_constant_wnaf_offset(0);

        // Construct 0xF..F
        for (size_t i = 0; i < num_rounds_excluding_stagger_bits; ++i) {
            negative_constant_wnaf_offset += uint256_t(wnaf_window_size * 2 - 1) * (uint256_t(1) << (i * wnaf_size));
        }
        // Shift by stagger
        negative_constant_wnaf_offset = negative_constant_wnaf_offset << stagger;
        // Add for stagger
        if (stagger > 0) {
            negative_constant_wnaf_offset += ((1ULL << wnaf_size) - 1ULL); // FROM STAGGER FRAMGENT
        }

        // TODO: improve efficiency by removing range constraint on lo_offset and hi_offset (we already know are
        // boolean)
        // Add the skew to the bigfield constant
        Fr offset = Fr(nullptr, negative_constant_wnaf_offset).add_to_lower_limb(negative_skew, uint256_t(1));
        // x_pos - x_neg
        Fr reconstructed = wnaf_sum - offset;

        secp256k1_wnaf wnaf_out{ .wnaf = wnaf,
                                 .positive_skew = positive_skew,
                                 .negative_skew = negative_skew,
                                 .least_significant_wnaf_fragment = stagger_fragment,
                                 .has_wnaf_fragment = (stagger > 0) };

        return std::make_pair<Fr, secp256k1_wnaf>((Fr)reconstructed, (secp256k1_wnaf)wnaf_out);
    };

    secp256k1::fr k(scalar.get_value().lo);
    secp256k1::fr klo(0);
    secp256k1::fr khi(0);
    bool klo_negative = false;
    bool khi_negative = false;
    secp256k1::fr::split_into_endomorphism_scalars(k.from_montgomery_form(), klo, khi);

    /* AUDITNOTE: it has been observed in testing that klo_negative is always false.
    On the other hand, khi_negative is sometimes true (e.g., in test_wnaf_secp256k1, take
    scalar_a = 0x3e3e7e9628094ee8942358f6daa1130790f5165d55705d83dad745c85f36807a). So it may be
    that this block is not needed. I could not quickly determine why this might be the case,
    so I leave it to the auditor to check whether the following if block is needed. */
    if (klo.uint256_t_no_montgomery_conversion().get_msb() > 129) {
        klo_negative = true;
        klo = -klo;
    }
    if (khi.uint256_t_no_montgomery_conversion().get_msb() > 129) {
        khi_negative = true;
        khi = -khi;
    }

    const auto [klo_reconstructed, klo_out] = compute_single_wnaf(klo, lo_stagger, klo_negative, true);
    const auto [khi_reconstructed, khi_out] = compute_single_wnaf(khi, hi_stagger, khi_negative, false);

    uint256_t minus_lambda_val(-secp256k1::fr::cube_root_of_unity());
    Fr minus_lambda(
        barretenberg::fr(minus_lambda_val.slice(0, 136)), barretenberg::fr(minus_lambda_val.slice(136, 256)), false);

    Fr reconstructed_scalar = khi_reconstructed.madd(minus_lambda, { klo_reconstructed });

    if (reconstructed_scalar.get_value() != scalar.get_value()) {
        std::cerr << "biggroup_nafs: secp256k1 reconstructed wnaf does not match input! " << reconstructed_scalar
                  << " vs " << scalar << std::endl;
    }
    scalar.binary_basis_limbs[0].element.assert_equal(reconstructed_scalar.binary_basis_limbs[0].element);
    scalar.binary_basis_limbs[1].element.assert_equal(reconstructed_scalar.binary_basis_limbs[1].element);
    scalar.binary_basis_limbs[2].element.assert_equal(reconstructed_scalar.binary_basis_limbs[2].element);
    scalar.binary_basis_limbs[3].element.assert_equal(reconstructed_scalar.binary_basis_limbs[3].element);
    scalar.prime_basis_limb.assert_equal(reconstructed_scalar.prime_basis_limb);

    return { .klo = klo_out, .khi = khi_out };
}

template <typename C, class Fq, class Fr, class G>
template <size_t max_num_bits, size_t WNAF_SIZE>
std::vector<field_t<C>> element<C, Fq, Fr, G>::compute_wnaf(const Fr& scalar)
{
    C* ctx = scalar.context;
    uint512_t scalar_multiplier_512 = uint512_t(uint256_t(scalar.get_value()) % Fr::modulus);
    uint256_t scalar_multiplier = scalar_multiplier_512.lo;

    constexpr size_t num_bits = (max_num_bits == 0) ? (Fr::modulus.get_msb() + 1) : (max_num_bits);
    constexpr size_t num_rounds = ((num_bits + WNAF_SIZE - 1) / WNAF_SIZE);

    uint64_t wnaf_values[num_rounds] = { 0 };
    bool skew = false;
    barretenberg::wnaf::fixed_wnaf<num_bits, 1, WNAF_SIZE>(&scalar_multiplier.data[0], &wnaf_values[0], skew, 0);

    std::vector<field_t<C>> wnaf_entries;
    for (size_t i = 0; i < num_rounds; ++i) {
        bool predicate = bool((wnaf_values[i] >> 31U) & 1U);
        uint64_t offset_entry;
        if (!predicate) {
            offset_entry = (1ULL << (WNAF_SIZE - 1)) + (wnaf_values[i] & 0xffffff);
        } else {
            offset_entry = (1ULL << (WNAF_SIZE - 1)) - 1 - (wnaf_values[i] & 0xffffff);
        }
        field_t<C> entry(witness_t<C>(ctx, offset_entry));

        entry.create_range_constraint(WNAF_SIZE);
        wnaf_entries.emplace_back(entry);
    }

    // add skew
    wnaf_entries.emplace_back(witness_t<C>(ctx, skew));
    wnaf_entries[wnaf_entries.size() - 1].create_range_constraint(1);

    // TODO: VALIDATE SUM DOES NOT OVERFLOW P

    // validate correctness of wNAF
    if constexpr (!Fr::is_composite) {
        std::vector<Fr> accumulators;
        for (size_t i = 0; i < num_rounds; ++i) {
            Fr entry = wnaf_entries[wnaf_entries.size() - 2 - i];
            entry *= 2;
            // entry -= 15;
            entry *= static_cast<Fr>(uint256_t(1) << (i * WNAF_SIZE));
            accumulators.emplace_back(entry);
        }
        accumulators.emplace_back(wnaf_entries[wnaf_entries.size() - 1] * -1);
        uint256_t negative_offset(0);
        for (size_t i = 0; i < num_rounds; ++i) {
            negative_offset += uint256_t((1ULL << WNAF_SIZE) - 1) * (uint256_t(1) << (i * WNAF_SIZE));
        }
        accumulators.emplace_back(-Fr(negative_offset));
        Fr accumulator_result = Fr::accumulate(accumulators);
        scalar.assert_equal(accumulator_result);
    } else {
        // If Fr is a non-native field element, we can't just accumulate the wnaf entries into a single value,
        // as we could overflow the circuit modulus
        //
        // We add the first 34 wnaf entries into a 'low' 136-bit accumulator (136 = 2 68 bit limbs)
        // We add the remaining wnaf entries into a 'high' accumulator
        // We can then directly construct a Fr element from the accumulators.
        // However we cannot underflow our accumulators, and our wnafs represent negative and positive values
        // The raw value of each wnaf value is contained in the range [0, 15], however these values represent integers
        // [-15, -13, -11, ..., 13, 15]
        //
        // To map from the raw value to the actual value, we must compute `value * 2 - 15`
        // However, we do not subtract off the -15 term when constructing our low and high accumulators. Instead of
        // multiplying by two when accumulating we simply add the accumulated value to itself. This way it automatically
        // updates multiplicative constants without computing new witnesses. This ensures the low accumulator will not
        // underflow
        //
        // Once we hvae reconstructed an Fr element out of our accumulators,
        // we ALSO construct an Fr element from the constant offset terms we left out
        // We then subtract off the constant term and call `Fr::assert_is_in_field` to reduce the value modulo
        // Fr::modulus
        const auto reconstruct_half_wnaf = [](field_t<C>* wnafs, const size_t half_round_length) {
            std::vector<field_t<C>> half_accumulators;
            for (size_t i = 0; i < half_round_length; ++i) {
                field_t<C> entry = wnafs[half_round_length - 1 - i];
                entry *= static_cast<field_t<C>>(uint256_t(1) << (i * 4));
                half_accumulators.emplace_back(entry);
            }
            return field_t<C>::accumulate(half_accumulators);
        };
        const size_t midpoint = num_rounds - (Fr::NUM_LIMB_BITS * 2) / WNAF_SIZE;
        auto hi_accumulators = reconstruct_half_wnaf(&wnaf_entries[0], midpoint);
        auto lo_accumulators = reconstruct_half_wnaf(&wnaf_entries[midpoint], num_rounds - midpoint);
        uint256_t negative_lo(0);
        uint256_t negative_hi(0);
        for (size_t i = 0; i < midpoint; ++i) {
            negative_hi += uint256_t(15) * (uint256_t(1) << (i * 4));
        }
        for (size_t i = 0; i < (num_rounds - midpoint); ++i) {
            negative_lo += uint256_t(15) * (uint256_t(1) << (i * 4));
        }
        ASSERT((num_rounds - midpoint) * 4 == 136);
        // If skew == 1 lo_offset = 0, else = 0xf...f
        field_t<C> lo_offset =
            (-field_t<C>(barretenberg::fr(negative_lo)))
                .madd(wnaf_entries[wnaf_entries.size() - 1], field_t<C>(barretenberg::fr(negative_lo)))
                .normalize();
        Fr offset =
            Fr(lo_offset, field_t<C>(barretenberg::fr(negative_hi)) + wnaf_entries[wnaf_entries.size() - 1], true);
        Fr reconstructed = Fr(lo_accumulators, hi_accumulators, true);
        reconstructed = (reconstructed + reconstructed) - offset;
        reconstructed.assert_is_in_field();
        reconstructed.assert_equal(scalar);
    }
    return wnaf_entries;
}

template <typename C, class Fq, class Fr, class G>
std::vector<bool_t<C>> element<C, Fq, Fr, G>::compute_naf(const Fr& scalar, const size_t max_num_bits)
{
    C* ctx = scalar.context;
    uint512_t scalar_multiplier_512 = uint512_t(uint256_t(scalar.get_value()) % Fr::modulus);
    uint256_t scalar_multiplier = scalar_multiplier_512.lo;

    const size_t num_rounds = (max_num_bits == 0) ? Fr::modulus.get_msb() + 1 : max_num_bits;
    std::vector<bool_t<C>> naf_entries(num_rounds + 1);

    // if boolean is false => do NOT flip y
    // if boolean is true => DO flip y
    // first entry is skew. i.e. do we subtract one from the final result or not
    if (scalar_multiplier.get_bit(0) == false) {
        // add skew
        naf_entries[num_rounds] = bool_t<C>(witness_t(ctx, true));
        scalar_multiplier += uint256_t(1);
    } else {
        naf_entries[num_rounds] = bool_t<C>(witness_t(ctx, false));
    }
    for (size_t i = 0; i < num_rounds - 1; ++i) {
        bool next_entry = scalar_multiplier.get_bit(i + 1);
        // if the next entry is false, we need to flip the sign of the current entry. i.e. make negative
        // This is a VERY hacky workaround to ensure that UltraComposer will apply a basic
        // range constraint per bool, and not a full 1-bit range gate
        if (next_entry == false) {
            bool_t<C> bit(ctx, true);
            bit.context = ctx;
            bit.witness_index = witness_t<C>(ctx, true).witness_index; // flip sign
            bit.witness_bool = true;
            ctx->create_range_constraint(
                bit.witness_index, 1, "biggroup_nafs: compute_naf extracted too many bits in non-next_entry case");
            naf_entries[num_rounds - i - 1] = bit;
        } else {
            bool_t<C> bit(ctx, false);
            bit.witness_index = witness_t<C>(ctx, false).witness_index; // don't flip sign
            bit.witness_bool = false;
            ctx->create_range_constraint(
                bit.witness_index, 1, "biggroup_nafs: compute_naf extracted too many bits in next_entry case");
            naf_entries[num_rounds - i - 1] = bit;
        }
    }
    naf_entries[0] = bool_t<C>(ctx, false); // most significant entry is always true

    // validate correctness of NAF
    if constexpr (!Fr::is_composite) {
        std::vector<Fr> accumulators;
        for (size_t i = 0; i < num_rounds; ++i) {
            // bit = 1 - 2 * naf
            Fr entry(naf_entries[naf_entries.size() - 2 - i]);
            entry *= -2;
            entry += 1;
            entry *= static_cast<Fr>(uint256_t(1) << (i));
            accumulators.emplace_back(entry);
        }
        accumulators.emplace_back(Fr(naf_entries[naf_entries.size() - 1]) * -1); // skew
        Fr accumulator_result = Fr::accumulate(accumulators);
        scalar.assert_equal(accumulator_result);
    } else {
        const auto reconstruct_half_naf = [](bool_t<C>* nafs, const size_t half_round_length) {
            // Q: need constraint to start from zero?
            field_t<C> negative_accumulator(0);
            field_t<C> positive_accumulator(0);
            for (size_t i = 0; i < half_round_length; ++i) {
                negative_accumulator = negative_accumulator + negative_accumulator + field_t<C>(nafs[i]);
                positive_accumulator =
                    positive_accumulator + positive_accumulator + field_t<C>(1) - field_t<C>(nafs[i]);
            }
            return std::make_pair(positive_accumulator, negative_accumulator);
        };
        const size_t midpoint = num_rounds - Fr::NUM_LIMB_BITS * 2;
        auto hi_accumulators = reconstruct_half_naf(&naf_entries[0], midpoint);
        auto lo_accumulators = reconstruct_half_naf(&naf_entries[midpoint], num_rounds - midpoint);

        lo_accumulators.second = lo_accumulators.second + field_t<C>(naf_entries[num_rounds]);

        Fr reconstructed_positive = Fr(lo_accumulators.first, hi_accumulators.first);
        Fr reconstructed_negative = Fr(lo_accumulators.second, hi_accumulators.second);
        Fr accumulator = reconstructed_positive - reconstructed_negative;
        accumulator.assert_equal(scalar);
    }
    return naf_entries;
}
} // namespace stdlib
} // namespace plonk