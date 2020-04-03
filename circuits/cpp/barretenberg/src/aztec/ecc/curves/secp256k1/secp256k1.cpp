#include "./secp256k1.hpp"

namespace secp256k1 {
namespace {

constexpr size_t max_num_generators = 1 << 10;

} // namespace
g1::affine_element get_generator(const size_t generator_index)
{
    static auto generators = g1::derive_generators<max_num_generators>();
    ASSERT(generator_index < max_num_generators);
    return generators[generator_index];
}
} // namespace secp256k1