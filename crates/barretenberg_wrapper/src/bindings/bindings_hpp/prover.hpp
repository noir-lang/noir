//from barretenberg/src/aztec/plonk/proof_system/prover/c_bind.cpp
//cannot generate bindings, TO BE REMOVED a priori
#include <stdint.h>
#include <stddef.h>
#include "../../../aztec2-internal/barretenberg/src/aztec/plonk/proof_system/prover/prover.hpp"

#define WASM_EXPORT __attribute__((visibility("default")))

using namespace barretenberg;

extern "C" {

WASM_EXPORT void prover_process_queue(waffle::TurboProver* prover);

WASM_EXPORT size_t prover_get_circuit_size(waffle::TurboProver* prover);

WASM_EXPORT void prover_get_work_queue_item_info(waffle::TurboProver* prover, uint8_t* result);

WASM_EXPORT fr* prover_get_scalar_multiplication_data(waffle::TurboProver* prover, size_t work_item_number);

WASM_EXPORT void prover_put_scalar_multiplication_data(waffle::TurboProver* prover,
                                                       g1::element* result,
                                                       const size_t work_item_number);

WASM_EXPORT fr* prover_get_fft_data(waffle::TurboProver* prover, fr* shift_factor, size_t work_item_number);

WASM_EXPORT void prover_put_fft_data(waffle::TurboProver* prover, fr* result, size_t work_item_number);
WASM_EXPORT fr* prover_get_ifft_data(waffle::TurboProver* prover, size_t work_item_number);
WASM_EXPORT void prover_put_ifft_data(waffle::TurboProver* prover, fr* result, size_t work_item_number);

WASM_EXPORT void prover_execute_preamble_round(waffle::TurboProver* prover);

WASM_EXPORT void prover_execute_first_round(waffle::TurboProver* prover);

WASM_EXPORT void prover_execute_second_round(waffle::TurboProver* prover);

WASM_EXPORT void prover_execute_third_round(waffle::TurboProver* prover);

WASM_EXPORT void prover_execute_fourth_round(waffle::TurboProver* prover);

WASM_EXPORT void prover_execute_fifth_round(waffle::TurboProver* prover);

WASM_EXPORT void prover_execute_sixth_round(waffle::TurboProver* prover);

WASM_EXPORT size_t prover_export_proof(waffle::TurboProver* prover, uint8_t** proof_data_buf);

WASM_EXPORT void coset_fft_with_generator_shift(fr* coefficients, fr* constant, evaluation_domain* domain);
WASM_EXPORT void ifft(fr* coefficients, evaluation_domain* domain);

WASM_EXPORT void* new_evaluation_domain(size_t circuit_size);

WASM_EXPORT void delete_evaluation_domain(void* domain);
}