#pragma once
#include <array>
#include <tuple>

#include "barretenberg/common/constexpr_utils.hpp"
#include "barretenberg/honk/proof_system/logderivative_library.hpp"
#include "barretenberg/polynomials/polynomial.hpp"
#include "barretenberg/polynomials/univariate.hpp"
#include "barretenberg/relations/relation_types.hpp"

namespace proof_system {

template <typename FF_> class DatabusLookupRelationImpl {
  public:
    using FF = FF_;
    static constexpr size_t READ_TERMS = 1;
    static constexpr size_t WRITE_TERMS = 1;
    // 1 + polynomial degree of this relation
    static constexpr size_t LENGTH = READ_TERMS + WRITE_TERMS + 3;

    static constexpr std::array<size_t, 2> SUBRELATION_PARTIAL_LENGTHS{
        LENGTH, // inverse polynomial correctness subrelation
        LENGTH  // log-derivative lookup argument subrelation
    };

    // The second subrelation is "linearly dependant" in the sense that it establishes the value of a sum across the
    // entire execution trace rather than a per-row identity.
    static constexpr std::array<bool, 2> SUBRELATION_LINEARLY_INDEPENDENT = { true, false };

    /**
     * @brief Determine whether the inverse I needs to be computed at a given row
     * @details The value of the inverse polynomial I(X) only needs to be computed when the databus lookup gate is
     * "active". Otherwise it is set to 0. This method allows for determination of when the inverse should be computed.
     *
     * @tparam AllValues
     * @param row
     * @return true
     * @return false
     */
    template <typename AllValues> static bool operation_exists_at_row(const AllValues& row)
    {
        return (row.q_busread == 1 || row.calldata_read_counts > 0);
    }

    /**
     * @brief Get the lookup inverse polynomial
     *
     * @tparam AllEntities
     * @param in
     * @return auto&
     */
    template <typename AllEntities> static auto& get_inverse_polynomial(AllEntities& in) { return in.lookup_inverses; }
    /**
     * @brief Compute the Accumulator whose values indicate whether the inverse is computed or not
     * @details This is needed for efficiency since we don't need to compute the inverse unless the log derivative
     * lookup relation is active at a given row.
     *
     */
    template <typename Accumulator, typename AllEntities>
    static Accumulator compute_inverse_exists(const AllEntities& in)
    {
        using View = typename Accumulator::View;
        // TODO(luke): row_has_read should really be a boolean object thats equal to 1 when counts > 0 and 0 otherwise.
        // This current structure will lead to failure if call_data_read_counts > 1.
        const auto row_has_write = View(in.q_busread);
        const auto row_has_read = View(in.calldata_read_counts);

        return row_has_write + row_has_read - (row_has_write * row_has_read);

        return Accumulator(View(in.q_busread) + View(in.calldata_read_counts));
    }

    template <typename Accumulator, size_t index, typename AllEntities>
    static Accumulator lookup_read_counts(const AllEntities& in)
    {
        using View = typename Accumulator::View;

        if constexpr (index == 0) {
            return Accumulator(View(in.calldata_read_counts));
        }
        return Accumulator(1);
    }

    /**
     * @brief Compute scalar for read term in log derivative lookup argument
     *
     */
    template <typename Accumulator, size_t read_index, typename AllEntities>
    static Accumulator compute_read_term_predicate([[maybe_unused]] const AllEntities& in)

    {
        using View = typename Accumulator::View;

        if constexpr (read_index == 0) {
            return Accumulator(View(in.q_busread));
        }
        return Accumulator(1);
    }

    /**
     * @brief Compute scalar for write term in log derivative lookup argument
     *
     */
    template <typename Accumulator, size_t write_index, typename AllEntities>
    static Accumulator compute_write_term_predicate(const AllEntities& /*unused*/)
    {
        return Accumulator(1);
    }

    /**
     * @brief Compute write term denominator in log derivative lookup argument
     *
     */
    template <typename Accumulator, size_t write_index, typename AllEntities, typename Parameters>
    static Accumulator compute_write_term(const AllEntities& in, const Parameters& params)
    {
        using View = typename Accumulator::View;
        using ParameterView = GetParameterView<Parameters, View>;

        static_assert(write_index < WRITE_TERMS);

        const auto& calldata = View(in.calldata);
        const auto& id = View(in.databus_id);

        const auto& gamma = ParameterView(params.gamma);
        const auto& beta = ParameterView(params.beta);

        // Construct b_i + idx_i*\beta + \gamma
        if constexpr (write_index == 0) {
            return calldata + gamma + id * beta; // degree 1
        }

        return Accumulator(1);
    }

    /**
     * @brief Compute read term denominator in log derivative lookup argument
     *
     */
    template <typename Accumulator, size_t read_index, typename AllEntities, typename Parameters>
    static Accumulator compute_read_term(const AllEntities& in, const Parameters& params)
    {
        using View = typename Accumulator::View;
        using ParameterView = GetParameterView<Parameters, View>;

        static_assert(read_index < READ_TERMS);

        // Bus value stored in w_1, index into bus column stored in w_2
        const auto& w_1 = View(in.w_l);
        const auto& w_2 = View(in.w_r);

        const auto& gamma = ParameterView(params.gamma);
        const auto& beta = ParameterView(params.beta);

        // Construct value + index*\beta + \gamma
        if constexpr (read_index == 0) {
            return w_1 + gamma + w_2 * beta;
        }

        return Accumulator(1);
    }

    /**
     * @brief Accumulate the contribution from two surelations for the log derivative databus lookup argument
     * @details See logderivative_library.hpp for details of the generic log-derivative lookup argument
     *
     * @param accumulator transformed to `evals + C(in(X)...)*scaling_factor`
     * @param in an std::array containing the fully extended Accumulator edges.
     * @param params contains beta, gamma, and public_input_delta, ....
     * @param scaling_factor optional term to scale the evaluation before adding to evals.
     */
    template <typename ContainerOverSubrelations, typename AllEntities, typename Parameters>
    static void accumulate(ContainerOverSubrelations& accumulator,
                           const AllEntities& in,
                           const Parameters& params,
                           const FF& scaling_factor)
    {
        honk::logderivative_library::
            accumulate_logderivative_lookup_subrelation_contributions<FF, DatabusLookupRelationImpl<FF>>(
                accumulator, in, params, scaling_factor);
    }
};

template <typename FF> using DatabusLookupRelation = Relation<DatabusLookupRelationImpl<FF>>;

} // namespace proof_system
