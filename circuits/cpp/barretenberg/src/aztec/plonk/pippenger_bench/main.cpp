#include <chrono>
#include <common/assert.hpp>
#include <cstdlib>
#include <ecc/curves/bn254/scalar_multiplication/scalar_multiplication.hpp>
#include <plonk/reference_string/file_reference_string.hpp>

//#include <valgrind/callgrind.h>
// CALLGRIND_START_INSTRUMENTATION;
// CALLGRIND_STOP_INSTRUMENTATION;
// CALLGRIND_DUMP_STATS;

using namespace barretenberg;

// constexpr size_t NUM_GATES = 1 << 10;

// size_t get_num_rounds(size_t bucket_size)
// {
//     return (127 + bucket_size) / (bucket_size + 1);
// }

// size_t get_num_bucket_adds(const size_t num_rounds, const size_t bucket_size)
// {
//     size_t num_buckets = 1UL << bucket_size;
//     return (2 * num_buckets + 2) * num_rounds;
// }

// size_t get_next_bucket_size(const size_t bucket_size)
// {
//     size_t old_rounds = get_num_rounds(bucket_size);
//     size_t acc = bucket_size;
//     size_t new_rounds = old_rounds;
//     while (old_rounds <= new_rounds)
//     {
//         ++acc;
//         new_rounds = get_num_rounds(acc);
//     }
//     return acc;
// }
constexpr size_t NUM_POINTS = 1 << 20;
std::vector<fr> scalars;
auto reference_string = std::make_shared<waffle::FileReferenceString>(NUM_POINTS, "../srs_db");

const auto init = []() {
    fr element = fr::random_element();
    fr accumulator = element;
    scalars.reserve(NUM_POINTS);
    for (size_t i = 0; i < NUM_POINTS; ++i) {
        accumulator *= element;
        scalars.emplace_back(accumulator);
    }
    //reference_string = waffle::ReferenceString(NUM_POINTS, "../srs_db");

    return 1;
}();
// constexpr double add_to_mixed_add_complexity = 1.36;

int pippenger()
{
    scalar_multiplication::unsafe_pippenger_runtime_state state(NUM_POINTS);
    std::chrono::steady_clock::time_point time_start = std::chrono::steady_clock::now();
    g1::element result =
        scalar_multiplication::pippenger_unsafe(&scalars[0], reference_string->get_monomials(), NUM_POINTS, state);
    std::chrono::steady_clock::time_point time_end = std::chrono::steady_clock::now();
    std::chrono::microseconds diff = std::chrono::duration_cast<std::chrono::microseconds>(time_end - time_start);
    std::cout << "run time: " << diff.count() << "us" << std::endl;
    std::cout << result.x << std::endl;
    return 1;
}
int main()
{
    std::cout << "executing pippenger algorithm" << std::endl;
    pippenger();
    pippenger();
    pippenger();
    pippenger();
    pippenger();
    return 0;
}