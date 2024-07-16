#pragma once
#include <array>
#include <tuple>

#include "barretenberg/common/constexpr_utils.hpp"
#include "barretenberg/honk/proof_system/logderivative_library.hpp"
#include "barretenberg/polynomials/polynomial.hpp"
#include "barretenberg/polynomials/univariate.hpp"
#include "barretenberg/relations/relation_types.hpp"

namespace bb {

template <typename FF_> class LogDerivLookupRelationImpl {
  public:
    using FF = FF_;
    static constexpr size_t READ_TERMS = 1;
    static constexpr size_t WRITE_TERMS = 1;
    // 1 + polynomial degree of this relation
    static constexpr size_t LENGTH = 5; // both subrelations are degree 4

    static constexpr std::array<size_t, 2> SUBRELATION_PARTIAL_LENGTHS{
        LENGTH, // inverse construction sub-relation
        LENGTH  // log derivative lookup argument sub-relation
    };
    /**
     * @brief For ZK-Flavors: The degrees of subrelations considered as polynomials only in witness polynomials,
     * i.e. all selectors and public polynomials are treated as constants.
     *
     */
    static constexpr std::array<size_t, 2> SUBRELATION_WITNESS_DEGREES{
        2, // inverse construction sub-relation
        3, // log derivative lookup argument sub-relation
    };

    // TODO(https://github.com/AztecProtocol/barretenberg/issues/1036): Scrutinize these adjustment factors. Counting
    // degrees suggests the first subrelation should require an adjustment of 2.
    static constexpr std::array<size_t, 2> TOTAL_LENGTH_ADJUSTMENTS{
        1, // inverse construction sub-relation
        1  // log derivative lookup argument sub-relation
    };

    static constexpr std::array<bool, 2> SUBRELATION_LINEARLY_INDEPENDENT = { true, false };

    template <typename AllEntities> inline static bool skip(const AllEntities& in)
    {
        // Ensure the input does not contain a lookup gate or data that is being read
        return in.q_lookup.is_zero() && in.lookup_read_counts.is_zero();
    }

    /**
     * @brief Does the provided row contain data relevant to table lookups; Used to determine whether the polynomial of
     * inverses must be computed at a given row
     * @details In order to avoid unnecessary computation, the polynomial of inverses I is only computed for rows at
     * which the lookup relation is "active". It is active if either (1) the present row contains a lookup gate (i.e.
     * q_lookup == 1), or (2) the present row contains table data that has been looked up in this circuit
     * (lookup_read_tags == 1, or equivalently, if the row in consideration has index i, the data in polynomials table_i
     * has been utlized in the circuit).
     *
     */
    template <typename AllValues> static bool operation_exists_at_row(const AllValues& row)
    {
        // is the row a lookup gate or does it contain table data that has been read at some point in this circuit
        return (row.q_lookup == 1) || (row.lookup_read_tags == 1);
    }

    // Get the inverse polynomial for this relation
    template <typename AllEntities> static auto& get_inverse_polynomial(AllEntities& in) { return in.lookup_inverses; }

    // Used in the inverse correctness subrelation; facilitates only computing inverses where necessary
    template <typename Accumulator, typename AllEntities>
    static Accumulator compute_inverse_exists(const AllEntities& in)
    {
        using View = typename Accumulator::View;

        const auto row_has_write = View(in.lookup_read_tags);
        const auto row_has_read = View(in.q_lookup);
        return row_has_write + row_has_read - (row_has_write * row_has_read);
    }

    template <typename Accumulator, size_t index, typename AllEntities>
    static Accumulator lookup_read_counts(const AllEntities& in)
    {
        using View = typename Accumulator::View;
        return Accumulator(View(in.lookup_read_counts));
    }

    // Compute table_1 + gamma + table_2 * eta + table_3 * eta_2 + table_4 * eta_3
    template <typename Accumulator, size_t write_index, typename AllEntities, typename Parameters>
    static Accumulator compute_write_term(const AllEntities& in, const Parameters& params)
    {
        using View = typename Accumulator::View;
        using ParameterView = GetParameterView<Parameters, View>;

        static_assert(write_index < WRITE_TERMS);

        const auto& gamma = ParameterView(params.gamma);
        const auto& eta = ParameterView(params.eta);
        const auto& eta_two = ParameterView(params.eta_two);
        const auto& eta_three = ParameterView(params.eta_three);

        auto table_1 = View(in.table_1);
        auto table_2 = View(in.table_2);
        auto table_3 = View(in.table_3);
        auto table_4 = View(in.table_4);

        return table_1 + gamma + table_2 * eta + table_3 * eta_two + table_4 * eta_three;
    }

    template <typename Accumulator, size_t read_index, typename AllEntities, typename Parameters>
    static Accumulator compute_read_term(const AllEntities& in, const Parameters& params)
    {
        using View = typename Accumulator::View;
        using ParameterView = GetParameterView<Parameters, View>;

        const auto& gamma = ParameterView(params.gamma);
        const auto& eta = ParameterView(params.eta);
        const auto& eta_two = ParameterView(params.eta_two);
        const auto& eta_three = ParameterView(params.eta_three);

        auto w_1 = View(in.w_l);
        auto w_2 = View(in.w_r);
        auto w_3 = View(in.w_o);

        auto w_1_shift = View(in.w_l_shift);
        auto w_2_shift = View(in.w_r_shift);
        auto w_3_shift = View(in.w_o_shift);

        auto table_index = View(in.q_o);
        auto negative_column_1_step_size = View(in.q_r);
        auto negative_column_2_step_size = View(in.q_m);
        auto negative_column_3_step_size = View(in.q_c);

        // The wire values for lookup gates are accumulators structured in such a way that the differences w_i -
        // step_size*w_i_shift result in values present in column i of a corresponding table. See the documentation in
        // method get_lookup_accumulators() in  for a detailed explanation.
        auto derived_table_entry_1 = w_1 + gamma + negative_column_1_step_size * w_1_shift;
        auto derived_table_entry_2 = w_2 + negative_column_2_step_size * w_2_shift;
        auto derived_table_entry_3 = w_3 + negative_column_3_step_size * w_3_shift;

        // (w_1 + \gamma q_2*w_1_shift) + η(w_2 + q_m*w_2_shift) + η₂(w_3 + q_c*w_3_shift) + η₃q_index.
        // deg 2 or 3
        return derived_table_entry_1 + derived_table_entry_2 * eta + derived_table_entry_3 * eta_two +
               table_index * eta_three;
    }

    /**
     * @brief Construct the polynomial I whose components are the inverse of the product of the read and write terms
     * @details If the denominators of log derivative lookup relation are read_term and write_term, then I_i =
     * (read_term_i*write_term_i)^{-1}.
     * @note Importantly, I_i = 0 for rows i at which there is no read or write, so the cost of this method is
     * proportional to the actual number of lookups.
     *
     */
    template <typename Polynomials>
    static void compute_logderivative_inverse(Polynomials& polynomials,
                                              auto& relation_parameters,
                                              const size_t circuit_size)
    {
        auto& inverse_polynomial = get_inverse_polynomial(polynomials);

        for (size_t i = 0; i < circuit_size; ++i) {
            // We only compute the inverse if this row contains a lookup gate or data that has been looked up
            if (polynomials.q_lookup[i] == 1 || polynomials.lookup_read_tags[i] == 1) {
                // TODO(https://github.com/AztecProtocol/barretenberg/issues/940): avoid get_row if possible.
                auto row = polynomials.get_row(i); // Note: this is a copy. use sparingly!
                inverse_polynomial[i] = compute_read_term<FF, 0>(row, relation_parameters) *
                                        compute_write_term<FF, 0>(row, relation_parameters);
            }
        }
        // Compute inverse polynomial I in place by inverting the product at each row
        FF::batch_invert(inverse_polynomial);
    };

    /**
     * @brief Log-derivative style lookup argument for conventional lookups form tables with 3 or fewer columns
     * @details The identity to be checked is of the form
     *
     * \sum{i=0}^{n-1} \frac{read_counts_i}{write_term_i} - \frac{q_lookup}{read_term_i} = 0
     *
     * where write_term = table_col_1 + \gamma + table_col_2 * \eta_1 + table_col_3 * \eta_2 + table_index * \eta_3
     * and read_term = derived_table_entry_1 + \gamma + derived_table_entry_2 * \eta_1 + derived_table_entry_3 * \eta_2
     * + table_index * \eta_3, with derived_table_entry_i = w_i - col_step_size_i\cdot w_i_shift. (The table entries
     * must be 'derived' from wire values in this way since the stored witnesses are actually successive accumulators,
     * the differences of which are equal to entries in a table. This is an efficiency trick to avoid using additional
     * gates to reconstruct full size values from the limbs contained in tables).
     *
     * In practice this identity is expressed in terms of polynomials by defining a polynomial of inverses I_i =
     * \frac{1}{read_term_i\cdot write_term_i} then rewriting the above identity as
     *
     * (1) \sum{i=0}^{n-1} (read_counts_i\cdot I_i\cdot read_term_i) - (q_lookup\cdot I_i\cdot write_term_i) = 0
     *
     * This requires a second subrelation to check that polynomial I was computed correctly. For all i, it must hold
     * that
     *
     * (2) I_i\cdot read_term_i\cdot write_term_i - 1 = 0
     *
     * Note that (1) is 'linearly dependent' in the sense that it holds only as a sum across the entire execution trace.
     * (2) on the other hand holds independently at every row. Finally, note that to avoid unnecessary computation, we
     * only compute I_i at indices where the relation is 'active', i.e. on rows which either contain a lookup gate or
     * table data that has been read. For inactive rows i, we set I_i = 0. We can thus rewrite (2) as
     *
     * (2) I_i\cdot read_term_i\cdot write_term_i - is_active_i
     *
     * where is_active = q_lookup + read_tags - q_lookup\cdot read_tags
     *
     * and read_tags is a polynomial taking boolean values indicating whether the table entry at the corresponding row
     * has been read or not.
     * @note This relation utilizes functionality in the log-derivative library to compute the polynomial of inverses
     *
     */
    template <typename ContainerOverSubrelations, typename AllEntities, typename Parameters>
    static void accumulate(ContainerOverSubrelations& accumulator,
                           const AllEntities& in,
                           const Parameters& params,
                           const FF& scaling_factor)
    {
        BB_OP_COUNT_TIME_NAME("Lookup::accumulate");
        // declare the accumulator of the maximum length, in non-ZK Flavors, they are of the same length,
        // whereas in ZK Flavors, the accumulator corresponding log derivative lookup argument sub-relation is the
        // longest
        using Accumulator = typename std::tuple_element_t<1, ContainerOverSubrelations>;
        using View = typename Accumulator::View;
        // allows to re-use the values accumulated by the accumulator of the size smaller than
        // the size of Accumulator declared above
        using ShortView = typename std::tuple_element_t<0, ContainerOverSubrelations>::View;

        const auto inverses = View(in.lookup_inverses);                         // Degree 1
        const auto read_counts = View(in.lookup_read_counts);                   // Degree 1
        const auto read_selector = View(in.q_lookup);                           // Degree 1
        const auto inverse_exists = compute_inverse_exists<Accumulator>(in);    // Degree 2
        const auto read_term = compute_read_term<Accumulator, 0>(in, params);   // Degree 2 (3)
        const auto write_term = compute_write_term<Accumulator, 0>(in, params); // Degree 1 (2)
        const auto write_inverse = inverses * read_term;                        // Degree 3 (4)
        const auto read_inverse = inverses * write_term;                        // Degree 2 (3)

        // Establish the correctness of the polynomial of inverses I. Note: inverses is computed so that the value is 0
        // if !inverse_exists.
        // Degrees:                     2 (3)       1 (2)        1              1
        std::get<0>(accumulator) +=
            ShortView((read_term * write_term * inverses - inverse_exists) * scaling_factor); // Deg 4 (6)

        // Establish validity of the read. Note: no scaling factor here since this constraint is 'linearly dependent,
        // i.e. enforced across the entire trace, not on a per-row basis.
        // Degrees:                       1            2 (3)            1            3 (4)
        std::get<1>(accumulator) += read_selector * read_inverse - read_counts * write_inverse; // Deg 4 (5)
    }
};

template <typename FF> using LogDerivLookupRelation = Relation<LogDerivLookupRelationImpl<FF>>;

} // namespace bb