#include "./secp256r1.hpp"

namespace secp256r1 {
namespace {

constexpr size_t max_num_generators = 1 << 10;
static std::array<g1::affine_element, max_num_generators> generators;
// TODO (#LARGE_MODULUS_AFFINE_POINT_COMPRESSION): Rewrite this test after designing point compression for p>2^255
// static bool init_generators = false;

} // namespace
/** TODO (#LARGE_MODULUS_AFFINE_POINT_COMPRESSION): Rewrite this test after designing point compression for p>2^255
g1::affine_element get_generator(const size_t generator_index)
{
    if (!init_generators) {
        generators = g1::derive_generators<max_num_generators>();
        init_generators = true;
    }
    ASSERT(generator_index < max_num_generators);
    return generators[generator_index];
}**/
} // namespace secp256r1