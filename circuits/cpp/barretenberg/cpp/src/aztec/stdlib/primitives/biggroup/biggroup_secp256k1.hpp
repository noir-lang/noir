#pragma once
/**
 * Special case function for performing secp256k1 ecdsa signature verification group operations
 *
 * TODO: we should try to genericize this, but this method is super fiddly and we need it to be efficient!
 *
 **/
namespace plonk {
namespace stdlib {

template <typename C, class Fq, class Fr, class G>
template <typename, typename>
element<C, Fq, Fr, G> element<C, Fq, Fr, G>::secp256k1_ecdsa_mul(const element& pubkey, const Fr& u1, const Fr& u2)
{
    if constexpr (C::type != waffle::ComposerType::PLOOKUP) {
        C* ctx = pubkey.get_context();
        return batch_mul({ element::one(ctx), pubkey }, { u1, u2 });
    }
    /**
     * Compute `out = u1.[1] + u2.[pubkey]
     *
     * Split scalar `u1` into 129-bit short scalars `u1_lo, u1_hi`, where `u1 = u1_lo * \lambda u1_hi`
     * (\lambda is the cube root of unity modulo the secp256k1 group order)
     *
     * Covert `u1_lo` and `u1_hi` into an 8-bit sliding window NAF. Our base point is the G1 generator.
     * We have a precomputed size-256 plookup table of the generator point, multiplied by all possible wNAF values
     *
     * We also split scalar `u2` using the secp256k1 endomorphism. Convert short scalars into 4-bit sliding window NAFs.
     * We will store the lookup table of all possible base-point wNAF states in a ROM table
     * (it's variable-base scalar multiplication in a SNARK with a lookup table! ho ho ho)
     *
     * The wNAFs `u1_lo_wnaf, u1_hi_wnaf, u2_lo_wnaf, u2_hi_wnaf` are each offset by 1 bit relative to each other.
     * i.e. we right-shift `u2_hi` by 1 bit before computing its wNAF
     *      we right-shift `u1_lo` by 2 bits
     *      we right-shift `u1_hi` by 3 bits
     *      we do not shift `u2_lo`
     *
     * We do this to ensure that we are never adding more than 1 point into our accumulator when performing our
     * double-and-add scalar multiplication. It is more efficient to use the montgomery ladder algorithm,
     * compared against doubling an accumulator and adding points into it.
     *
     * The bits removed by the right-shifts are stored in the wnaf's respective `least_significant_wnaf_fragment` member
     * variable
     */
    const auto [u1_lo_wnaf, u1_hi_wnaf] = compute_secp256k1_endo_wnaf<8, 2, 3>(u1);
    const auto [u2_lo_wnaf, u2_hi_wnaf] = compute_secp256k1_endo_wnaf<4, 0, 1>(u2);

    /**
     * Construct our 4-bit variable-base and 8-bit fixed base lookup tables
     **/
    auto P1 = element::one(pubkey.get_context());
    auto P2 = pubkey;
    const auto P1_table =
        element::eight_bit_fixed_base_table<>(element::eight_bit_fixed_base_table<>::CurveType::SECP256K1, false);
    const auto endoP1_table =
        element::eight_bit_fixed_base_table<>(element::eight_bit_fixed_base_table<>::CurveType::SECP256K1, true);
    const auto [P2_table, endoP2_table] = create_endo_pair_four_bit_table_plookup(P2);

    // Initialize our accumulator
    auto accumulator = P2_table[u2_lo_wnaf.wnaf[0]];

    /**
     * main double-and-add loop
     *
     * Acc = Acc + Acc
     * Acc = Acc + Acc
     * Acc = Acc + u2_hi_wnaf.[endoP2] + Acc
     * Acc = Acc + u2_lo_wnaf.[P2] + Acc
     * Acc = Acc + u1_hi_wnaf.[endoP1] + Acc
     * Acc = Acc + u1_lo_wnaf.[P1] + Acc
     * Acc = Acc + u2_hi_wnaf.[endoP2] + Acc
     * Acc = Acc + u2_lo_wnaf.[P2] + Acc
     *
     * We add u2 points into the accumulator twice per 'round' as we only have a 4-bit lookup table
     * (vs the 8-bit table for u1)
     *
     * At the conclusion of this loop, we will need to add a final contribution from `u2_hi, u1_lo, u1_hi`.
     * This is because we offset our wNAFs to take advantage of the montgomery ladder, but this means we
     * have doubled our accumulator AFTER adding our final wnaf contributions from u2_hi, u1_lo and u1_hi
     **/
    for (size_t i = 0; i < 16; ++i) {
        accumulator = accumulator.dbl();
        accumulator = accumulator.dbl();

        // u2_hi_wnaf.wnaf[2 * i] is a field_t element (as are the other wnafs).
        // See `stdlib/memory/rom_table.hpp` for how indirect array accesses are implemented in UltraPlonk
        const auto& add_1 = endoP2_table[u2_hi_wnaf.wnaf[2 * i]];
        const auto& add_2 = P2_table[u2_lo_wnaf.wnaf[2 * i + 1]];
        accumulator = accumulator.double_montgomery_ladder(add_1, add_2);

        const auto& add_3 = endoP1_table[u1_hi_wnaf.wnaf[i]];
        const auto& add_4 = P1_table[u1_lo_wnaf.wnaf[i]];
        accumulator = accumulator.double_montgomery_ladder(add_3, add_4);

        const auto& add_5 = endoP2_table[u2_hi_wnaf.wnaf[2 * i + 1]];
        const auto& add_6 = P2_table[u2_lo_wnaf.wnaf[2 * i + 2]];
        accumulator = accumulator.double_montgomery_ladder(add_5, add_6);
    }

    /**
     * Add the final contributions from `u2_hi, u1_lo, u1_hi`
     **/
    const auto& add_1 = endoP1_table[u1_hi_wnaf.least_significant_wnaf_fragment];
    const auto& add_2 = endoP2_table[u2_hi_wnaf.least_significant_wnaf_fragment];
    const auto& add_3 = P1_table[u1_lo_wnaf.least_significant_wnaf_fragment];
    accumulator = element::chain_add_end(
        element::chain_add(add_3, element::chain_add(add_2, element::chain_add_start(accumulator, add_1))));

    /**
     * Handle wNAF skew.
     *
     * scalars represented via the non-adjacent form can only be odd. If our scalars are even, we must either
     * add or subtract the relevant base point into the accumulator
     **/
    // TODO REMOVE BOOL CASTS, VALUES HAVE ALREADY BEEN RANGE CONSTRAINED
    const auto conditional_add = [](const element& accumulator,
                                    const element& base_point,
                                    const field_t<C>& positive_skew,
                                    const field_t<C>& negative_skew) {
        const bool_t<C> positive_skew_bool(positive_skew);
        const bool_t<C> negative_skew_bool(negative_skew);
        auto to_add = base_point;
        to_add.y = to_add.y.conditional_negate(negative_skew_bool);
        element result = accumulator + to_add;

        // when computing the wNAF we have already validated that positive_skew and negative_skew cannot both be true
        bool_t<C> skew_combined = positive_skew_bool ^ negative_skew_bool;
        result.x = accumulator.x.conditional_select(result.x, skew_combined);
        result.y = accumulator.y.conditional_select(result.y, skew_combined);
        return result;
    };

    accumulator = conditional_add(accumulator, P1, u1_lo_wnaf.positive_skew, u1_lo_wnaf.negative_skew);
    accumulator = conditional_add(accumulator, endoP1_table[128], u1_hi_wnaf.positive_skew, u1_hi_wnaf.negative_skew);
    accumulator = conditional_add(accumulator, P2, u2_lo_wnaf.positive_skew, u2_lo_wnaf.negative_skew);
    accumulator = conditional_add(accumulator, endoP2_table[8], u2_hi_wnaf.positive_skew, u2_hi_wnaf.negative_skew);

    return accumulator;
}
} // namespace stdlib
} // namespace plonk