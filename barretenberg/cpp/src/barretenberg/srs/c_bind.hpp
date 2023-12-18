#include <barretenberg/common/wasm_export.hpp>
#include <cstdint>

WASM_EXPORT void srs_init_srs(uint8_t const* points_buf, uint32_t const* num_points, uint8_t const* g2_point_buf);
WASM_EXPORT void srs_init_grumpkin_srs(uint8_t const* points_buf, uint32_t const* num_points);