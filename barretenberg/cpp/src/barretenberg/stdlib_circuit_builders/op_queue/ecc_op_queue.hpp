#pragma once

#include "barretenberg/ecc/curves/bn254/bn254.hpp"
#include "barretenberg/eccvm/eccvm_builder_types.hpp"
#include "barretenberg/stdlib/primitives/bigfield/constants.hpp"
namespace bb {

enum EccOpCode { NULL_OP, ADD_ACCUM, MUL_ACCUM, EQUALITY };

struct UltraOp {
    using Fr = curve::BN254::ScalarField;
    EccOpCode op_code = NULL_OP;
    Fr op;
    Fr x_lo;
    Fr x_hi;
    Fr y_lo;
    Fr y_hi;
    Fr z_1;
    Fr z_2;
};

/**
 * @brief Used to construct execution trace representations of elliptic curve operations.
 *
 * @details Currently the targets in execution traces are: four advice wires in UltraCircuitBuilder and 5 wires in the
 * ECCVM. In each case, the variable values are stored in this class, since the same values will need to be used later
 * by the TranslationVMCircuitBuilder. The circuit builders will store witness indices which are indices in the
 * ultra (resp. eccvm) ops members of this class (rather than in the builder's variables array).
 */
class ECCOpQueue {
    using Curve = curve::BN254;
    using Point = Curve::AffineElement;
    using Fr = Curve::ScalarField;
    using Fq = Curve::BaseField; // Grumpkin's scalar field
    Point point_at_infinity = Curve::Group::affine_point_at_infinity;

    // The operations written to the queue are also performed natively; the result is stored in accumulator
    Point accumulator = point_at_infinity;

    static constexpr size_t DEFAULT_NON_NATIVE_FIELD_LIMB_BITS = stdlib::NUM_LIMB_BITS_IN_FIELD_SIMULATION;

    std::vector<bb::eccvm::VMOperation<Curve::Group>> raw_ops;
    std::array<std::vector<Fr>, 4> ultra_ops; // ops encoded in the width-4 Ultra format

    size_t current_ultra_ops_size = 0;  // M_i
    size_t previous_ultra_ops_size = 0; // M_{i-1}

    std::array<Point, 4> ultra_ops_commitments;

  public:
    using ECCVMOperation = bb::eccvm::VMOperation<Curve::Group>;

    // as we populate the op_queue, we track the number of rows in each circuit section,
    // as well as the number of multiplications performed.
    // This is to avoid expensive O(n) logic to compute the number of rows and muls during witness computation
    uint32_t cached_num_muls = 0;
    uint32_t cached_active_msm_count = 0;
    uint32_t num_transcript_rows = 0;
    uint32_t num_precompute_table_rows = 0;
    uint32_t num_msm_rows = 0;

    const std::vector<ECCVMOperation>& get_raw_ops() { return raw_ops; }

    // TODO(https://github.com/AztecProtocol/barretenberg/issues/905): Can remove this with better handling of scalar
    // mul against 0
    void append_nonzero_ops()
    {
        // Add an element and scalar the accumulation of which leaves no Point-at-Infinity commitments
        const auto x = uint256_t(0xd3c208c16d87cfd3, 0xd97816a916871ca8, 0x9b85045b68181585, 0x30644e72e131a02);
        const auto y = uint256_t(0x3ce1cc9c7e645a83, 0x2edac647851e3ac5, 0xd0cbe61fced2bc53, 0x1a76dae6d3272396);
        auto padding_element = Point(x, y);
        auto padding_scalar = -Fr::one();
        mul_accumulate(padding_element, padding_scalar);
        eq_and_reset();
    }

    /**
     * @brief A fuzzing only method for setting raw ops directly
     *
     */
    void set_raw_ops_for_fuzzing(std::vector<ECCVMOperation>& raw_ops_in) { raw_ops = raw_ops_in; }

    /**
     * @brief A testing only method that adds an erroneous equality op to the raw ops
     * @brief May be used to ensure that ECCVM responds as expected when encountering a bad op
     *
     */
    void add_erroneous_equality_op_for_testing()
    {
        raw_ops.emplace_back(ECCVMOperation{ .add = false,
                                             .mul = false,
                                             .eq = true,
                                             .reset = true,
                                             .base_point = Point::random_element(),
                                             .z1 = 0,
                                             .z2 = 0,
                                             .mul_scalar_full = 0 });
    }

    /**
     * @brief Write empty row to queue
     * @warning This is for testing purposes only. Currently no valid use case.
     *
     */
    void empty_row_for_testing()
    {
        raw_ops.emplace_back(ECCVMOperation{
            .add = false,
            .mul = false,
            .eq = false,
            .reset = false,
            .base_point = point_at_infinity,
            .z1 = 0,
            .z2 = 0,
            .mul_scalar_full = 0,
        });
        num_transcript_rows += 1;

        update_cached_msms(raw_ops.back());
    }

    Point get_accumulator() { return accumulator; }

    /**
     * @brief Prepend the information from the previous queue (used before accumulation/merge proof to be able to run
     * circuit construction separately)
     *
     * @param previous
     */
    void prepend_previous_queue(const ECCOpQueue& previous)
    {
        if (!previous.raw_ops.empty() && !raw_ops.empty()) {
            // Check we are not merging op queue that does not reset accumulator!
            // Note - eccvm does not directly constrain this to not happen. If we need such checks they need to be
            // applied when the transcript is being written into
            ASSERT(previous.raw_ops.back().eq || previous.raw_ops.back().reset);
        }
        // We shouldn't be merging if there is a previous active msm!
        ASSERT(previous.cached_active_msm_count == 0);

        cached_num_muls += previous.cached_num_muls;
        num_msm_rows += previous.num_msm_rows;
        num_precompute_table_rows += previous.num_precompute_table_rows;
        num_transcript_rows += previous.num_transcript_rows;

        // Allocate enough space
        std::vector<ECCVMOperation> raw_ops_updated(raw_ops.size() + previous.raw_ops.size());
        // Copy the previous raw ops to the beginning of the new vector
        std::copy(previous.raw_ops.begin(), previous.raw_ops.end(), raw_ops_updated.begin());
        // Copy the raw ops from current queue after the ones from the previous queue (concatenate them)
        std::copy(raw_ops.begin(),
                  raw_ops.end(),
                  std::next(raw_ops_updated.begin(), static_cast<long>(previous.raw_ops.size())));

        // Swap raw_ops underlying storage
        raw_ops.swap(raw_ops_updated);
        // Do the same 3 operations for ultra_ops
        for (size_t i = 0; i < 4; i++) {
            // Allocate new vector
            std::vector<Fr> current_ultra_op(ultra_ops[i].size() + previous.ultra_ops[i].size());
            // Copy the previous ultra ops to the beginning of the new vector
            std::copy(previous.ultra_ops[i].begin(), previous.ultra_ops[i].end(), current_ultra_op.begin());
            // Copy the ultra ops from current queue after the ones from the previous queue (concatenate them)
            std::copy(ultra_ops[i].begin(),
                      ultra_ops[i].end(),
                      std::next(current_ultra_op.begin(), static_cast<long>(previous.ultra_ops[i].size())));
            // Swap storage
            ultra_ops[i].swap(current_ultra_op);
        }
        // Update sizes
        current_ultra_ops_size += previous.ultra_ops[0].size();
        previous_ultra_ops_size += previous.ultra_ops[0].size();
        // Update commitments
        ultra_ops_commitments = previous.ultra_ops_commitments;
    }
    /**
     * @brief Prepend the information from the previous queue (used before accumulation/merge proof to be able to run
     * circuit construction separately)
     *
     * @param previous_ptr
     */
    void prepend_previous_queue(const ECCOpQueue* previous_ptr) { prepend_previous_queue(*previous_ptr); }

    /**
     * @brief Enable using std::swap on queues
     *
     */
    friend void swap(ECCOpQueue& lhs, ECCOpQueue& rhs)
    {
        // Swap vectors
        lhs.raw_ops.swap(rhs.raw_ops);
        for (size_t i = 0; i < 4; i++) {
            lhs.ultra_ops[i].swap(rhs.ultra_ops[i]);
        }
        // Swap sizes
        size_t temp = lhs.current_ultra_ops_size;
        lhs.current_ultra_ops_size = rhs.current_ultra_ops_size;
        rhs.current_ultra_ops_size = temp;
        temp = lhs.previous_ultra_ops_size;
        lhs.previous_ultra_ops_size = rhs.previous_ultra_ops_size;
        rhs.previous_ultra_ops_size = temp;
        // Swap commitments
        auto commit_temp = lhs.ultra_ops_commitments;
        lhs.ultra_ops_commitments = rhs.ultra_ops_commitments;
        rhs.ultra_ops_commitments = commit_temp;

        std::swap(lhs.cached_num_muls, rhs.cached_num_muls);
        std::swap(lhs.cached_active_msm_count, rhs.cached_active_msm_count);
        std::swap(lhs.num_transcript_rows, rhs.num_transcript_rows);
        std::swap(lhs.num_precompute_table_rows, rhs.num_precompute_table_rows);
        std::swap(lhs.num_msm_rows, rhs.num_msm_rows);
    }

    /**
     * @brief Set the current and previous size of the ultra_ops transcript
     *
     * @details previous_ultra_ops_size = M_{i-1} is needed by the prover to extract the previous aggregate op
     * queue transcript T_{i-1} from the current one T_i. This method should be called when a circuit is 'finalized'.
     */
    void set_size_data()
    {
        previous_ultra_ops_size = current_ultra_ops_size;
        current_ultra_ops_size = ultra_ops[0].size();
    }

    [[nodiscard]] size_t get_previous_size() const { return previous_ultra_ops_size; }
    [[nodiscard]] size_t get_current_size() const { return current_ultra_ops_size; }

    void set_commitment_data(std::array<Point, 4>& commitments) { ultra_ops_commitments = commitments; }
    const auto& get_ultra_ops_commitments() { return ultra_ops_commitments; }

    /**
     * @brief Get a 'view' of the current ultra ops object
     *
     * @return std::vector<std::span<Fr>>
     */
    std::vector<std::span<Fr>> get_aggregate_transcript()
    {
        std::vector<std::span<Fr>> result;
        result.reserve(ultra_ops.size());
        for (auto& entry : ultra_ops) {
            result.emplace_back(entry);
        }
        return result;
    }

    /**
     * @brief Get a 'view' of the previous ultra ops object
     *
     * @return std::vector<std::span<Fr>>
     */
    std::vector<std::span<Fr>> get_previous_aggregate_transcript()
    {
        std::vector<std::span<Fr>> result;
        result.reserve(ultra_ops.size());
        // Construct T_{i-1} as a view of size M_{i-1} into T_i
        for (auto& entry : ultra_ops) {
            result.emplace_back(entry.begin(), previous_ultra_ops_size);
        }
        return result;
    }

    /**
     * @brief Get the number of rows in the 'msm' column section o the ECCVM, associated with a single multiscalar mul
     *
     * @param msm_count
     * @return uint32_t
     */
    static uint32_t get_msm_row_count_for_single_msm(const size_t msm_count)
    {
        const size_t rows_per_round =
            (msm_count / eccvm::ADDITIONS_PER_ROW) + (msm_count % eccvm::ADDITIONS_PER_ROW != 0 ? 1 : 0);
        constexpr size_t num_rounds = eccvm::NUM_SCALAR_BITS / eccvm::WNAF_SLICE_BITS;
        const size_t num_rows_for_all_rounds = (num_rounds + 1) * rows_per_round; // + 1 round for skew
        const size_t num_double_rounds = num_rounds - 1;
        const size_t num_rows_for_msm = num_rows_for_all_rounds + num_double_rounds;

        return static_cast<uint32_t>(num_rows_for_msm);
    }

    /**
     * @brief Get the number of rows in the 'msm' column section, for all msms in the circuit
     *
     * @return size_t
     */
    size_t get_num_msm_rows() const
    {
        size_t msm_rows = num_msm_rows + 2;
        if (cached_active_msm_count > 0) {
            msm_rows += get_msm_row_count_for_single_msm(cached_active_msm_count);
        }
        return msm_rows;
    }

    /**
     * @brief Get the number of rows for the current ECCVM circuit
     *
     * @return size_t
     */
    size_t get_num_rows() const
    {
        // add 1 row to start and end of transcript and msm sections
        const size_t transcript_rows = num_transcript_rows + 2;
        size_t msm_rows = num_msm_rows + 2;
        // add 1 row to start of precompute table section
        size_t precompute_rows = num_precompute_table_rows + 1;
        if (cached_active_msm_count > 0) {
            msm_rows += get_msm_row_count_for_single_msm(cached_active_msm_count);
            precompute_rows += get_precompute_table_row_count_for_single_msm(cached_active_msm_count);
        }

        return std::max(transcript_rows, std::max(msm_rows, precompute_rows));
    }

    /**
     * @brief Write point addition op to queue and natively perform addition
     *
     * @param to_add
     */
    UltraOp add_accumulate(const Point& to_add)
    {
        // Update the accumulator natively
        accumulator = accumulator + to_add;

        // Construct and store the operation in the ultra op format
        auto ultra_op = construct_and_populate_ultra_ops(ADD_ACCUM, to_add);

        // Store the raw operation
        raw_ops.emplace_back(ECCVMOperation{
            .add = true,
            .mul = false,
            .eq = false,
            .reset = false,
            .base_point = to_add,
            .z1 = 0,
            .z2 = 0,
            .mul_scalar_full = 0,
        });
        num_transcript_rows += 1;
        update_cached_msms(raw_ops.back());

        return ultra_op;
    }

    /**
     * @brief Write multiply and add op to queue and natively perform operation
     *
     * @param to_add
     */
    UltraOp mul_accumulate(const Point& to_mul, const Fr& scalar)
    {
        // Update the accumulator natively
        accumulator = accumulator + to_mul * scalar;

        // Construct and store the operation in the ultra op format
        auto ultra_op = construct_and_populate_ultra_ops(MUL_ACCUM, to_mul, scalar);

        // Store the raw operation
        raw_ops.emplace_back(ECCVMOperation{
            .add = false,
            .mul = true,
            .eq = false,
            .reset = false,
            .base_point = to_mul,
            .z1 = ultra_op.z_1,
            .z2 = ultra_op.z_2,
            .mul_scalar_full = scalar,
        });
        num_transcript_rows += 1;
        update_cached_msms(raw_ops.back());

        return ultra_op;
    }

    /**
     * @brief Write equality op using internal accumulator point
     *
     * @return current internal accumulator point (prior to reset to 0)
     */
    UltraOp eq_and_reset()
    {
        auto expected = accumulator;
        accumulator.self_set_infinity();

        // Construct and store the operation in the ultra op format
        auto ultra_op = construct_and_populate_ultra_ops(EQUALITY, expected);

        // Store raw operation
        raw_ops.emplace_back(ECCVMOperation{
            .add = false,
            .mul = false,
            .eq = true,
            .reset = true,
            .base_point = expected,
            .z1 = 0,
            .z2 = 0,
            .mul_scalar_full = 0,
        });
        num_transcript_rows += 1;
        update_cached_msms(raw_ops.back());

        return ultra_op;
    }

  private:
    /**
     * @brief when inserting operations, update the number of multiplications in the latest scalar mul
     *
     * @param op
     */
    void update_cached_msms(const ECCVMOperation& op)
    {
        if (op.mul) {
            if (op.z1 != 0) {
                cached_active_msm_count++;
            }
            if (op.z2 != 0) {
                cached_active_msm_count++;
            }
        } else if (cached_active_msm_count != 0) {
            num_msm_rows += get_msm_row_count_for_single_msm(cached_active_msm_count);
            num_precompute_table_rows += get_precompute_table_row_count_for_single_msm(cached_active_msm_count);
            cached_num_muls += cached_active_msm_count;
            cached_active_msm_count = 0;
        }
    }

    /**
     * @brief Get the precompute table row count for single msm object
     *
     * @param msm_count
     * @return uint32_t
     */
    static uint32_t get_precompute_table_row_count_for_single_msm(const size_t msm_count)
    {
        constexpr size_t num_precompute_rows_per_scalar = eccvm::NUM_WNAF_SLICES / eccvm::WNAF_SLICES_PER_ROW;
        const size_t num_rows_for_precompute_table = msm_count * num_precompute_rows_per_scalar;
        return static_cast<uint32_t>(num_rows_for_precompute_table);
    }

    /**
     * @brief Given an ecc operation and its inputs, decompose into ultra format and populate ultra_ops
     *
     * @param op_code
     * @param point
     * @param scalar
     * @return UltraOp
     */
    UltraOp construct_and_populate_ultra_ops(EccOpCode op_code, const Point& point, const Fr& scalar = Fr::zero())
    {
        UltraOp ultra_op;
        ultra_op.op_code = op_code;
        ultra_op.op = Fr(static_cast<uint256_t>(op_code));

        // Decompose point coordinates (Fq) into hi-lo chunks (Fr)
        const size_t CHUNK_SIZE = 2 * DEFAULT_NON_NATIVE_FIELD_LIMB_BITS;
        auto x_256 = uint256_t(point.x);
        auto y_256 = uint256_t(point.y);
        ultra_op.x_lo = Fr(x_256.slice(0, CHUNK_SIZE));
        ultra_op.x_hi = Fr(x_256.slice(CHUNK_SIZE, CHUNK_SIZE * 2));
        ultra_op.y_lo = Fr(y_256.slice(0, CHUNK_SIZE));
        ultra_op.y_hi = Fr(y_256.slice(CHUNK_SIZE, CHUNK_SIZE * 2));

        // Split scalar into 128 bit endomorphism scalars
        Fr z_1 = 0;
        Fr z_2 = 0;
        auto converted = scalar.from_montgomery_form();
        Fr::split_into_endomorphism_scalars(converted, z_1, z_2);
        ultra_op.z_1 = z_1.to_montgomery_form();
        ultra_op.z_2 = z_2.to_montgomery_form();

        append_to_ultra_ops(ultra_op);

        return ultra_op;
    }

    /**
     * @brief Populate two rows of the ultra ops,representing a complete ECC operation
     * @note Only the first 'op' field is utilized so the second is explicitly set to 0
     *
     * @param tuple
     */
    void append_to_ultra_ops(UltraOp tuple)
    {
        ultra_ops[0].emplace_back(tuple.op);
        ultra_ops[1].emplace_back(tuple.x_lo);
        ultra_ops[2].emplace_back(tuple.x_hi);
        ultra_ops[3].emplace_back(tuple.y_lo);

        ultra_ops[0].emplace_back(0);
        ultra_ops[1].emplace_back(tuple.y_hi);
        ultra_ops[2].emplace_back(tuple.z_1);
        ultra_ops[3].emplace_back(tuple.z_2);
    }
};

} // namespace bb
