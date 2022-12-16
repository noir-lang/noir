#include "prover.hpp"

#define WASM_EXPORT __attribute__((visibility("default")))

using namespace barretenberg;

extern "C" {

typedef std::
    conditional_t<waffle::SYSTEM_COMPOSER == waffle::TURBO, waffle::UnrolledTurboProver, waffle::UnrolledUltraProver>
        WasmUnrolledProver;

WASM_EXPORT void unrolled_prover_process_queue(WasmUnrolledProver* prover)
{
    prover->queue.process_queue();
}

WASM_EXPORT size_t unrolled_prover_get_circuit_size(WasmUnrolledProver* prover)
{
    return prover->get_circuit_size();
}

WASM_EXPORT void unrolled_prover_get_work_queue_item_info(WasmUnrolledProver* prover, uint8_t* result)
{
    auto info = prover->get_queued_work_item_info();
    memcpy(result, &info, sizeof(info));
}

WASM_EXPORT fr* unrolled_prover_get_scalar_multiplication_data(WasmUnrolledProver* prover, size_t work_item_number)
{
    return prover->get_scalar_multiplication_data(work_item_number);
}

WASM_EXPORT size_t unrolled_prover_get_scalar_multiplication_size(WasmUnrolledProver* prover, size_t work_item_number)
{
    return prover->get_scalar_multiplication_size(work_item_number);
}

WASM_EXPORT void unrolled_prover_put_scalar_multiplication_data(WasmUnrolledProver* prover,
                                                                g1::element* result,
                                                                const size_t work_item_number)
{
    prover->put_scalar_multiplication_data(*result, work_item_number);
}

WASM_EXPORT fr* unrolled_prover_get_fft_data(WasmUnrolledProver* prover, fr* shift_factor, size_t work_item_number)
{
    auto data = prover->get_fft_data(work_item_number);
    *shift_factor = data.shift_factor;
    return data.data;
}

WASM_EXPORT void unrolled_prover_put_fft_data(WasmUnrolledProver* prover, fr* result, size_t work_item_number)
{
    prover->put_fft_data(result, work_item_number);
}

WASM_EXPORT fr* unrolled_prover_get_ifft_data(WasmUnrolledProver* prover, size_t work_item_number)
{
    return prover->get_ifft_data(work_item_number);
}

WASM_EXPORT void unrolled_prover_put_ifft_data(WasmUnrolledProver* prover, fr* result, size_t work_item_number)
{
    prover->put_ifft_data(result, work_item_number);
}

WASM_EXPORT void unrolled_prover_execute_preamble_round(WasmUnrolledProver* prover)
{
    prover->execute_preamble_round();
}

WASM_EXPORT void unrolled_prover_execute_first_round(WasmUnrolledProver* prover)
{
    prover->execute_first_round();
}

WASM_EXPORT void unrolled_prover_execute_second_round(WasmUnrolledProver* prover)
{
    prover->execute_second_round();
}

WASM_EXPORT void unrolled_prover_execute_third_round(WasmUnrolledProver* prover)
{
    prover->execute_third_round();
}

WASM_EXPORT void unrolled_prover_execute_fourth_round(WasmUnrolledProver* prover)
{
    prover->execute_fourth_round();
}

WASM_EXPORT void unrolled_prover_execute_fifth_round(WasmUnrolledProver* prover)
{
    prover->execute_fifth_round();
}

WASM_EXPORT void unrolled_prover_execute_sixth_round(WasmUnrolledProver* prover)
{
    prover->execute_sixth_round();
}

WASM_EXPORT size_t unrolled_prover_export_proof(WasmUnrolledProver* prover, uint8_t** proof_data_buf)
{
    auto& proof_data = prover->export_proof().proof_data;
    *proof_data_buf = proof_data.data();
    return proof_data.size();
}
}
