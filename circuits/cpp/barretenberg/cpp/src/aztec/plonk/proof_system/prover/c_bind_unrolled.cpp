#include "prover.hpp"

#define WASM_EXPORT __attribute__((visibility("default")))

using namespace barretenberg;

extern "C" {

WASM_EXPORT void unrolled_prover_process_queue(waffle::UnrolledTurboProver* prover)
{
    prover->queue.process_queue();
}

WASM_EXPORT size_t unrolled_prover_get_circuit_size(waffle::UnrolledTurboProver* prover)
{
    return prover->get_circuit_size();
}

WASM_EXPORT void unrolled_prover_get_work_queue_item_info(waffle::UnrolledTurboProver* prover, uint8_t* result)
{
    auto info = prover->get_queued_work_item_info();
    memcpy(result, &info, sizeof(info));
}

WASM_EXPORT fr* unrolled_prover_get_scalar_multiplication_data(waffle::UnrolledTurboProver* prover,
                                                               size_t work_item_number)
{
    return prover->get_scalar_multiplication_data(work_item_number);
}

WASM_EXPORT size_t unrolled_prover_get_scalar_multiplication_size(waffle::UnrolledTurboProver* prover,
                                                                  size_t work_item_number)
{
    return prover->get_scalar_multiplication_size(work_item_number);
}

WASM_EXPORT void unrolled_prover_put_scalar_multiplication_data(waffle::UnrolledTurboProver* prover,
                                                                g1::element* result,
                                                                const size_t work_item_number)
{
    prover->put_scalar_multiplication_data(*result, work_item_number);
}

WASM_EXPORT fr* unrolled_prover_get_fft_data(waffle::UnrolledTurboProver* prover,
                                             fr* shift_factor,
                                             size_t work_item_number)
{
    auto data = prover->get_fft_data(work_item_number);
    *shift_factor = data.shift_factor;
    return data.data;
}

WASM_EXPORT void unrolled_prover_put_fft_data(waffle::UnrolledTurboProver* prover, fr* result, size_t work_item_number)
{
    prover->put_fft_data(result, work_item_number);
}

WASM_EXPORT fr* unrolled_prover_get_ifft_data(waffle::UnrolledTurboProver* prover, size_t work_item_number)
{
    return prover->get_ifft_data(work_item_number);
}

WASM_EXPORT void unrolled_prover_put_ifft_data(waffle::UnrolledTurboProver* prover, fr* result, size_t work_item_number)
{
    prover->put_ifft_data(result, work_item_number);
}

WASM_EXPORT void unrolled_prover_execute_preamble_round(waffle::UnrolledTurboProver* prover)
{
    prover->execute_preamble_round();
}

WASM_EXPORT void unrolled_prover_execute_first_round(waffle::UnrolledTurboProver* prover)
{
    prover->execute_first_round();
}

WASM_EXPORT void unrolled_prover_execute_second_round(waffle::UnrolledTurboProver* prover)
{
    prover->execute_second_round();
}

WASM_EXPORT void unrolled_prover_execute_third_round(waffle::UnrolledTurboProver* prover)
{
    prover->execute_third_round();
}

WASM_EXPORT void unrolled_prover_execute_fourth_round(waffle::UnrolledTurboProver* prover)
{
    prover->execute_fourth_round();
}

WASM_EXPORT void unrolled_prover_execute_fifth_round(waffle::UnrolledTurboProver* prover)
{
    prover->execute_fifth_round();
}

WASM_EXPORT void unrolled_prover_execute_sixth_round(waffle::UnrolledTurboProver* prover)
{
    prover->execute_sixth_round();
}

WASM_EXPORT size_t unrolled_prover_export_proof(waffle::UnrolledTurboProver* prover, uint8_t** proof_data_buf)
{
    auto& proof_data = prover->export_proof().proof_data;
    *proof_data_buf = proof_data.data();
    return proof_data.size();
}
}
