#include "keccak.hpp"
#include "barretenberg/common/constexpr_utils.hpp"
#include "barretenberg/numeric/bitop/sparse_form.hpp"
#include "barretenberg/stdlib/primitives/logic/logic.hpp"
#include "barretenberg/stdlib/primitives/uint/uint.hpp"
namespace bb::stdlib {

using namespace plookup;

/**
 * @brief Normalize a base-11 limb and left-rotate by keccak::ROTATIONS[lane_index] bits.
 *        This method also extracts the most significant bit of the normalised rotated limb.
 *        Used in the RHO and IOTA rounds and in `sponge_absorb`.
 *
 * Normalize process:
 *  Input v = \sum_{i=0}^63 b_i * 11^i , where b is in range [0, 1, 2]
 *  Output  = \sum_{i=0}^63 (b_i & 1) * 11^i (i.e. even values go to 0)
 *
 * Implementation is via a sequence of lookup tables
 *
 * @tparam lane_index What keccak lane are we working on?
 * @param limb Input limb we want to normalize and rotate
 * @param msb (return parameter) The most significant bit of the normalized and rotated limb
 * @return field_t<Builder> The normalized and rotated output
 */
template <typename Builder>
template <size_t lane_index>
field_t<Builder> keccak<Builder>::normalize_and_rotate(const field_ct& limb, field_ct& msb)
{
    // left_bits = the number of bits that wrap around 11^64 (left_bits)
    constexpr size_t left_bits = ROTATIONS[lane_index];

    // right_bits = the number of bits that don't wrap
    constexpr size_t right_bits = 64 - ROTATIONS[lane_index];

    // TODO read from same source as plookup table code
    constexpr size_t max_bits_per_table = plookup::keccak_tables::Rho<>::MAXIMUM_MULTITABLE_BITS;

    // compute the number of lookups required for our left and right bit slices
    constexpr size_t num_left_tables = left_bits / max_bits_per_table + (left_bits % max_bits_per_table > 0 ? 1 : 0);
    constexpr size_t num_right_tables = right_bits / max_bits_per_table + (right_bits % max_bits_per_table > 0 ? 1 : 0);

    // get the numerical value of the left and right bit slices
    // (lookup table input values derived from left / right)
    uint256_t input = limb.get_value();
    constexpr uint256_t slice_divisor = BASE.pow(right_bits);
    const auto [left, right] = input.divmod(slice_divisor);

    // compute the normalized values for the left and right bit slices
    // (lookup table output values derived from left_normalised / right_normalized)
    uint256_t left_normalized = normalize_sparse(left);
    uint256_t right_normalized = normalize_sparse(right);

    /**
     * manually construct the ReadData object required to generate plookup gate constraints.
     * To explain in more detail: the input integer can be represented via two the bit slices [A, B]
     * (A = left, B = right)
     *
     * For example, imagine our input is a 32-bit integer A represented as: A = A3.11^24 + A2.11^16 + A1.11^8 + A0,
     *              and our output is a 32-bit integer B = B3.11^24 + B2.11^16 + B1.11^8 + B0
     *
     * In this example, we want to normalize A and left-rotate by 16 bits.
     *
     * Our lookup gate wire values will look like the following:
     *
     * | Row | C0                                       | C1           | C2       |
     * | --- | -----------------------------------------| ------------ | -------- |
     * |  0  | A3.11^24 + A2.11^16 + A1.11^8  + A0      | B1.11^8 + B0 | A0.msb() |
     * |  1  |            A3.11^16 + A2.11^8  + A1      |           B1 | A1.msb() |
     * |  2  |                       A1311^8  + A2      | B3.11^8 + B2 | A2.msb() |
     * |  3  |                                  A3      |           B3 | A3.msb() |
     *
     * The plookup table keys + values are derived via the expression:
     *
     * C1[i] + C1[i+1].q1[i] = LOOKUP[C0[i] + C0[i+1].q0[i]]
     *
     * (the same applies for C2, however q2[i] = 0 for all rows)
     *
     * The plookup coefficients for the rows treat Column0 as a single accumulating sum,
     * but Column1 is a pair of accumulating sums.
     * In the above example, the q coefficient value are:
     *
     * | Row | Q1   | Q2   | Q3 |
     * | --- | ---- | ---- | -- |
     * |  0  | 11^8 | 11^8 | 0  |
     * |  1  | 11^8 | 0    | 0  |
     * |  2  | 11^8 | 11^8 | 0  |
     * |  3  | 0    | 0    | 0  |
     *
     * stdlib::plookup cannot derive witnesses in the above pattern without a substantial rewrite,
     * so we do it manually in this method!
     **/
    plookup::ReadData<bb::fr> lookup;

    // compute plookup witness values for a given slice
    // (same lambda can be used to compute witnesses for left and right slices)
    auto compute_lookup_witnesses_for_limb = [&]<size_t limb_bits, size_t num_lookups>(uint256_t& normalized) {
        // (use a constexpr loop to make some pow and div operations compile-time)
        bb::constexpr_for<0, num_lookups, 1>([&]<size_t i> {
            constexpr size_t num_bits_processed = i * max_bits_per_table;

            // How many bits can this slice contain?
            // We want to implicitly range-constrain `normalized < 11^{limb_bits}`,
            // which means potentially using a lookup table that is not of size 11^{max_bits_per_table}
            // for the most-significant slice
            constexpr size_t bit_slice = (num_bits_processed + max_bits_per_table > limb_bits)
                                             ? limb_bits % max_bits_per_table
                                             : max_bits_per_table;

            // current column values are tracked via 'input' and 'normalized'
            lookup[ColumnIdx::C1].push_back(input);
            lookup[ColumnIdx::C2].push_back(normalized);

            constexpr uint64_t divisor = numeric::pow64(static_cast<uint64_t>(BASE), bit_slice);
            constexpr uint64_t msb_divisor = divisor / static_cast<uint64_t>(BASE);

            // compute the value of the most significant bit of this slice and store in C3
            const auto [normalized_quotient, normalized_slice] = normalized.divmod(divisor);

            // 256-bit divisions are expensive! cast to u64s when we don't need the extra bits
            const uint64_t normalized_msb = (static_cast<uint64_t>(normalized_slice) / msb_divisor);
            lookup[ColumnIdx::C3].push_back(normalized_msb);

            // We need to provide a key/value object for this lookup in order for the Builder
            // to compute the plookup sorted list commitment
            const auto [input_quotient, input_slice] = input.divmod(divisor);
            lookup.key_entries.push_back(
                { { static_cast<uint64_t>(input_slice), 0 }, { normalized_slice, normalized_msb } });

            // reduce the input and output by 11^{bit_slice}
            input = input_quotient;
            normalized = normalized_quotient;
        });
    };

    // template lambda syntax is a little funky.
    // Need to explicitly write `.template operator()` (instead of just `()`).
    // Otherwise compiler cannot distinguish between `>` symbol referring to closing the template parameter list,
    // OR `>` being a greater-than operator :/
    compute_lookup_witnesses_for_limb.template operator()<right_bits, num_right_tables>(right_normalized);
    compute_lookup_witnesses_for_limb.template operator()<left_bits, num_left_tables>(left_normalized);

    // Call builder method to create plookup constraints.
    // The MultiTable table index can be derived from `lane_idx`
    // Each lane_idx has a different rotation amount, which changes sizes of left/right slices
    // and therefore the selector constants required (i.e. the Q1, Q2, Q3 values in the earlier example)
    const auto accumulator_witnesses = limb.context->create_gates_from_plookup_accumulators(
        (plookup::MultiTableId)((size_t)KECCAK_NORMALIZE_AND_ROTATE + lane_index),
        lookup,
        limb.normalize().get_witness_index());

    // extract the most significant bit of the normalized output from the final lookup entry in column C3
    msb = field_ct::from_witness_index(limb.get_context(),
                                       accumulator_witnesses[ColumnIdx::C3][num_left_tables + num_right_tables - 1]);

    // Extract the witness that maps to the normalized right slice
    const field_t<Builder> right_output =
        field_t<Builder>::from_witness_index(limb.get_context(), accumulator_witnesses[ColumnIdx::C2][0]);

    if (num_left_tables == 0) {
        // if the left slice size is 0 bits (i.e. no rotation), return `right_output`
        return right_output;
    } else {
        // Extract the normalized left slice
        const field_t<Builder> left_output = field_t<Builder>::from_witness_index(
            limb.get_context(), accumulator_witnesses[ColumnIdx::C2][num_right_tables]);

        // Stitch the right/left slices together to create our rotated output
        constexpr uint256_t shift = BASE.pow(ROTATIONS[lane_index]);
        return (left_output + right_output * shift);
    }
}

/**
 * @brief Compute twisted representation of hash lane
 *
 * The THETA round requires computation of XOR(A, ROTL(B, 1))
 *
 * We do this via a 'twisted' base-11 representation.
 *
 * If the bit slices for a regular variable are arranged [b63, ..., b0],
 * the twisted representation is a 65-bit variable [b63, ..., b0, b63]
 *
 * The equivalent of XOR(A, ROTL(B, 1)) is A.twist + 2B.twist (in base-11 form)
 * The output is present in bit slices 1-64
 *
 * @tparam Builder
 * @param internal
 */
template <typename Builder> void keccak<Builder>::compute_twisted_state(keccak_state& internal)
{
    for (size_t i = 0; i < NUM_KECCAK_LANES; ++i) {
        internal.twisted_state[i] = ((internal.state[i] * 11) + internal.state_msb[i]).normalize();
    }
}

/**
 * @brief THETA round
 *
 * @tparam Builder
 *
 * THETA consists of XOR operations as well as left rotations by 1 bit.
 *
 * We represent 64-bit integers in a base-11 representation where
 *  limb = \sum_{i=0}^63 b_i * 11^i
 *
 * At the start of THETA, all b_i values are either 0 or 1
 *
 * We can efficiently evaluate XOR operations via simple additions!
 * If b_i = even, this represents a bit value of 0
 * If b_i = odd, this represents a bit value of 1
 *
 * The KECCAK_THETA_OUTPUT lookup table is used to 'normalize' base-11 integers,
 * i.e. convert b_i values from [0, ..., 10] to [0, 1] where even == 0, odd == 1
 *
 * The choice of base for our representation effects the following:
 * 1. the number of normalization lookups required to avoid overflowing the base
 * 2. the cost of normalization lookups
 *
 * Bigger base reduces (1) but increases (2). For THETA, base-11 is optimal (I think...)
 *
 * ### HANDLING ROTATIONS
 *
 * We need to left-rotate the C[5] array by 1-bit to compute D[5]. Naive way is expensive so we cheat!
 * When converting integers into base-11 representation, we use a lookup table column to give us the
 * most significant bit of the integer.
 *
 * This enables us to create a 'twisted' representation of the integer in base-11:
 *
 * twisted_limb = (b_63) + \sum_{i=0}^63 b_i * 11^{i + 1}
 *
 * e.g. if limb's bit ordering is [0,   b63, ..., b1, b0 ]
 * twisted limb bit ordering is   [b63, b62, ..., b0, b63]
 *
 * We want to be able to compute XOR(A, B.rotate_left(1)) and can do this via twisted representations
 *
 * The equivalent in base-11 world is twisted_A * 2 + twisted_B.
 * The output of the XOR operation exists in bit-slices 1, ..., 63
 * (which can be extracted by removing the least and most significant slices of the output)
 * This is MUCH cheaper than the extra range constraints required for a naive left-rotation
 *
 * Total cost of theta = 20.5 gates per 5 lanes + 25 = 127.5 per round
 */
template <typename Builder> void keccak<Builder>::theta(keccak_state& internal)
{
    std::array<field_ct, 5> C;
    std::array<field_ct, 5> D;

    auto& state = internal.state;
    const auto& twisted_state = internal.twisted_state;
    for (size_t i = 0; i < 5; ++i) {

        /**
         * field_ct::accumulate can compute 5 addition operations in only 2 gates:
         * Gate 0 wires [a0, a1, a2, a3]
         * Gate 1 wires [b0, b1, b2, b3]
         * b3 = a0 + a1 + a2 + a3
         * b2 = b3 + b0 + b1
         * (b2 is the output wire)
         **/
        C[i] = field_ct::accumulate({ twisted_state[i],
                                      twisted_state[5 + i],
                                      twisted_state[10 + i],
                                      twisted_state[15 + i],
                                      twisted_state[20 + i] });
    }

    /**
     * Compute D by exploiting twisted representation
     * to get a cheap left-rotation by 1 bit
     */
    for (size_t i = 0; i < 5; ++i) {
        const auto non_shifted_equivalent = (C[(i + 4) % 5]);
        const auto shifted_equivalent = C[(i + 1) % 5] * BASE;
        D[i] = (non_shifted_equivalent + shifted_equivalent);
    }

    /**
     * D contains 66 base-11 slices.
     *
     * We need to remove the 2 most significant slices as they
     * are artifacts of our twist operation.
     *
     * We also need to 'normalize' D (i.e. convert each base value to be 0 or 1),
     * to prevent our base from overflowing when we XOR D into internal.state
     *
     * 1. create sliced_D witness, plus lo and hi slices
     * 2. validate D == lo + (sliced_D * 11) + (hi * 11^65)
     * 3. feed sliced_D into KECCAK_THETA_OUTPUT lookup table
     *
     * KECCAK_THETA_OUTPUT currently splices its input into 16 4-bit slices (in base 11 i.e. from 0 to 11^4 - 1)
     * This ensures that sliced_D is correctly range constrained to be < 11^64
     */
    static constexpr uint256_t divisor = BASE.pow(64);
    static constexpr uint256_t multiplicand = BASE.pow(65);
    for (size_t i = 0; i < 5; ++i) {
        uint256_t D_native = D[i].get_value();
        const auto [D_quotient, lo_native] = D_native.divmod(BASE);
        const uint256_t hi_native = D_quotient / divisor;
        const uint256_t mid_native = D_quotient - hi_native * divisor;

        field_ct hi(witness_ct(internal.context, hi_native));
        field_ct mid(witness_ct(internal.context, mid_native));
        field_ct lo(witness_ct(internal.context, lo_native));

        // assert equal should cost 1 gate (multipliers are all constants)
        D[i].assert_equal((hi * multiplicand).add_two(mid * 11, lo));
        internal.context->create_new_range_constraint(hi.get_witness_index(), static_cast<uint64_t>(BASE));
        internal.context->create_new_range_constraint(lo.get_witness_index(), static_cast<uint64_t>(BASE));

        // If number of bits in KECCAK_THETA_OUTPUT table does NOT cleanly divide 64,
        // we need an additional range constraint to ensure that mid < 11^64
        if constexpr (64 % plookup::keccak_tables::Theta::TABLE_BITS == 0) {
            // N.B. we could optimize out 5 gates per round here but it's very fiddly...
            // In previous section, D[i] = X + Y (non shifted equiv and shifted equiv)
            // We also want to validate D[i] == hi' + mid' + lo (where hi', mid' are hi, mid scaled by constants)
            // We *could* create a big addition gate to validate the previous logic w. following structure:
            // | w1 | w2  | w3 | w4 |
            // | -- | --- | -- | -- |
            // | hi | mid | lo | X  |
            // | P0 | P1  | P2 | Y  |
            // To save a gate, we would need to place the wires for the first KECCAK_THETA_OUTPUT plookup gate
            // at P0, P1, P2. This is fiddly builder logic that is circuit-width-dependent
            // (this would save 120 gates per hash block... not worth making the code less readable for that)
            D[i] = plookup_read<Builder>::read_from_1_to_2_table(KECCAK_THETA_OUTPUT, mid);
        } else {
            const auto accumulators = plookup_read<Builder>::get_lookup_accumulators(KECCAK_THETA_OUTPUT, D[i]);
            D[i] = accumulators[ColumnIdx::C2][0];

            // Ensure input to lookup is < 11^64,
            // by validating most significant input slice is < 11^{64 mod slice_bits}
            const field_ct most_significant_slice = accumulators[ColumnIdx::C1][accumulators[ColumnIdx::C1].size() - 1];

            // N.B. cheaper to validate (11^{64 mod slice_bits} - slice < 2^14) as this
            // prevents an extra range table from being created
            constexpr uint256_t maximum = BASE.pow(64 % plookup::keccak_tables::Theta::TABLE_BITS);
            const field_ct target = -most_significant_slice + maximum;
            ASSERT(((uint256_t(1) << Builder::DEFAULT_PLOOKUP_RANGE_BITNUM) - 1) > maximum);
            target.create_range_constraint(Builder::DEFAULT_PLOOKUP_RANGE_BITNUM,
                                           "input to KECCAK_THETA_OUTPUT too large!");
        }
    }

    // compute state[j * 5 + i] XOR D[i] in base-11 representation
    for (size_t i = 0; i < 5; ++i) {
        for (size_t j = 0; j < 5; ++j) {
            state[j * 5 + i] = state[j * 5 + i] + D[i];
        }
    }
}

/**
 * @brief RHO round
 *
 * @tparam Builder
 *
 * The limbs of internal.state are represented via base-11 integers
 *  limb = \sum_{i=0}^63 b_i * 11^i
 * The value of each b_i can be in the range [0, 1, 2] due to the THETA round XOR operations
 *
 * We need to do the following:
 *
 * 1. 'normalize' each limb so that each b_i value is 0 or 1
 * 2. left-rotate each limb as defined by the keccak `rotations` matrix
 *
 * The KECCAK_RHO_OUTPUT lookup table is used for both. See `normalize_and_rotate` for more details
 *
 * COST PER LIMB...
 *     8 gates for first lane (no rotation. Lookup table is 8-bits per slice = 8 lookups for 64 bits)
 *     10 gates for other 24 lanes (lookup sequence is split into 6 8-bit slices and 2 slices that sum to 8 bits,
 *     an addition gate is required to complete the rotation)
 *
 * Total costs is 248 gates.
 *
 * N.B. Can reduce lookup costs by using larger lookup tables.
 * Current algo is optimized for lookup tables where sum of all table sizes is < 2^64
 */
template <typename Builder> void keccak<Builder>::rho(keccak_state& internal)
{
    constexpr_for<0, NUM_KECCAK_LANES, 1>(
        [&]<size_t i>() { internal.state[i] = normalize_and_rotate<i>(internal.state[i], internal.state_msb[i]); });
}

/**
 * @brief PI
 *
 * PI permutes the keccak lanes. Adds 0 constraints as this is simply a
 * re-ordering of witnesses
 *
 * @tparam Builder
 * @param internal
 */
template <typename Builder> void keccak<Builder>::pi(keccak_state& internal)
{
    std::array<field_ct, NUM_KECCAK_LANES> B;

    for (size_t j = 0; j < 5; ++j) {
        for (size_t i = 0; i < 5; ++i) {
            B[j * 5 + i] = internal.state[j * 5 + i];
        }
    }

    for (size_t y = 0; y < 5; ++y) {
        for (size_t x = 0; x < 5; ++x) {
            size_t u = (0 * x + 1 * y) % 5;
            size_t v = (2 * x + 3 * y) % 5;

            internal.state[v * 5 + u] = B[5 * y + x];
        }
    }
}

/**
 * @brief CHI
 *
 * The CHI round applies the following logic to the hash lanes:
 *     A XOR (~B AND C)
 *
 * In base-11 representation we can create an equivalent linear operation:
 *     1 + 2A - B + C
 *
 * Output values will range from [0, 1, 2, 3, 4] and are mapped back into [0, 1]
 * via the KECCAK_CHI_OUTPUT lookup table
 *
 * N.B. the KECCAK_CHI_OUTPUT table also has a column for the most significant bit of each lookup.
 *      We use this to create a 'twisted representation of each hash lane (see THETA comments for more details)
 * @tparam Builder
 */
template <typename Builder> void keccak<Builder>::chi(keccak_state& internal)
{
    // (cost = 12 * 25 = 300?)
    auto& state = internal.state;

    for (size_t y = 0; y < 5; ++y) {
        std::array<field_ct, 5> lane_outputs;
        for (size_t x = 0; x < 5; ++x) {
            const auto A = state[y * 5 + x];
            const auto B = state[y * 5 + ((x + 1) % 5)];
            const auto C = state[y * 5 + ((x + 2) % 5)];

            // vv should cost 1 gate
            lane_outputs[x] = (A + A + CHI_OFFSET).add_two(-B, C);
        }
        for (size_t x = 0; x < 5; ++x) {
            // Normalize lane outputs and assign to internal.state
            auto accumulators = plookup_read<Builder>::get_lookup_accumulators(KECCAK_CHI_OUTPUT, lane_outputs[x]);
            internal.state[y * 5 + x] = accumulators[ColumnIdx::C2][0];
            internal.state_msb[y * 5 + x] = accumulators[ColumnIdx::C3][accumulators[ColumnIdx::C3].size() - 1];
        }
    }
}

/**
 * @brief IOTA
 *
 * XOR first hash limb with a precomputed constant.
 * We re-use the RHO_OUTPUT table to normalize after this operation
 * @tparam Builder
 * @param internal
 * @param round
 */
template <typename Builder> void keccak<Builder>::iota(keccak_state& internal, size_t round)
{
    const field_ct xor_result = internal.state[0] + SPARSE_RC[round];

    // normalize lane value so that we don't overflow our base11 modulus boundary in the next round
    internal.state[0] = normalize_and_rotate<0>(xor_result, internal.state_msb[0]);

    // No need to add constraints to compute twisted repr if this is the last round
    if (round != NUM_KECCAK_ROUNDS - 1) {
        compute_twisted_state(internal);
    }
}

template <typename Builder> void keccak<Builder>::keccakf1600(keccak_state& internal)
{
    for (size_t i = 0; i < NUM_KECCAK_ROUNDS; ++i) {
        theta(internal);
        rho(internal);
        pi(internal);
        chi(internal);
        iota(internal, i);
    }
}

template <typename Builder>
void keccak<Builder>::sponge_absorb(keccak_state& internal,
                                    const std::vector<field_ct>& input_buffer,
                                    const std::vector<field_ct>& msb_buffer,
                                    const field_ct& num_blocks_with_data)
{
    const size_t l = input_buffer.size();

    const size_t num_blocks = l / (BLOCK_SIZE / 8);

    for (size_t i = 0; i < num_blocks; ++i) {
        // create a copy of our keccak state in case we need to revert this hash block application
        keccak_state previous = internal;
        if (i == 0) {
            for (size_t j = 0; j < LIMBS_PER_BLOCK; ++j) {
                internal.state[j] = input_buffer[j];
                internal.state_msb[j] = msb_buffer[j];
            }
            for (size_t j = LIMBS_PER_BLOCK; j < NUM_KECCAK_LANES; ++j) {
                internal.state[j] = witness_ct::create_constant_witness(internal.context, 0);
                internal.state_msb[j] = witness_ct::create_constant_witness(internal.context, 0);
            }
        } else {
            for (size_t j = 0; j < LIMBS_PER_BLOCK; ++j) {
                internal.state[j] += input_buffer[i * LIMBS_PER_BLOCK + j];
                internal.state[j] = normalize_and_rotate<0>(internal.state[j], internal.state_msb[j]);
            }
        }

        compute_twisted_state(internal);
        keccakf1600(internal);

        // if `i >= num_blocks_with_data` then we want to revert the effects of this block and set `internal_state` to
        // equal `previous`.
        // This can happen for circuits where the input hash size is not known at circuit-compile time (only the maximum
        // hash size).
        // For example, a circuit that hashes up to 544 bytes (but maybe less depending on the witness assignment)
        bool_ct block_predicate = field_ct(i).template ranged_less_than<8>(num_blocks_with_data);

        for (size_t j = 0; j < NUM_KECCAK_LANES; ++j) {
            internal.state[j] = field_ct::conditional_assign(block_predicate, internal.state[j], previous.state[j]);
            internal.state_msb[j] =
                field_ct::conditional_assign(block_predicate, internal.state_msb[j], previous.state_msb[j]);
            internal.twisted_state[j] =
                field_ct::conditional_assign(block_predicate, internal.twisted_state[j], previous.twisted_state[j]);
        }
    }
}

template <typename Builder> byte_array<Builder> keccak<Builder>::sponge_squeeze(keccak_state& internal)
{
    byte_array_ct result(internal.context);

    // Each hash limb represents a little-endian integer. Need to reverse bytes before we write into the output array
    for (size_t i = 0; i < 4; ++i) {
        field_ct output_limb = plookup_read<Builder>::read_from_1_to_2_table(KECCAK_FORMAT_OUTPUT, internal.state[i]);
        byte_array_ct limb_bytes(output_limb, 8);
        byte_array_ct little_endian_limb_bytes(internal.context, 8);
        little_endian_limb_bytes.set_byte(0, limb_bytes[7]);
        little_endian_limb_bytes.set_byte(1, limb_bytes[6]);
        little_endian_limb_bytes.set_byte(2, limb_bytes[5]);
        little_endian_limb_bytes.set_byte(3, limb_bytes[4]);
        little_endian_limb_bytes.set_byte(4, limb_bytes[3]);
        little_endian_limb_bytes.set_byte(5, limb_bytes[2]);
        little_endian_limb_bytes.set_byte(6, limb_bytes[1]);
        little_endian_limb_bytes.set_byte(7, limb_bytes[0]);
        result.write(little_endian_limb_bytes);
    }
    return result;
}

/**
 * @brief Convert the input buffer into 8-bit keccak lanes in little-endian form.
 *        Additionally, insert padding bytes if required,
 *        and add the keccak terminating bytes 0x1/0x80
 *        (0x1 inserted after the final byte of input data)
 *        (0x80 inserted at the end of the final block)
 *
 * @tparam Builder
 * @param input
 * @param num_bytes
 * @return std::vector<field_t<Builder>>
 */
template <typename Builder>
std::vector<field_t<Builder>> keccak<Builder>::format_input_lanes(byte_array_ct& _input, const uint32_ct& num_bytes)
{
    byte_array_ct input(_input);

    // make sure that every byte past `num_bytes` is zero!
    for (size_t i = 0; i < input.size(); ++i) {
        bool_ct valid_byte = uint32_ct(static_cast<uint32_t>(i)) < num_bytes;
        input.set_byte(i, (input[i] * valid_byte));
    }

    auto* ctx = input.get_context();

    // We require that `num_bytes` does not exceed the size of our input byte array.
    // (can be less if the hash size is not known at circuit-compile time, only the maximum)
    ASSERT(input.size() >= static_cast<size_t>(num_bytes.get_value()));
    field_ct(num_bytes > uint32_ct(static_cast<uint32_t>(input.size()))).assert_equal(0);
    const size_t input_size = input.size();
    // max_blocks_length = maximum number of bytes to hash
    const size_t max_blocks = (input_size + BLOCK_SIZE) / BLOCK_SIZE;
    const size_t max_blocks_length = (BLOCK_SIZE * (max_blocks));

    byte_array_ct block_bytes(input);

    const size_t byte_difference = max_blocks_length - input_size;
    byte_array_ct padding_bytes(ctx, byte_difference);
    for (size_t i = 0; i < byte_difference; ++i) {
        padding_bytes.set_byte(i, witness_ct::create_constant_witness(ctx, 0));
    }
    block_bytes.write(padding_bytes);

    uint32_ct num_real_blocks = (num_bytes + BLOCK_SIZE) / BLOCK_SIZE;
    uint32_ct num_real_blocks_bytes = num_real_blocks * BLOCK_SIZE;

    // Keccak requires that 0x1 is appended after the final byte of input data.
    // Similarly, the final byte of the final padded block must be 0x80.
    // If `num_bytes` is constant then we know where to write these values at circuit-compile time
    if (num_bytes.is_constant()) {
        const auto terminating_byte = static_cast<size_t>(num_bytes.get_value());
        const auto terminating_block_byte = static_cast<size_t>(num_real_blocks_bytes.get_value()) - 1;
        block_bytes.set_byte(terminating_byte, witness_ct::create_constant_witness(ctx, 0x1));
        block_bytes.set_byte(terminating_block_byte, witness_ct::create_constant_witness(ctx, 0x80));
    }

    // keccak lanes interpret memory as little-endian integers,
    // means we need to swap our byte ordering...
    for (size_t i = 0; i < block_bytes.size(); i += 8) {
        std::array<field_ct, 8> temp;
        for (size_t j = 0; j < 8; ++j) {
            temp[j] = block_bytes[i + j];
        }
        block_bytes.set_byte(i, temp[7]);
        block_bytes.set_byte(i + 1, temp[6]);
        block_bytes.set_byte(i + 2, temp[5]);
        block_bytes.set_byte(i + 3, temp[4]);
        block_bytes.set_byte(i + 4, temp[3]);
        block_bytes.set_byte(i + 5, temp[2]);
        block_bytes.set_byte(i + 6, temp[1]);
        block_bytes.set_byte(i + 7, temp[0]);
    }
    const size_t byte_size = block_bytes.size();

    const size_t num_limbs = byte_size / WORD_SIZE;
    std::vector<field_ct> sliced_buffer;

    // populate a vector of 64-bit limbs from our byte array
    for (size_t i = 0; i < num_limbs; ++i) {
        field_ct sliced;
        if (i * WORD_SIZE + WORD_SIZE > byte_size) {
            const size_t slice_size = byte_size - (i * WORD_SIZE);
            const size_t byte_shift = (WORD_SIZE - slice_size) * 8;
            sliced = field_ct(block_bytes.slice(i * WORD_SIZE, slice_size));
            sliced = (sliced * (uint256_t(1) << byte_shift)).normalize();
        } else {
            sliced = field_ct(block_bytes.slice(i * WORD_SIZE, WORD_SIZE));
        }
        sliced_buffer.emplace_back(sliced);
    }

    // If the input preimage size is known at circuit-compile time, nothing more to do.
    if (num_bytes.is_constant()) {
        return sliced_buffer;
    }

    // If we do *not* know the preimage size at circuit-compile time, we have several steps we must execute:
    // 1. Validate that `input[num_bytes], input[num_bytes + 1], ..., input[input.size() - 1]` are all ZERO.
    // 2. Insert the keccak input terminating byte `0x1` at `input[num_bytes]`
    // 3. Insert the keccak block terminating byte `0x80` at `input[num_real_block_bytes - 1]`
    // We do these steps after we have converted into 64 bit lanes as we have fewer elements to iterate over (is
    // cheaper)
    std::vector<field_ct> lanes = sliced_buffer;

    // compute the lane index of the terminating input byte
    field_ct num_bytes_as_field(num_bytes);
    field_ct terminating_index = field_ct(uint32_ct((num_bytes) / WORD_SIZE));

    // compute the value we must add to limbs[terminating_index] to insert 0x1 at the correct byte index (accounting for
    // the previous little-endian conversion)
    field_ct terminating_index_bytes_shift = (num_bytes_as_field) - (terminating_index * WORD_SIZE);
    field_ct terminating_index_limb_addition = field_ct(256).pow(terminating_index_bytes_shift);

    // compute the lane index of the terminating block byte
    field_ct terminating_block_index = field_ct((num_real_blocks_bytes - 1) / WORD_SIZE);
    field_ct terminating_block_bytes_shift =
        field_ct(num_real_blocks_bytes - 1) - (terminating_block_index * WORD_SIZE);
    // compute the value we must add to limbs[terminating_index] to insert 0x1 at the correct byte index (accounting for
    // the previous little-endian conversion)
    field_ct terminating_block_limb_addition = field_ct(0x80ULL) * field_ct(256).pow(terminating_block_bytes_shift);

    // validate the number of lanes is less than the default plookup size (we use the default size to do a cheap `<`
    // check later on. Should be fine as this translates to ~2MB of input data)
    ASSERT(uint256_t(sliced_buffer.size()) < (uint256_t(1ULL) << Builder::DEFAULT_PLOOKUP_RANGE_BITNUM));

    // If the terminating input byte index matches the terminating block byte index, we set the byte to 0x80.
    // If we trigger this case, set `terminating_index_limb_addition` to 0 so that we do not write `0x01 + 0x80`
    terminating_index_limb_addition = field_ct::conditional_assign(
        field_ct(num_bytes) == field_ct(num_real_blocks_bytes) - 1, 0, terminating_index_limb_addition);
    field_ct terminating_limb;

    // iterate over our lanes to perform the above listed checks
    for (size_t i = 0; i < sliced_buffer.size(); ++i) {
        // If i > terminating_index, limb must be 0
        bool_ct limb_must_be_zeroes =
            terminating_index.template ranged_less_than<Builder::DEFAULT_PLOOKUP_RANGE_BITNUM>(field_ct(i));
        // Is i == terminating_limb_index?
        bool_ct is_terminating_limb = terminating_index == field_ct(i);

        // Is i == terminating_block_limb?
        bool_ct is_terminating_block_limb = terminating_block_index == field_ct(i);

        (lanes[i] * limb_must_be_zeroes).assert_equal(0);

        // If i == terminating_limb_index, *some* of the limb must be zero.
        // Assign to `terminating_limb` that we will check later.
        terminating_limb = lanes[i].madd(is_terminating_limb, terminating_limb);

        // conditionally insert terminating_index_limb_addition and/or terminating_block_limb_addition into limb
        // (addition is as good as "insertion" as we check the original byte value at this position is 0)
        lanes[i] = terminating_index_limb_addition.madd(is_terminating_limb, lanes[i]);
        lanes[i] = terminating_block_limb_addition.madd(is_terminating_block_limb, lanes[i]);
    }

    // check terminating_limb has correct number of zeroes
    {
        // we know terminating_limb < 2^64
        // offset of first zero byte = (num_bytes % 8)
        // i.e. in our 8-byte limb, bytes[(8 - offset), ..., 7] are zeroes in little-endian form
        // i.e. we multiply the limb by the above, the result should still be < 2^64 (but only if excess bytes are 0)
        field_ct limb_shift = field_ct(256).pow(field_ct(8) - terminating_index_bytes_shift);
        field_ct to_constrain = terminating_limb * limb_shift;
        to_constrain.create_range_constraint(WORD_SIZE * 8);
    }
    return lanes;
}

// Returns the keccak f1600 permutation of the input state
// We first convert the state into 'extended' representation, along with the 'twisted' state
// and then we call keccakf1600() with this keccak 'internal state'
// Finally, we convert back the state from the extented representation
template <typename Builder>
std::array<field_t<Builder>, keccak<Builder>::NUM_KECCAK_LANES> keccak<Builder>::permutation_opcode(
    std::array<field_t<Builder>, NUM_KECCAK_LANES> state, Builder* ctx)
{
    std::vector<field_t<Builder>> converted_buffer(NUM_KECCAK_LANES);
    std::vector<field_t<Builder>> msb_buffer(NUM_KECCAK_LANES);
    // populate keccak_state, convert our 64-bit lanes into an extended base-11 representation
    keccak_state internal;
    internal.context = ctx;
    for (size_t i = 0; i < state.size(); ++i) {
        const auto accumulators = plookup_read<Builder>::get_lookup_accumulators(KECCAK_FORMAT_INPUT, state[i]);
        internal.state[i] = accumulators[ColumnIdx::C2][0];
        internal.state_msb[i] = accumulators[ColumnIdx::C3][accumulators[ColumnIdx::C3].size() - 1];
    }
    compute_twisted_state(internal);
    keccakf1600(internal);
    // we convert back to the normal lanes
    return extended_2_normal(internal);
}

// This function is similar to sponge_absorb()
// but it uses permutation_opcode() instead of calling directly keccakf1600().
// As a result, this function is less efficient and should only be used to test permutation_opcode()
template <typename Builder>
void keccak<Builder>::sponge_absorb_with_permutation_opcode(keccak_state& internal,
                                                            std::vector<field_t<Builder>>& input_buffer,
                                                            const size_t input_size)
{
    // populate keccak_state
    const size_t num_blocks = input_size / (BLOCK_SIZE / 8);
    for (size_t i = 0; i < num_blocks; ++i) {
        if (i == 0) {
            for (size_t j = 0; j < LIMBS_PER_BLOCK; ++j) {
                internal.state[j] = input_buffer[j];
            }
            for (size_t j = LIMBS_PER_BLOCK; j < NUM_KECCAK_LANES; ++j) {
                internal.state[j] = witness_ct::create_constant_witness(internal.context, 0);
            }
        } else {
            for (size_t j = 0; j < LIMBS_PER_BLOCK; ++j) {
                internal.state[j] = stdlib::logic<Builder>::create_logic_constraint(
                    internal.state[j], input_buffer[i * LIMBS_PER_BLOCK + j], 64, true);
            }
        }
        internal.state = permutation_opcode(internal.state, internal.context);
    }
}

// This function computes the keccak hash, like the hash() function
// but it uses permutation_opcode() instead of calling directly keccakf1600().
// As a result, this function is less efficient and should only be used to test permutation_opcode()
template <typename Builder>
stdlib::byte_array<Builder> keccak<Builder>::hash_using_permutation_opcode(byte_array_ct& input,
                                                                           const uint32_ct& num_bytes)
{
    auto ctx = input.get_context();

    ASSERT(uint256_t(num_bytes.get_value()) == input.size());

    if (ctx == nullptr) {
        // if buffer is constant compute hash and return w/o creating constraints
        byte_array_ct output(nullptr, 32);
        const std::vector<uint8_t> result = hash_native(input.get_value());
        for (size_t i = 0; i < 32; ++i) {
            output.set_byte(i, result[i]);
        }
        return output;
    }

    // convert the input byte array into 64-bit keccak lanes (+ apply padding)
    auto formatted_slices = format_input_lanes(input, num_bytes);

    keccak_state internal;
    internal.context = ctx;
    uint32_ct num_blocks_with_data = (num_bytes + BLOCK_SIZE) / BLOCK_SIZE;
    sponge_absorb_with_permutation_opcode(internal, formatted_slices, formatted_slices.size());

    auto result = sponge_squeeze_for_permutation_opcode(internal.state, ctx);

    return result;
}

template <typename Builder>
stdlib::byte_array<Builder> keccak<Builder>::hash(byte_array_ct& input, const uint32_ct& num_bytes)
{
    auto ctx = input.get_context();

    ASSERT(uint256_t(num_bytes.get_value()) <= input.size());

    if (ctx == nullptr) {
        // if buffer is constant compute hash and return w/o creating constraints
        byte_array_ct output(nullptr, 32);
        const std::vector<uint8_t> result = hash_native(input.get_value());
        for (size_t i = 0; i < 32; ++i) {
            output.set_byte(i, result[i]);
        }
        return output;
    }

    // convert the input byte array into 64-bit keccak lanes (+ apply padding)
    auto formatted_slices = format_input_lanes(input, num_bytes);

    std::vector<field_ct> converted_buffer(formatted_slices.size());
    std::vector<field_ct> msb_buffer(formatted_slices.size());

    // populate keccak_state, convert our 64-bit lanes into an extended base-11 representation
    keccak_state internal;
    internal.context = ctx;
    for (size_t i = 0; i < formatted_slices.size(); ++i) {
        const auto accumulators =
            plookup_read<Builder>::get_lookup_accumulators(KECCAK_FORMAT_INPUT, formatted_slices[i]);
        converted_buffer[i] = accumulators[ColumnIdx::C2][0];
        msb_buffer[i] = accumulators[ColumnIdx::C3][accumulators[ColumnIdx::C3].size() - 1];
    }

    uint32_ct num_blocks_with_data = (num_bytes + BLOCK_SIZE) / BLOCK_SIZE;
    sponge_absorb(internal, converted_buffer, msb_buffer, field_ct(num_blocks_with_data));

    auto result = sponge_squeeze(internal);

    return result;
}

// Convert the 'extended' representation of the internal Keccak state into the usual array of 64 bits lanes
template <typename Builder>
std::array<field_t<Builder>, keccak<Builder>::NUM_KECCAK_LANES> keccak<Builder>::extended_2_normal(
    keccak_state& internal)
{
    std::array<field_t<Builder>, NUM_KECCAK_LANES> conversion;

    // Each hash limb represents a little-endian integer. Need to reverse bytes before we write into the output array
    for (size_t i = 0; i < internal.state.size(); ++i) {
        field_ct output_limb = plookup_read<Builder>::read_from_1_to_2_table(KECCAK_FORMAT_OUTPUT, internal.state[i]);
        conversion[i] = output_limb;
    }

    return conversion;
}

// This function is the same as sponge_squeeze, except that it does not convert
// from extended representation and assumes the input has already being converted
template <typename Builder>
stdlib::byte_array<Builder> keccak<Builder>::sponge_squeeze_for_permutation_opcode(
    std::array<field_t<Builder>, NUM_KECCAK_LANES> lanes, Builder* context)
{
    byte_array_ct result(context);

    // Each hash limb represents a little-endian integer. Need to reverse bytes before we write into the output array
    for (size_t i = 0; i < 4; ++i) {
        byte_array_ct limb_bytes(lanes[i], 8);
        byte_array_ct little_endian_limb_bytes(context, 8);
        little_endian_limb_bytes.set_byte(0, limb_bytes[7]);
        little_endian_limb_bytes.set_byte(1, limb_bytes[6]);
        little_endian_limb_bytes.set_byte(2, limb_bytes[5]);
        little_endian_limb_bytes.set_byte(3, limb_bytes[4]);
        little_endian_limb_bytes.set_byte(4, limb_bytes[3]);
        little_endian_limb_bytes.set_byte(5, limb_bytes[2]);
        little_endian_limb_bytes.set_byte(6, limb_bytes[1]);
        little_endian_limb_bytes.set_byte(7, limb_bytes[0]);
        result.write(little_endian_limb_bytes);
    }
    return result;
}
template class keccak<bb::UltraCircuitBuilder>;
template class keccak<bb::GoblinUltraCircuitBuilder>;

} // namespace bb::stdlib
