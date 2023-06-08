#include <cstddef>
#include <functional>

void parallel_for_omp(size_t num_iterations, const std::function<void(size_t)>& func)
{
#pragma omp parallel for
    for (size_t i = 0; i < num_iterations; ++i) {
        func(i);
    }
}
