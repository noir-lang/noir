#include <fstream>

#include "crs.hpp"

#include "barretenberg/srs/reference_string/file_reference_string.hpp"
#include "barretenberg/ecc/curves/bn254/scalar_multiplication/c_bind.hpp"

const int NUM_POINTS_IN_TRANSCRIPT = 5040001;

extern "C" {
/**
 * @brief In WASM, loads the verifier reference string.
 * Used in native code to quickly create an in-memory reference string.
 */
uint8_t* env_load_verifier_crs()
{
    std::ifstream transcript;
    transcript.open("../srs_db/ignition/monomial/transcript00.dat", std::ifstream::binary);
    // We need two g2 points, each 64 bytes.
    size_t g2_points_size = 128;
    std::vector<uint8_t> g2_points(g2_points_size);
    transcript.seekg(28 + NUM_POINTS_IN_TRANSCRIPT * 64);
    transcript.read((char*)g2_points.data(), (std::streamsize)g2_points_size);
    transcript.close();
    auto* g2_points_copy = (uint8_t*)aligned_alloc(64, g2_points_size);
    memcpy(g2_points_copy, g2_points.data(), g2_points_size);
    return g2_points_copy;
}

/**
 * @brief In WASM, loads the prover reference string.
 * Provided as a utility for c-binds to implement a ReferenceStringFactory.
 * In native code, not intended to be used.
 * @param num_points The number of points to load.
 */
uint8_t* env_load_prover_crs(size_t num_points)
{
    // Note: This implementation is only meant to be instructive.
    // This should only be used in c-binds to implement the C++ abstractions.
    std::ifstream transcript;
    transcript.open("../srs_db/ignition/monomial/transcript00.dat", std::ifstream::binary);
    // Each g1 point is 64 bytes.
    size_t g1_points_size = (num_points)*64;
    std::vector<uint8_t> g1_points(g1_points_size);
    transcript.seekg(28);
    transcript.read((char*)g1_points.data(), (std::streamsize)g1_points_size);
    transcript.close();
    auto* g1_points_copy = (uint8_t*)aligned_alloc(64, g1_points_size);
    memcpy(g1_points_copy, g1_points.data(), g1_points_size);
    return g1_points_copy;
}
}