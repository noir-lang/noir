#pragma once
/**
 * Special case function for performing BN254 group operations
 *
 * TODO: we should try to genericize this, but this method is super fiddly and we need it to be efficient!
 *
 * We use a special case algorithm to split bn254 scalar multipliers into endomorphism scalars
 *
 **/
namespace plonk {
namespace stdlib {

/**
 * Perform a multi-scalar multiplication over the BN254 curve
 *
 * The inputs are:
 *
 * `big_scalars/big_points` : 254-bit scalar multipliers (hardcoded to be 4 at the moment)
 * `small_scalars/small_points` : 128-bit scalar multipliers
 * `generator_scalar` : a 254-bit scalar multiplier over the bn254 generator point
 *
 **/
template <class C, class Fq, class Fr, class G>
template <typename, typename>
element<C, Fq, Fr, G> element<C, Fq, Fr, G>::bn254_endo_batch_mul_with_generator(
    const std::vector<element>& big_points,
    const std::vector<Fr>& big_scalars,
    const std::vector<element>& small_points,
    const std::vector<Fr>& small_scalars,
    const Fr& generator_scalar,
    const size_t max_num_small_bits)
{
    C* ctx = nullptr;
    for (auto element : big_points) {
        if (element.get_context()) {
            ctx = element.get_context();
            break;
        }
    }
    if constexpr (C::type != waffle::ComposerType::PLOOKUP) {
        // MERGENOTE: these four lines don't have an equivalent in d-b-p
        std::vector<element> modified_big_points = big_points;
        std::vector<Fr> modified_big_scalars = big_scalars;
        modified_big_points.emplace_back(element::one(ctx));
        modified_big_scalars.emplace_back(generator_scalar);
        return bn254_endo_batch_mul(
            modified_big_points, modified_big_scalars, small_points, small_scalars, max_num_small_bits);
    } else {
        constexpr size_t NUM_BIG_POINTS = 4;
        // TODO: handle situation where big points size is not 4 :/

        auto big_table_pair =
            create_endo_pair_quad_lookup_table({ big_points[0], big_points[1], big_points[2], big_points[3] });
        auto& big_table = big_table_pair.first;
        auto& endo_table = big_table_pair.second;
        batch_lookup_table small_table(small_points);
        std::vector<std::vector<bool_t<C>>> big_naf_entries;
        std::vector<std::vector<bool_t<C>>> endo_naf_entries;
        std::vector<std::vector<bool_t<C>>> small_naf_entries;

        const auto split_into_endomorphism_scalars = [ctx](const Fr& scalar) {
            barretenberg::fr k = scalar.get_value();
            barretenberg::fr k1(0);
            barretenberg::fr k2(0);
            barretenberg::fr::split_into_endomorphism_scalars(k.from_montgomery_form(), k1, k2);
            Fr scalar_k1 = Fr::from_witness(ctx, k1.to_montgomery_form());
            Fr scalar_k2 = Fr::from_witness(ctx, k2.to_montgomery_form());
            barretenberg::fr beta = barretenberg::fr::cube_root_of_unity();
            scalar.assert_equal(scalar_k1 - scalar_k2 * beta);
            return std::make_pair<Fr, Fr>((Fr)scalar_k1, (Fr)scalar_k2);
        };

        for (size_t i = 0; i < NUM_BIG_POINTS; ++i) {
            const auto [scalar_k1, scalar_k2] = split_into_endomorphism_scalars(big_scalars[i]);
            big_naf_entries.emplace_back(compute_naf(scalar_k1, max_num_small_bits));
            endo_naf_entries.emplace_back(compute_naf(scalar_k2, max_num_small_bits));
        }

        const auto [generator_k1, generator_k2] = split_into_endomorphism_scalars(generator_scalar);
        const std::vector<field_t<C>> generator_wnaf = compute_wnaf<128, 8>(generator_k1);
        const std::vector<field_t<C>> generator_endo_wnaf = compute_wnaf<128, 8>(generator_k2);
        const auto generator_table =
            element::eight_bit_fixed_base_table<>(element::eight_bit_fixed_base_table<>::CurveType::BN254, false);
        const auto generator_endo_table =
            element::eight_bit_fixed_base_table<>(element::eight_bit_fixed_base_table<>::CurveType::BN254, true);

        for (size_t i = 0; i < small_points.size(); ++i) {
            small_naf_entries.emplace_back(compute_naf(small_scalars[i], max_num_small_bits));
        }

        const size_t num_rounds = max_num_small_bits;

        const auto offset_generators = compute_offset_generators(num_rounds);

        auto init_point = element::chain_add(offset_generators.first, small_table.get_chain_initial_entry());
        init_point = element::chain_add(endo_table[0], init_point);
        init_point = element::chain_add(big_table[0], init_point);

        element accumulator = element::chain_add_end(init_point);

        const auto get_point_to_add = [&](size_t naf_index) {
            std::vector<bool_t<C>> small_nafs;
            std::vector<bool_t<C>> big_nafs;
            std::vector<bool_t<C>> endo_nafs;
            for (size_t i = 0; i < small_points.size(); ++i) {
                small_nafs.emplace_back(small_naf_entries[i][naf_index]);
            }
            for (size_t i = 0; i < NUM_BIG_POINTS; ++i) {
                big_nafs.emplace_back(big_naf_entries[i][naf_index]);
                endo_nafs.emplace_back(endo_naf_entries[i][naf_index]);
            }

            auto to_add = small_table.get_chain_add_accumulator(small_nafs);
            to_add = element::chain_add(big_table.get({ big_nafs[0], big_nafs[1], big_nafs[2], big_nafs[3] }), to_add);
            to_add =
                element::chain_add(endo_table.get({ endo_nafs[0], endo_nafs[1], endo_nafs[2], endo_nafs[3] }), to_add);
            return to_add;
        };

        for (size_t i = 1; i < num_rounds / 2; ++i) {

            auto add_1 = get_point_to_add(i * 2 - 1);
            auto add_2 = get_point_to_add(i * 2);

            // TODO update this to work if num_bits is odd
            if ((i * 2) % 8 == 0) {
                add_1 = element::chain_add(generator_table[generator_wnaf[(i * 2 - 8) / 8]], add_1);
                add_1 = element::chain_add(generator_endo_table[generator_endo_wnaf[(i * 2 - 8) / 8]], add_1);
            }
            if (!add_1.is_element) {
                accumulator = accumulator.double_montgomery_ladder(add_1, add_2);
            } else {
                accumulator = accumulator.double_montgomery_ladder(element(add_1.x3_prev, add_1.y3_prev),
                                                                   element(add_2.x3_prev, add_2.y3_prev));
            }
        }

        if ((num_rounds & 0x01ULL) == 0x00ULL) {
            auto add_1 = get_point_to_add(num_rounds - 1);
            add_1 = element::chain_add(generator_table[generator_wnaf[generator_wnaf.size() - 2]], add_1);
            add_1 = element::chain_add(generator_endo_table[generator_endo_wnaf[generator_wnaf.size() - 2]], add_1);
            if (add_1.is_element) {
                element temp(add_1.x3_prev, add_1.y3_prev);
                accumulator = accumulator.montgomery_ladder(temp);
            } else {
                accumulator = accumulator.montgomery_ladder(add_1);
            }
        }

        for (size_t i = 0; i < small_points.size(); ++i) {
            element skew = accumulator - small_points[i];
            Fq out_x = accumulator.x.conditional_select(skew.x, small_naf_entries[i][num_rounds]);
            Fq out_y = accumulator.y.conditional_select(skew.y, small_naf_entries[i][num_rounds]);
            accumulator = element(out_x, out_y);
        }

        uint256_t beta_val = barretenberg::field<typename Fq::TParams>::cube_root_of_unity();
        Fq beta(barretenberg::fr(beta_val.slice(0, 136)), barretenberg::fr(beta_val.slice(136, 256)), false);

        for (size_t i = 0; i < NUM_BIG_POINTS; ++i) {
            element skew_point = big_points[i];
            skew_point.x *= beta;
            element skew = accumulator + skew_point;
            Fq out_x = accumulator.x.conditional_select(skew.x, endo_naf_entries[i][num_rounds]);
            Fq out_y = accumulator.y.conditional_select(skew.y, endo_naf_entries[i][num_rounds]);
            accumulator = element(out_x, out_y);
        }
        {
            element skew = accumulator - generator_table[128];
            Fq out_x = accumulator.x.conditional_select(skew.x, bool_t<C>(generator_wnaf[generator_wnaf.size() - 1]));
            Fq out_y = accumulator.y.conditional_select(skew.y, bool_t<C>(generator_wnaf[generator_wnaf.size() - 1]));
            accumulator = element(out_x, out_y);
        }
        {
            element skew = accumulator - generator_endo_table[128];
            Fq out_x =
                accumulator.x.conditional_select(skew.x, bool_t<C>(generator_endo_wnaf[generator_wnaf.size() - 1]));
            Fq out_y =
                accumulator.y.conditional_select(skew.y, bool_t<C>(generator_endo_wnaf[generator_wnaf.size() - 1]));
            accumulator = element(out_x, out_y);
        }

        for (size_t i = 0; i < NUM_BIG_POINTS; ++i) {
            element skew = accumulator - big_points[i];
            Fq out_x = accumulator.x.conditional_select(skew.x, big_naf_entries[i][num_rounds]);
            Fq out_y = accumulator.y.conditional_select(skew.y, big_naf_entries[i][num_rounds]);
            accumulator = element(out_x, out_y);
        }
        accumulator = accumulator - offset_generators.second;

        return accumulator;
    }
}

/**
 * A batch multiplication method for the BN254 curve. This method is only available if Fr == field_t<barretenberg::fr>
 *
 * big_points : group elements we will multiply by full 254-bit scalar multipliers
 * big_scalars : 254-bit scalar multipliers. We want to compute (\sum big_scalars[i] * big_points[i])
 * small_points : group elements we will multiply by short scalar mutipliers whose max value will be (1 <<
 *max_num_small_bits) small_scalars : short scalar mutipliers whose max value will be (1 << max_num_small_bits)
 * max_num_small_bits : MINIMUM value must be 128 bits
 * (we will be splitting `big_scalars` into two 128-bit scalars, we assume all scalars after this transformation are 128
 *bits)
 **/
template <typename C, class Fq, class Fr, class G>
template <typename, typename>
element<C, Fq, Fr, G> element<C, Fq, Fr, G>::bn254_endo_batch_mul(const std::vector<element>& big_points,
                                                                  const std::vector<Fr>& big_scalars,
                                                                  const std::vector<element>& small_points,
                                                                  const std::vector<Fr>& small_scalars,
                                                                  const size_t max_num_small_bits)
{
    ASSERT(max_num_small_bits >= 128);
    const size_t num_big_points = big_points.size();
    const size_t num_small_points = small_points.size();
    C* ctx = nullptr;
    for (auto element : big_points) {
        if (element.get_context()) {
            ctx = element.get_context();
            break;
        }
    }

    std::vector<element> points;
    std::vector<Fr> scalars;
    std::vector<element> endo_points;
    std::vector<Fr> endo_scalars;

    /**
     * Split big scalars into short 128-bit scalars.
     *
     * For `big_scalars` we use the BN254 curve endomorphism to split the scalar into two short 128-bit scalars.
     * i.e. for scalar multiplier `k` we derive 128-bit values `k1, k2` where:
     *   k = k1 - k2 * \lambda
     * (\lambda is the cube root of unity modulo the group order of the BN254 curve)
     *
     * This ensures ALL our scalar multipliers can now be treated as 128-bit scalars,
     * which halves the number of iterations of our main "double and add" loop!
     */
    barretenberg::fr lambda = barretenberg::fr::cube_root_of_unity();
    barretenberg::fq beta = barretenberg::fq::cube_root_of_unity();
    for (size_t i = 0; i < num_big_points; ++i) {
        Fr scalar = big_scalars[i];
        // Q: is it a problem if wraps? get_value is 512 bits
        // A: it can't wrap, this method only compiles if the Fr type is a field_t<barretenberg::fr> type

        // Split k into short scalars (scalar_k1, scalar_k2) using bn254 endomorphism.
        barretenberg::fr k = uint256_t(scalar.get_value());
        barretenberg::fr k1(0);
        barretenberg::fr k2(0);
        barretenberg::fr::split_into_endomorphism_scalars(k.from_montgomery_form(), k1, k2);
        Fr scalar_k1 = Fr::from_witness(ctx, k1.to_montgomery_form());
        Fr scalar_k2 = Fr::from_witness(ctx, k2.to_montgomery_form());

        // Add copy constraint that validates k1 = scalar_k1 - scalar_k2 * \lambda
        scalar.assert_equal(scalar_k1 - scalar_k2 * lambda);
        scalars.push_back(scalar_k1);
        endo_scalars.push_back(scalar_k2);
        element point = big_points[i];
        points.push_back(point);

        // negate the point that maps to the endo scalar `scalar_k2`
        // instead of computing scalar_k1 * [P] - scalar_k2 * [P], we compute scalar_k1 * [P] + scalar_k2 * [-P]
        point.y = -point.y;
        point.x = point.x * Fq(ctx, uint256_t(beta));
        point.y.self_reduce();
        endo_points.push_back(point);
    }
    for (size_t i = 0; i < num_small_points; ++i) {
        points.push_back(small_points[i]);
        scalars.push_back(small_scalars[i]);
    }
    std::copy(endo_points.begin(), endo_points.end(), std::back_inserter(points));
    std::copy(endo_scalars.begin(), endo_scalars.end(), std::back_inserter(scalars));

    ASSERT(big_scalars.size() == num_big_points);
    ASSERT(small_scalars.size() == num_small_points);

    /**
     * Compute batch_lookup_table
     *
     * batch_lookup_table implements a lookup table for a vector of points.
     *
     * For TurboPlonk, we subdivide `batch_lookup_table` into a set of 3-bit lookup tables,
     * (using 2-bit and 1-bit tables if points.size() is not a multiple of 8)
     *
     * We index the lookup table using a vector of NAF values for each point
     *
     * e.g. for points P_1, .., P_N and naf values s_1, ..., s_n (where S_i = +1 or -1),
     * the lookup table will compute:
     *
     *  \sum_{i=0}^n (s_i ? -P_i : P_i)
     **/
    batch_lookup_table point_table(points);

    /**
     * Compute scalar multiplier NAFs
     *
     * A Non Adjacent Form is a representation of an integer where each 'bit' is either +1 OR -1, i.e. each bit entry is
     *non-zero. This is VERY useful for biggroup operations, as this removes the need to conditionally add points
     *depending on whether the scalar mul bit is +1 or 0 (instead we multiply the y-coordinate by the NAF value, which
     *is cheaper)
     *
     * The vector `naf_entries` tracks the `naf` set for each point, where each `naf` set is a vector of bools
     * if `naf[i][j] = 0` this represents a NAF value of -1
     * if `naf[i][j] = 1` this represents a NAF value of +1
     **/
    const size_t num_rounds = max_num_small_bits;
    const size_t num_points = points.size();
    std::vector<std::vector<bool_t<C>>> naf_entries;
    for (size_t i = 0; i < num_points; ++i) {
        naf_entries.emplace_back(compute_naf(scalars[i], max_num_small_bits));
    }

    /**
     * Initialize accumulator point with an offset generator. See `compute_offset_generators` for detailed explanation
     **/
    const auto offset_generators = compute_offset_generators(num_rounds);

    /**
     * Get the initial entry of our point table. This is the same as point_table.get_accumulator for the most
     *significant NAF entry. HOWEVER, we know the most significant NAF value is +1 because our scalar muls are positive.
     * `get_initial_entry` handles this special case as it's cheaper than `point_table.get_accumulator`
     **/
    element accumulator = offset_generators.first + point_table.get_initial_entry();

    /**
     * Main "double and add" loop
     *
     * Each loop iteration traverses TWO bits of our scalar multiplier. Algorithm performs following:
     *
     * 1. Extract NAF value for bit `2*i - 1` for each scalar multiplier and store in `nafs` vector.
     * 2. Use `nafs` vector to derive the point that we need (`add_1`) to add into our accumulator.
     * 3. Repeat the above 2 steps but for bit `2 * i` (`add_2`)
     * 4. Compute `accumulator = 4 * accumulator + 2 * add_1 + add_2` using `double_montgomery_ladder` method
     *
     * The purpose of the above is to minimize the number of required range checks (vs a simple double and add algo).
     *
     * When computing two iterations of the montgomery ladder algorithm, we can neglect computing the y-coordinate of
     *the 1st ladder output. See `double_montgomery_ladder` for more details.
     **/
    for (size_t i = 1; i < num_rounds / 2; ++i) {
        // `nafs` tracks the naf value for each point for the current round
        std::vector<bool_t<C>> nafs;
        for (size_t j = 0; j < points.size(); ++j) {
            nafs.emplace_back(naf_entries[j][i * 2 - 1]);
        }

        /**
         * Get `chain_add_accumulator`.
         *
         * Recovering a point from our point table requires group additions iff the table is >3 bits.
         * We can chain repeated add operations together without computing the y-coordinate of intermediate addition
         *outputs.
         *
         * This is represented using the `chain_add_accumulator` type. See the type declaration for more details
         *
         * (this is cheaper than regular additions iff point_table.get_accumulator require 2 or more point additions.
         *  Cost is the same as `point_table.get_accumulator` if 1 or 0 point additions are required)
         **/
        element::chain_add_accumulator add_1 = point_table.get_chain_add_accumulator(nafs);
        for (size_t j = 0; j < points.size(); ++j) {
            nafs[j] = (naf_entries[j][i * 2]);
        }
        element::chain_add_accumulator add_2 = point_table.get_chain_add_accumulator(nafs);

        // Perform the double montgomery ladder. We need to convert our chain_add_accumulator types into regular
        // elements if the accumuator does not contain a y-coordinate
        if (!add_1.is_element) {
            accumulator = accumulator.double_montgomery_ladder(add_1, add_2);
        } else {
            accumulator = accumulator.double_montgomery_ladder(element(add_1.x3_prev, add_1.y3_prev),
                                                               element(add_2.x3_prev, add_2.y3_prev));
        }
    }

    // we need to iterate 1 more time if the number of rounds is even
    if ((num_rounds & 0x01ULL) == 0x00ULL) {
        std::vector<bool_t<C>> nafs;
        for (size_t j = 0; j < points.size(); ++j) {
            nafs.emplace_back(naf_entries[j][num_rounds - 1]);
        }
        element::chain_add_accumulator add_1 = point_table.get_chain_add_accumulator(nafs);
        if (add_1.is_element) {
            element temp(add_1.x3_prev, add_1.y3_prev);
            accumulator = accumulator.montgomery_ladder(temp);
        } else {
            accumulator = accumulator.montgomery_ladder(add_1);
        }
    }

    /**
     * Handle skew factors.
     *
     * We represent scalar multipliers via Non Adjacent Form values (NAF).
     * In a NAF, each bit value is either -1 or +1.
     * We use this representation to avoid having to conditionally add points
     * (i.e. every bit we iterate over will result in either a point addition or subtraction,
     *  instead of conditionally adding a point into an accumulator,
     *  we conditionally negate the point's y-coordinate and *always* add it into the accumulator)
     *
     * However! The problem here is that we can only represent odd integers with a NAF.
     * For even integers we add +1 to the integer and set that multiplier's `skew` value to `true`.
     *
     * We record a scalar multiplier's skew value at the end of their NAF values
     *(`naf_entries[point_index][num_rounds]`)
     *
     * If the skew is true, we must subtract the original point from the accumulator.
     **/
    for (size_t i = 0; i < num_points; ++i) {
        element skew = accumulator - points[i];
        Fq out_x = accumulator.x.conditional_select(skew.x, naf_entries[i][num_rounds]);
        Fq out_y = accumulator.y.conditional_select(skew.y, naf_entries[i][num_rounds]);
        accumulator = element(out_x, out_y);
    }

    // Remove the offset generator point!
    accumulator = accumulator - offset_generators.second;

    // Return our scalar mul output
    return accumulator;
}
} // namespace stdlib
} // namespace plonk