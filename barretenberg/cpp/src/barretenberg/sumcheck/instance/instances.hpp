#pragma once
#include "barretenberg/sumcheck/instance/prover_instance.hpp"
#include "barretenberg/sumcheck/instance/verifier_instance.hpp"

namespace bb {

template <typename Flavor_, size_t NUM_ = 2> struct ProverInstances_ {
  public:
    static_assert(NUM_ > 1, "Must have at least two prover instances");
    using Flavor = Flavor_;
    using FF = typename Flavor::FF;
    static constexpr size_t NUM = NUM_;
    static constexpr size_t NUM_SUBRELATIONS = Flavor::NUM_SUBRELATIONS;
    using Instance = ProverInstance_<Flavor>;

    using ArrayType = std::array<std::shared_ptr<Instance>, NUM_>;
    // The extended length here is the length of a composition of polynomials.
    static constexpr size_t EXTENDED_LENGTH = (Flavor::MAX_TOTAL_RELATION_LENGTH - 1) * (NUM - 1) + 1;
    static constexpr size_t BATCHED_EXTENDED_LENGTH = (Flavor::MAX_TOTAL_RELATION_LENGTH - 1 + NUM - 1) * (NUM - 1) + 1;
    using RelationParameters = bb::RelationParameters<Univariate<FF, EXTENDED_LENGTH>>;
    using OptimisedRelationParameters = bb::RelationParameters<Univariate<FF, EXTENDED_LENGTH, 0, NUM_ - 1>>;
    using RelationSeparator = std::array<Univariate<FF, BATCHED_EXTENDED_LENGTH>, NUM_SUBRELATIONS - 1>;
    ArrayType _data;
    RelationParameters relation_parameters;
    OptimisedRelationParameters optimised_relation_parameters;
    RelationSeparator alphas;
    std::vector<FF> next_gate_challenges;

    std::shared_ptr<Instance> const& operator[](size_t idx) const { return _data[idx]; }
    typename ArrayType::iterator begin() { return _data.begin(); };
    typename ArrayType::iterator end() { return _data.end(); };
    ProverInstances_() = default;
    ProverInstances_(std::vector<std::shared_ptr<Instance>> data)
    {
        ASSERT(data.size() == NUM);
        for (size_t idx = 0; idx < data.size(); idx++) {
            _data[idx] = std::move(data[idx]);
        }
    };

    /**
     * @brief For a fixed row index and each polynomial, construct univariates from the corresponding value
     * from each instance.
     *
     * @example if the row index is 2, and there are 4 instances visually we have
     *
     *           Instance 0       Instance 1       Instance 2       Instance 3
     *           q_c q_l q_r ...  q_c q_l q_r ...  q_c q_l q_r ...  q_c q_l q_r ...
     *           *   *            *   *            *   *            *   *
     *           *   *            *   *            *   *            *   *
     *           a_1 a_2 a_3 ...  b_1 b_2 b_3 ...  c_1 c_2 c_3 ...  d_1 d_2 d_3 ...
     *           *   *            *   *            *   *            *   *
     *
     * and the function returns the univariates [{a_1, b_1, c_1, d_1}, {a_2, b_2, c_2, d_2}, ...]
     *
     * @param row_idx A fixed row position in several execution traces
     * @tparam skip_count Construct univariates that skip some of the indices when computing results
     * @return The univariates whose extensions will be used to construct the combiner.
     */
    template <size_t skip_count = 0> auto row_to_univariates(size_t row_idx) const
    {
        auto insts_prover_polynomials_views = get_polynomials_views();
        std::array<Univariate<FF, NUM, 0, skip_count>, insts_prover_polynomials_views[0].size()> results;
        // Set the size corresponding to the number of rows in the execution trace
        size_t instance_idx = 0;
        // Iterate over the prover polynomials' views corresponding to each instance
        for (auto& get_all : insts_prover_polynomials_views) {
            // Iterate over all columns in the trace execution of an instance and extract their value at row_idx.
            for (auto [result, poly_ptr] : zip_view(results, get_all)) {
                result.evaluations[instance_idx] = poly_ptr[row_idx];
            }
            instance_idx++;
        }
        return results;
    }

  private:
    // Returns a vector containing pointer views to the prover polynomials corresponding to each instance.
    auto get_polynomials_views() const
    {
        // As a practical measure, get the first instance's view to deduce the array type
        std::array<decltype(_data[0]->proving_key.polynomials.get_all()), NUM> views;
        for (size_t i = 0; i < NUM; i++) {
            views[i] = _data[i]->proving_key.polynomials.get_all();
        }
        return views;
    }
};

template <typename Flavor_, size_t NUM_ = 2> struct VerifierInstances_ {
    static_assert(NUM_ > 1, "Must have at least two prover instances");
    using Flavor = Flavor_;
    using VerificationKey = typename Flavor::VerificationKey;
    using Instance = VerifierInstance_<Flavor>;
    using ArrayType = std::array<std::shared_ptr<Instance>, NUM_>;

  public:
    static constexpr size_t NUM = NUM_;
    static constexpr size_t BATCHED_EXTENDED_LENGTH = (Flavor::MAX_TOTAL_RELATION_LENGTH - 1 + NUM - 1) * (NUM - 1) + 1;
    ArrayType _data;
    std::shared_ptr<Instance> const& operator[](size_t idx) const { return _data[idx]; }
    typename ArrayType::iterator begin() { return _data.begin(); };
    typename ArrayType::iterator end() { return _data.end(); };
    VerifierInstances_() = default;

    VerifierInstances_(const std::vector<std::shared_ptr<Instance>>& data)
    {
        ASSERT(data.size() == NUM);
        for (size_t idx = 0; idx < data.size(); idx++) {
            _data[idx] = std::move(data[idx]);
        }
    };
};
} // namespace bb
