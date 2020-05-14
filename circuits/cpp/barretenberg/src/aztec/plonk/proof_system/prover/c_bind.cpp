#include "prover.hpp"

#define WASM_EXPORT __attribute__((visibility("default")))

using namespace barretenberg;

extern "C" {

WASM_EXPORT void prover_process_queue(waffle::UnrolledTurboProver* prover)
{
    prover->queue.process_queue();
}

WASM_EXPORT size_t prover_get_circuit_size(waffle::UnrolledTurboProver* prover)
{
    return prover->get_circuit_size();
}

WASM_EXPORT void prover_get_work_queue_item_info(waffle::UnrolledTurboProver* prover, uint8_t* result)
{
    auto info = prover->get_queued_work_item_info();
    memcpy(result, &info, sizeof(info));
}

WASM_EXPORT fr* prover_get_scalar_multiplication_data(waffle::UnrolledTurboProver* prover, size_t work_item_number)
{
    return prover->get_scalar_multiplication_data(work_item_number);
}

WASM_EXPORT void prover_put_scalar_multiplication_data(waffle::UnrolledTurboProver* prover,
                                                       g1::element* result,
                                                       const size_t work_item_number)
{
    prover->put_scalar_multiplication_data(*result, work_item_number);
}

WASM_EXPORT fr* prover_get_fft_data(waffle::UnrolledTurboProver* prover, fr* shift_factor, size_t work_item_number)
{
    auto data = prover->get_fft_data(work_item_number);
    *shift_factor = data.shift_factor;
    return data.data;
}

WASM_EXPORT void prover_put_fft_data(waffle::UnrolledTurboProver* prover, fr* result, size_t work_item_number)
{
    prover->put_fft_data(result, work_item_number);
}

WASM_EXPORT fr* prover_get_ifft_data(waffle::UnrolledTurboProver* prover, size_t work_item_number)
{
    return prover->get_ifft_data(work_item_number);
}

WASM_EXPORT void prover_put_ifft_data(waffle::UnrolledTurboProver* prover, fr* result, size_t work_item_number)
{
    prover->put_ifft_data(result, work_item_number);
}

WASM_EXPORT void prover_execute_preamble_round(waffle::UnrolledTurboProver* prover)
{
    prover->execute_preamble_round();
}

WASM_EXPORT void prover_execute_first_round(waffle::UnrolledTurboProver* prover)
{
    prover->execute_first_round();
}

WASM_EXPORT void prover_execute_second_round(waffle::UnrolledTurboProver* prover)
{
    prover->execute_second_round();
}

WASM_EXPORT void prover_execute_third_round(waffle::UnrolledTurboProver* prover)
{
    prover->execute_third_round();
}

WASM_EXPORT void prover_execute_fourth_round(waffle::UnrolledTurboProver* prover)
{
    prover->execute_fourth_round();
}

WASM_EXPORT void prover_execute_fifth_round(waffle::UnrolledTurboProver* prover)
{
    prover->execute_fifth_round();
}

WASM_EXPORT size_t prover_export_proof(waffle::UnrolledTurboProver* prover, uint8_t** proof_data_buf)
{
    auto& proof_data = prover->export_proof().proof_data;
    *proof_data_buf = proof_data.data();
    return proof_data.size();
}

WASM_EXPORT void coset_fft_with_generator_shift(fr* coefficients, fr* constant, evaluation_domain* domain)
{
    polynomial_arithmetic::coset_fft_with_generator_shift(coefficients, *domain, *constant);
}

WASM_EXPORT void ifft(fr* coefficients, evaluation_domain* domain)
{
    polynomial_arithmetic::ifft(coefficients, *domain);
}

WASM_EXPORT void* new_evaluation_domain(size_t circuit_size)
{
    auto domain = new evaluation_domain(circuit_size);
    domain->compute_lookup_table();
    return domain;
}

WASM_EXPORT void delete_evaluation_domain(void* domain)
{
    delete reinterpret_cast<evaluation_domain*>(domain);
}
}
