#pragma once
#include "barretenberg/ecc/curves/bn254/fr.hpp"
#include "barretenberg/ecc/curves/grumpkin/grumpkin.hpp"
#include <vector>

namespace bb {

template <typename FF> class EvaluationDomain {
  public:
    EvaluationDomain()
        : size(0)
        , num_threads(0)
        , thread_size(0)
        , log2_size(0)
        , log2_thread_size(0)
        , log2_num_threads(0)
        , generator_size(0)
        , root(FF::zero())
        , root_inverse(FF::zero())
        , domain(FF::zero())
        , domain_inverse(FF::zero())
        , generator(FF::zero())
        , generator_inverse(FF::zero())
        , four_inverse(FF::zero())
        , roots(nullptr){};

    EvaluationDomain(const size_t domain_size, const size_t target_generator_size = 0);
    EvaluationDomain(const EvaluationDomain& other);
    EvaluationDomain(EvaluationDomain&& other);

    EvaluationDomain& operator=(const EvaluationDomain&) = delete;
    EvaluationDomain& operator=(EvaluationDomain&&);

    ~EvaluationDomain();

    void compute_lookup_table();
    void compute_generator_table(const size_t target_generator_size);

    const std::vector<FF*>& get_round_roots() const { return round_roots; };
    const std::vector<FF*>& get_inverse_round_roots() const { return inverse_round_roots; }

    size_t size;        // n, always a power of 2
    size_t num_threads; // num_threads * thread_size = size
    size_t thread_size;
    size_t log2_size;
    size_t log2_thread_size;
    size_t log2_num_threads;
    size_t generator_size;

    FF root;           // omega; the nth root of unity
    FF root_inverse;   // omega^{-1}
    FF domain;         // n; same as size
    FF domain_inverse; // n^{-1}
    FF generator;
    FF generator_inverse;
    FF four_inverse;

  private:
    std::vector<FF*> round_roots; // An entry for each of the log(n) rounds: each entry is a pointer to
                                  // the subset of the roots of unity required for that fft round.
                                  // E.g. round_roots[0] = [1, ω^(n/2 - 1)],
                                  //      round_roots[1] = [1, ω^(n/4 - 1), ω^(n/2 - 1), ω^(3n/4 - 1)]
                                  //      ...
    std::vector<FF*> inverse_round_roots;

    std::shared_ptr<FF[]> roots;
};

// add alias for compatibility
using evaluation_domain = EvaluationDomain<bb::fr>;
} // namespace bb
