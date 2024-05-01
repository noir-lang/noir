#ifndef NO_MULTITHREADING
#include <cstddef>
#include <functional>

namespace bb {
void parallel_for_omp(size_t num_iterations, const std::function<void(size_t)>& func)
{
#ifndef NO_OMP_MULTITHREADING
#pragma omp parallel for
#endif
    for (size_t i = 0; i < num_iterations; ++i) {
        func(i);
    }
}
} // namespace bb
#endif