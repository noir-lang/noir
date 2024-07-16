#pragma once
#include <array>
#include <tuple>

#include "barretenberg/common/constexpr_utils.hpp"
#include "barretenberg/polynomials/polynomial.hpp"
#include "barretenberg/polynomials/univariate.hpp"
#include "barretenberg/relations/relation_types.hpp"

namespace bb {

template <typename FF_> class ECCVMLookupRelationImpl {
  public:
    using FF = FF_;
    static constexpr size_t READ_TERMS = 4;
    static constexpr size_t WRITE_TERMS = 2;
    // 1 + polynomial degree of this relation
    static constexpr size_t LENGTH = READ_TERMS + WRITE_TERMS + 3; // 9

    static constexpr std::array<size_t, 2> SUBRELATION_PARTIAL_LENGTHS{
        LENGTH, // grand product construction sub-relation
        LENGTH  // left-shiftable polynomial sub-relation
    };
    /**
     * @brief For ZK-Flavors: Upper bound on the degrees of subrelations considered as polynomials only in witness
polynomials,
     * i.e. all selectors and public polynomials are treated as constants. The subrelation witness degree does not
     * exceed the subrelation partial degree given by LENGTH - 1.
     */
    static constexpr std::array<size_t, 2> SUBRELATION_WITNESS_DEGREES{
        LENGTH - 1, // grand product construction sub-relation
        LENGTH - 1  // left-shiftable polynomial sub-relation
    };

    static constexpr std::array<bool, 2> SUBRELATION_LINEARLY_INDEPENDENT = { true, false };

    template <typename AllValues> static bool operation_exists_at_row(const AllValues& row)

    {
        return (row.msm_add == 1) || (row.msm_skew == 1) || (row.precompute_select == 1);
    }

    /**
     * @brief Get the inverse lookup polynomial
     *
     * @tparam AllEntities
     * @param in
     * @return auto&
     */
    template <typename AllEntities> static auto& get_inverse_polynomial(AllEntities& in) { return in.lookup_inverses; }

    template <typename Accumulator, typename AllEntities>
    static Accumulator compute_inverse_exists(const AllEntities& in)
    {
        using View = typename Accumulator::View;

        const auto row_has_write = View(in.precompute_select);
        const auto row_has_read = View(in.msm_add) + View(in.msm_skew);
        return row_has_write + row_has_read - (row_has_write * row_has_read);
    }

    template <typename Accumulator, size_t index, typename AllEntities>
    static Accumulator lookup_read_counts(const AllEntities& in)
    {
        using View = typename Accumulator::View;

        if constexpr (index == 0) {
            return Accumulator(View(in.lookup_read_counts_0));
        }
        if constexpr (index == 1) {
            return Accumulator(View(in.lookup_read_counts_1));
        }
        return Accumulator(1);
    }

    template <typename Accumulator, size_t read_index, typename AllEntities>
    static Accumulator compute_read_term_predicate(const AllEntities& in)

    {
        using View = typename Accumulator::View;

        if constexpr (read_index == 0) {
            return Accumulator(View(in.msm_add1));
        }
        if constexpr (read_index == 1) {
            return Accumulator(View(in.msm_add2));
        }
        if constexpr (read_index == 2) {
            return Accumulator(View(in.msm_add3));
        }
        if constexpr (read_index == 3) {
            return Accumulator(View(in.msm_add4));
        }
        return Accumulator(1);
    }

    template <typename Accumulator, size_t write_index, typename AllEntities>
    static Accumulator compute_write_term_predicate(const AllEntities& in)
    {
        using View = typename Accumulator::View;

        if constexpr (write_index == 0) {
            return Accumulator(View(in.precompute_select));
        }
        if constexpr (write_index == 1) {
            // TODO(https://github.com/AztecProtocol/barretenberg/issues/750) Is this a bug?
            return Accumulator(View(in.precompute_select));
        }
        return Accumulator(1);
    }

    template <typename Accumulator, size_t write_index, typename AllEntities, typename Parameters>
    static Accumulator compute_write_term(const AllEntities& in, const Parameters& params)
    {
        using View = typename Accumulator::View;

        static_assert(write_index < WRITE_TERMS);

        // what are we looking up?
        // we want to map:
        // 1: point pc
        // 2: point slice
        // 3: point x
        // 4: point y
        // for each point in our point table, we want to map `slice` to (x, -y) AND `slice + 8` to (x, y)

        // round starts at 0 and increments to 7
        // point starts at 15[P] and decrements to [P]
        // a slice value of 0 maps to -15[P]
        // 1 -> -13[P]
        // 7 -> -[P]
        // 8 -> P
        // 15 -> 15[P]
        // negative points map pc, round, x, -y
        // positive points map pc, 15 - (round * 2), x, y
        const auto& precompute_pc = View(in.precompute_pc);
        const auto& tx = View(in.precompute_tx);
        const auto& ty = View(in.precompute_ty);
        const auto& precompute_round = View(in.precompute_round);
        const auto& gamma = params.gamma;
        const auto& beta = params.beta;
        const auto& beta_sqr = params.beta_sqr;
        const auto& beta_cube = params.beta_cube;

        // slice value : (wnaf value) : lookup term
        // 0 : -15 : 0
        // 1 : -13 : 1
        // 7 : -1 : 7
        // 8 : 1 : 0
        // 9 : 3 : 1
        // 15 : 15 : 7

        // slice value : negative term : positive term
        // 0 : 0 : 7
        // 1 : 1 : 6
        // 2 : 2 : 5
        // 3 : 3 : 4
        // 7 : 7 : 0

        // | 0 | 15[P].x | 15[P].y  | 0, -15[P].x, -15[P].y | 15, 15[P].x, 15[P].y |
        // | 1 | 13[P].x | 13[P].y | 1, -13[P].x, -13[P].y | 14, 13[P].x, 13[P].y
        // | 2 | 11[P].x | 11[P].y
        // | 3 |  9[P].x |  9[P].y
        // | 4 |  7[P].x |  7[P].y
        // | 5 |  5[P].x |  5[P].y
        // | 6 |  3[P].x |  3[P].y
        // | 7 |  1[P].x |  1[P].y | 7, -[P].x, -[P].y | 8 , [P].x, [P].y |

        const auto negative_term = precompute_pc + gamma + precompute_round * beta + tx * beta_sqr - ty * beta_cube;
        const auto positive_slice_value = -(precompute_round) + 15;
        const auto positive_term = precompute_pc + gamma + positive_slice_value * beta + tx * beta_sqr + ty * beta_cube;

        // todo optimize this?
        if constexpr (write_index == 0) {
            return positive_term; // degree 1
        }
        if constexpr (write_index == 1) {
            return negative_term; // degree 1
        }
        return Accumulator(1);
    }

    template <typename Accumulator, size_t read_index, typename AllEntities, typename Parameters>
    static Accumulator compute_read_term(const AllEntities& in, const Parameters& params)
    {
        using View = typename Accumulator::View;

        // read term:
        // pc, slice, x, y
        static_assert(read_index < READ_TERMS);
        const auto& gamma = params.gamma;
        const auto& beta = params.beta;
        const auto& beta_sqr = params.beta_sqr;
        const auto& beta_cube = params.beta_cube;
        const auto& msm_pc = View(in.msm_pc);
        const auto& msm_count = View(in.msm_count);
        const auto& msm_slice1 = View(in.msm_slice1);
        const auto& msm_slice2 = View(in.msm_slice2);
        const auto& msm_slice3 = View(in.msm_slice3);
        const auto& msm_slice4 = View(in.msm_slice4);
        const auto& msm_x1 = View(in.msm_x1);
        const auto& msm_x2 = View(in.msm_x2);
        const auto& msm_x3 = View(in.msm_x3);
        const auto& msm_x4 = View(in.msm_x4);
        const auto& msm_y1 = View(in.msm_y1);
        const auto& msm_y2 = View(in.msm_y2);
        const auto& msm_y3 = View(in.msm_y3);
        const auto& msm_y4 = View(in.msm_y4);

        // how do we get pc value
        // row pc = value of pc after msm
        // row count = num processed points in round
        // size_of_msm = msm_size
        // value of pc at start of msm = msm_pc - msm_size_of_msm
        // value of current pc = msm_pc - msm_size_of_msm + msm_count + (0,1,2,3)
        const auto current_pc = msm_pc - msm_count;

        const auto read_term1 = (current_pc) + gamma + msm_slice1 * beta + msm_x1 * beta_sqr + msm_y1 * beta_cube;
        const auto read_term2 = (current_pc - 1) + gamma + msm_slice2 * beta + msm_x2 * beta_sqr + msm_y2 * beta_cube;
        const auto read_term3 = (current_pc - 2) + gamma + msm_slice3 * beta + msm_x3 * beta_sqr + msm_y3 * beta_cube;
        const auto read_term4 = (current_pc - 3) + gamma + msm_slice4 * beta + msm_x4 * beta_sqr + msm_y4 * beta_cube;

        if constexpr (read_index == 0) {
            return read_term1; // degree 1
        }
        if constexpr (read_index == 1) {
            return read_term2; // degree 1
        }
        if constexpr (read_index == 2) {
            return read_term3; // degree 1
        }
        if constexpr (read_index == 3) {
            return read_term4; // degree 1
        }
        return Accumulator(1);
    }

    /**
     * @brief Expression for ECCVM lookup tables.
     * @details We use log-derivative lookup tables for the following case:
     * Table writes: ECCVMPointTable columns: we define Straus point table:
     * { {0, -15[P]}, {1, -13[P]}, ..., {15, 15[P]} }
     * write source: { precompute_round, precompute_tx, precompute_ty }
     * Table reads: ECCVMMSM columns. Each row adds up to 4 points into MSM accumulator
     * read source: { msm_slice1, msm_x1, msm_y1 }, ..., { msm_slice4, msm_x4, msm_y4 }
     * @param accumulator transformed to `evals + C(in(X)...)*scaling_factor`
     * @param in an std::array containing the fully extended Accumulator edges.
     * @param relation_params contains beta, gamma, and public_input_delta, ....
     * @param scaling_factor optional term to scale the evaluation before adding to evals.
     */
    template <typename ContainerOverSubrelations, typename AllEntities, typename Parameters>
    static void accumulate(ContainerOverSubrelations& accumulator,
                           const AllEntities& in,
                           const Parameters& params,
                           const FF& scaling_factor);
};

template <typename FF> using ECCVMLookupRelation = Relation<ECCVMLookupRelationImpl<FF>>;

} // namespace bb