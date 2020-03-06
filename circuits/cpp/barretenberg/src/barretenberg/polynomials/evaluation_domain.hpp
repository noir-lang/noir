#pragma once

#include "../curves/bn254/fr.hpp"
#include "../types.hpp"
#include <vector>

namespace barretenberg {
class evaluation_domain {
  public:
    evaluation_domain()
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
        , roots(nullptr){};

    evaluation_domain(const size_t domain_size, const size_t target_generator_size = 0);
    evaluation_domain(const evaluation_domain& other);
    evaluation_domain(evaluation_domain&& other);

    evaluation_domain& operator=(const evaluation_domain&) = delete;
    evaluation_domain& operator=(evaluation_domain&&);

    ~evaluation_domain();

    void compute_lookup_table();
    void compute_generator_table(const size_t target_generator_size);

    const std::vector<barretenberg::fr*>& get_round_roots() const { return round_roots; };
    const std::vector<barretenberg::fr*>& get_inverse_round_roots() const { return inverse_round_roots; }

    size_t size;
    size_t num_threads;
    size_t thread_size;
    size_t log2_size;
    size_t log2_thread_size;
    size_t log2_num_threads;
    size_t generator_size;

    barretenberg::fr root;
    barretenberg::fr root_inverse;
    barretenberg::fr domain;
    barretenberg::fr domain_inverse;
    barretenberg::fr generator;
    barretenberg::fr generator_inverse;

  private:
    std::vector<barretenberg::fr*> round_roots;
    std::vector<barretenberg::fr*> inverse_round_roots;

    barretenberg::fr* roots;
};
} // namespace barretenberg
