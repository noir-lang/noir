#include "prover.hpp"

#define WASM_EXPORT __attribute__((visibility("default")))

extern "C" {

WASM_EXPORT size_t prover_get_circuit_size(waffle::TurboProver* prover)
{
    return prover->get_circuit_size();
}

WASM_EXPORT size_t prover_get_num_queued_scalar_multiplications(waffle::TurboProver* prover)
{
    return prover->queue.get_num_queued_scalar_multiplications();
}

WASM_EXPORT barretenberg::fr* prover_get_scalar_multiplication_data(waffle::TurboProver* prover, size_t work_item_number)
{
    return prover->queue.get_scalar_multiplication_data(work_item_number);
}

WASM_EXPORT void prover_put_scalar_multiplication_data(waffle::TurboProver* prover,
                                           barretenberg::g1::element* result,
                                           const size_t work_item_number)
{
    prover->queue.put_scalar_multiplication_data(*result, work_item_number);
}

WASM_EXPORT void prover_execute_preamble_round(waffle::TurboProver* prover) {
    prover->execute_preamble_round();
}

WASM_EXPORT void prover_execute_first_round(waffle::TurboProver* prover) {
    prover->execute_first_round();
}

WASM_EXPORT void prover_execute_second_round(waffle::TurboProver* prover) {
    prover->execute_second_round();
}

WASM_EXPORT void prover_execute_third_round(waffle::TurboProver* prover) {
    prover->execute_third_round();
}

WASM_EXPORT void prover_execute_fourth_round(waffle::TurboProver* prover) {
    prover->execute_fourth_round();
}

WASM_EXPORT void prover_execute_fifth_round(waffle::TurboProver* prover) {
    prover->execute_fifth_round();
}

WASM_EXPORT size_t prover_export_proof(waffle::TurboProver* prover, uint8_t** proof_data_buf) {
    auto& proof_data = prover->export_proof().proof_data;
    *proof_data_buf = proof_data.data();
    return proof_data.size();
}

}