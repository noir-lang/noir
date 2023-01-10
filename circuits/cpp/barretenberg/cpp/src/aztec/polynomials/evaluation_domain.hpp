#pragma once
#include <ecc/curves/bn254/fr.hpp>
#include <ecc/curves/grumpkin/grumpkin.hpp>
#include <vector>

namespace barretenberg {

template <typename Fr> class EvaluationDomain {
  public:
    EvaluationDomain()
        : size(0)
        , num_threads(0)
        , thread_size(0)
        , log2_size(0)
        , log2_thread_size(0)
        , log2_num_threads(0)
        , generator_size(0)
        , root(fr::zero())
        , root_inverse(fr::zero())
        , domain(fr::zero())
        , domain_inverse(fr::zero())
        , generator(fr::zero())
        , generator_inverse(fr::zero())
        , four_inverse(fr::zero())
        , roots(nullptr){};

    EvaluationDomain(const size_t domain_size, const size_t target_generator_size = 0);
    EvaluationDomain(const EvaluationDomain& other);
    EvaluationDomain(EvaluationDomain&& other);

    EvaluationDomain& operator=(const EvaluationDomain&) = delete;
    EvaluationDomain& operator=(EvaluationDomain&&);

    ~EvaluationDomain();

    void compute_lookup_table();
    void compute_generator_table(const size_t target_generator_size);

    const std::vector<Fr*>& get_round_roots() const { return round_roots; };
    const std::vector<Fr*>& get_inverse_round_roots() const { return inverse_round_roots; }

    size_t size;        // n, always a power of 2
    size_t num_threads; // num_threads * thread_size = size
    size_t thread_size;
    size_t log2_size;
    size_t log2_thread_size;
    size_t log2_num_threads;
    size_t generator_size;

    Fr root;           // omega; the nth root of unity
    Fr root_inverse;   // omega^{-1}
    Fr domain;         // n; same as size
    Fr domain_inverse; // n^{-1}
    Fr generator;
    Fr generator_inverse;
    Fr four_inverse;

  private:
    std::vector<Fr*> round_roots; // An entry for each of the log(n) rounds: each entry is a pointer to
                                  // the subset of the roots of unity required for that fft round.
                                  // E.g. round_roots[0] = [1, ω^(n/2 - 1)],
                                  //      round_roots[1] = [1, ω^(n/4 - 1), ω^(n/2 - 1), ω^(3n/4 - 1)]
                                  //      ...
    std::vector<Fr*> inverse_round_roots;

    Fr* roots;
};

// tell the compiler we will take care of instantiating these in the .cpp file
extern template class EvaluationDomain<barretenberg::fr>;
extern template class EvaluationDomain<grumpkin::fr>;
// add alias for compatibility
using evaluation_domain = EvaluationDomain<barretenberg::fr>;
} // namespace barretenberg
