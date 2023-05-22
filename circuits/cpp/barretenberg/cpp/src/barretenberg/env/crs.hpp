#include <stddef.h>
#include <stdint.h>

// To be provided by the environment.
// Outputs from a trusted setup "Common reference string" model:
// https://en.wikipedia.org/wiki/Common_reference_string_model For a WASM build, this is provided by the JavaScript
// environment. For a native build, this is provided in this module.

/**
 * @brief In WASM, loads the verifier reference string.
 * Used in native code to quickly create an in-memory reference string.
 * @returns An array of two g2 points.
 */
extern "C" uint8_t* env_load_verifier_crs();

/**
 * @brief In WASM, loads the prover reference string.
 * Provided as a utility for c-binds to implement a ReferenceStringFactory.
 * In native code, not intended to be used.
 * @param num_points The number of g1 points to load.
 * @returns An array of g1 points.
 */
extern "C" uint8_t* env_load_prover_crs(size_t num_points);