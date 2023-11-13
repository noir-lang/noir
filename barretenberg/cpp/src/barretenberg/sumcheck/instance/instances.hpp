#pragma once
#include "barretenberg/sumcheck/instance/prover_instance.hpp"
#include "barretenberg/sumcheck/instance/verifier_instance.hpp"

namespace proof_system::honk {

template <typename Flavor_, size_t NUM_> struct ProverInstances_ {
  public:
    static_assert(NUM_ > 0, "Must have at least one prover instance");
    using Flavor = Flavor_;
    using FF = typename Flavor::FF;
    static constexpr size_t NUM = NUM_;
    using Instance = ProverInstance_<Flavor>;

    using ArrayType = std::array<std::shared_ptr<Instance>, NUM_>;
    // The extended length here is the length of a composition of polynomials.
    static constexpr size_t EXTENDED_LENGTH = (Flavor::MAX_TOTAL_RELATION_LENGTH - 1) * (NUM - 1) + 1;
    using RelationParameters = proof_system::RelationParameters<Univariate<FF, EXTENDED_LENGTH>>;

    ArrayType _data;
    RelationParameters relation_parameters;

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
     * @return The univariates whose extensions will be used to construct the combiner.
     */
    std::vector<Univariate<FF, NUM>> row_to_univariates(size_t row_idx) const
    {
        auto instance_polynomial_views = get_polynomial_pointer_views();
        std::vector<Univariate<FF, NUM>> results;
        // Initialize to our amount of columns
        results.resize(instance_polynomial_views[0].size());
        size_t instance_idx = 0;
        // Iterate instances
        for (auto& pointer_view : instance_polynomial_views) {
            // Iterate columns
            for (auto [result, poly_ptr] : zip_view(results, pointer_view)) {
                // Assign row for each instance
                result.evaluations[instance_idx] = (*poly_ptr)[row_idx];
            }
            instance_idx++;
        }
        return results;
    }

  private:
    auto get_polynomial_pointer_views() const
    {
        // As a practical measure, get the first instance's pointer view to deduce the vector type
        std::vector pointer_views{ _data[0]->prover_polynomials.pointer_view() };
        // complete the views, starting from the second item
        for (size_t i = 1; i < NUM; i++) {
            pointer_views.push_back(_data[i]->prover_polynomials.pointer_view());
        }
        return pointer_views;
    }
};

template <typename Flavor_, size_t NUM_> struct VerifierInstances_ {
    using Flavor = Flavor_;
    using VerificationKey = typename Flavor::VerificationKey;
    using Instance = VerifierInstance_<Flavor>;
    using ArrayType = std::array<std::shared_ptr<Instance>, NUM_>;

  public:
    static constexpr size_t NUM = NUM_;
    ArrayType _data;
    std::shared_ptr<Instance> const& operator[](size_t idx) const { return _data[idx]; }
    typename ArrayType::iterator begin() { return _data.begin(); };
    typename ArrayType::iterator end() { return _data.end(); };
    VerifierInstances_(std::vector<std::shared_ptr<VerificationKey>> vks)
    {
        ASSERT(vks.size() == NUM);
        for (size_t idx = 0; idx < vks.size(); idx++) {
            Instance inst;
            inst.verification_key = std::move(vks[idx]);
            _data[idx] = std::make_unique<Instance>(inst);
        }
    };
};
} // namespace proof_system::honk
