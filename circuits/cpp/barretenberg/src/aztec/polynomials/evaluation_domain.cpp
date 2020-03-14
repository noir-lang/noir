#include "evaluation_domain.hpp"
#include <common/assert.hpp>
#include <common/mem.hpp>
#include <math.h>
#include <memory.h>
#include <numeric/bitop/get_msb.hpp>

#ifndef NO_MULTITHREADING
#include "omp.h"
#endif

using namespace barretenberg;

namespace {
constexpr size_t MIN_GROUP_PER_THREAD = 4;

size_t compute_num_threads(const size_t size)
{
#ifndef NO_MULTITHREADING
    size_t num_threads = static_cast<size_t>(omp_get_max_threads());
#else
    size_t num_threads = 1;
#endif
    if (size <= (num_threads * MIN_GROUP_PER_THREAD)) {
        num_threads = 1;
    }
    return num_threads;
}

void compute_lookup_table_single(const fr& input_root,
                                 const size_t size,
                                 fr* const roots,
                                 std::vector<fr*>& round_roots)
{
    const size_t num_rounds = static_cast<size_t>(numeric::get_msb(size));

    round_roots.emplace_back(&roots[0]);
    for (size_t i = 1; i < num_rounds - 1; ++i) {
        round_roots.emplace_back(round_roots.back() + (1UL << i));
    }

    for (size_t i = 0; i < num_rounds - 1; ++i) {
        const size_t m = 1UL << (i + 1);
        const fr round_root = input_root.pow(static_cast<uint64_t>(size / (2 * m)));
        fr* const current_round_roots = round_roots[i];
        current_round_roots[0] = fr::one();
        for (size_t j = 1; j < m; ++j) {
            current_round_roots[j] = current_round_roots[j - 1] * round_root;
        }
    }
}
} // namespace

evaluation_domain::evaluation_domain(const size_t domain_size, const size_t target_generator_size)
    : size(domain_size)
    , num_threads(compute_num_threads(domain_size))
    , thread_size(domain_size / num_threads)
    , log2_size(static_cast<size_t>(numeric::get_msb(size)))
    , log2_thread_size(static_cast<size_t>(numeric::get_msb(thread_size)))
    , log2_num_threads(static_cast<size_t>(numeric::get_msb(num_threads)))
    , generator_size(target_generator_size ? target_generator_size : domain_size)
    , root(fr::get_root_of_unity(log2_size))
    , root_inverse(root.invert())
    , domain(fr{ size, 0, 0, 0 }.to_montgomery_form())
    , domain_inverse(domain.invert())
    , generator(fr::coset_generator(0))
    , generator_inverse(fr::coset_generator(0).invert())
    , roots(nullptr)
{
    ASSERT((1UL << log2_size) == size || (size == 0));
    ASSERT((1UL << log2_thread_size) == thread_size || (size == 0));
    ASSERT((1UL << log2_num_threads) == num_threads || (size == 0));
}

evaluation_domain::evaluation_domain(const evaluation_domain& other)
    : size(other.size)
    , num_threads(compute_num_threads(other.size))
    , thread_size(other.size / num_threads)
    , log2_size(static_cast<size_t>(numeric::get_msb(size)))
    , log2_thread_size(static_cast<size_t>(numeric::get_msb(thread_size)))
    , log2_num_threads(static_cast<size_t>(numeric::get_msb(num_threads)))
    , generator_size(other.generator_size)
    , root(fr::get_root_of_unity(log2_size))
    , root_inverse(root.invert())
    , domain(fr{ size, 0, 0, 0 }.to_montgomery_form())
    , domain_inverse(domain.invert())
    , generator(other.generator)
    , generator_inverse(other.generator_inverse)
{
    ASSERT((1UL << log2_size) == size);
    ASSERT((1UL << log2_thread_size) == thread_size);
    ASSERT((1UL << log2_num_threads) == num_threads);
    if (other.roots != nullptr) {
        const size_t mem_size = sizeof(fr) * size * 2;
        roots = static_cast<fr*>(aligned_alloc(32, mem_size));
        memcpy(static_cast<void*>(roots), static_cast<void*>(other.roots), mem_size);
        round_roots.resize(log2_size - 1);
        inverse_round_roots.resize(log2_size - 1);
        round_roots[0] = &roots[0];
        inverse_round_roots[0] = &roots[size];
        for (size_t i = 1; i < log2_size - 1; ++i) {
            round_roots[i] = round_roots[i - 1] + (1UL << i);
            inverse_round_roots[i] = inverse_round_roots[i - 1] + (1UL << i);
        }
    } else {
        roots = nullptr;
    }
}

evaluation_domain::evaluation_domain(evaluation_domain&& other)
    : size(other.size)
    , num_threads(compute_num_threads(other.size))
    , thread_size(other.size / num_threads)
    , log2_size(static_cast<size_t>(numeric::get_msb(size)))
    , log2_thread_size(static_cast<size_t>(numeric::get_msb(thread_size)))
    , log2_num_threads(static_cast<size_t>(numeric::get_msb(num_threads)))
    , generator_size(other.generator_size)
    , root(fr::get_root_of_unity(log2_size))
    , root_inverse(root.invert())
    , domain(fr{ size, 0, 0, 0 }.to_montgomery_form())
    , domain_inverse(domain.invert())
    , generator(other.generator)
    , generator_inverse(other.generator_inverse)
{
    roots = other.roots;
    round_roots = std::move(other.round_roots);
    inverse_round_roots = std::move(other.inverse_round_roots);
    other.roots = nullptr;
}

evaluation_domain& evaluation_domain::operator=(evaluation_domain&& other)
{
    size = other.size;
    generator_size = other.generator_size;
    num_threads = compute_num_threads(other.size);
    thread_size = other.size / num_threads;
    log2_size = static_cast<size_t>(numeric::get_msb(size));
    log2_thread_size = static_cast<size_t>(numeric::get_msb(thread_size));
    log2_num_threads = static_cast<size_t>(numeric::get_msb(num_threads));
    fr::__copy(other.root, root);
    fr::__copy(other.root_inverse, root_inverse);
    fr::__copy(other.domain, domain);
    fr::__copy(other.domain_inverse, domain_inverse);
    fr::__copy(other.generator, generator);
    fr::__copy(other.generator_inverse, generator_inverse);
    if (roots != nullptr) {
        aligned_free(roots);
    }
    roots = nullptr;
    if (other.roots != nullptr) {
        roots = other.roots;
        round_roots = std::move(other.round_roots);
        inverse_round_roots = std::move(other.inverse_round_roots);
    }
    other.roots = nullptr;
    return *this;
}

evaluation_domain::~evaluation_domain()
{
    if (roots != nullptr) {
        aligned_free(roots);
    }
}

void evaluation_domain::compute_lookup_table()
{
    ASSERT(roots == nullptr);
    roots = (fr*)(aligned_alloc(32, sizeof(fr) * size * 2));
    compute_lookup_table_single(root, size, roots, round_roots);
    compute_lookup_table_single(root_inverse, size, &roots[size], inverse_round_roots);
}
