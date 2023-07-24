#include "c_bind.hpp"
#include "./io.hpp"
#include "barretenberg/ecc/curves/bn254/bn254.hpp"
#include "global_crs.hpp"
#include <barretenberg/common/streams.hpp>
#include <barretenberg/ecc/curves/bn254/g1.hpp>
#include <barretenberg/ecc/curves/bn254/g2.hpp>

using namespace barretenberg;

/**
 * WARNING: The SRS is not encoded the same way as all the read/write methods encode.
 * Have to use the old school io functions to parse the buffers.
 */
WASM_EXPORT void srs_init_srs(uint8_t const* points_buf, uint32_t const* num_points, uint8_t const* g2_point_buf)
{
    auto points = std::vector<g1::affine_element>(ntohl(*num_points));
    srs::IO<curve::BN254>::read_affine_elements_from_buffer(points.data(), (char*)points_buf, points.size() * 64);

    g2::affine_element g2_point;
    srs::IO<curve::BN254>::read_affine_elements_from_buffer(&g2_point, (char*)g2_point_buf, 128);

    barretenberg::srs::init_crs_factory(points, g2_point);
}