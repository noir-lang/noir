#pragma once

#include "../../transcript/transcript_wrappers.hpp"
#include "../proving_key/proving_key.hpp"
#include "../types/program_witness.hpp"

#include <polynomials/iterate_over_domain.hpp>
#include <polynomials/polynomial_arithmetic.hpp>
#include <ecc/curves/bn254/scalar_multiplication/scalar_multiplication.hpp>

namespace waffle {
class work_queue {
  public:
    enum WorkType { FFT, IFFT, SCALAR_MULTIPLICATION };

    struct work_item {
        WorkType work_type;
        barretenberg::fr* mul_scalars;
        std::string tag;
    };

    work_queue(proving_key* prover_key = nullptr,
               program_witness* program_witness = nullptr,
               transcript::StandardTranscript* prover_transcript = nullptr)
        : key(prover_key)
        , witness(program_witness)
        , transcript(prover_transcript)
        , work_item_queue()
    {}

    work_queue(const work_queue& other) = default;
    work_queue(work_queue&& other) = default;
    work_queue& operator=(const work_queue& other) = default;
    work_queue& operator=(work_queue&& other) = default;

    size_t get_num_queued_scalar_multiplications() const
    {
        size_t count = 0;
        for (const auto& item : work_item_queue) {
            if (item.work_type == WorkType::SCALAR_MULTIPLICATION) {
                ++count;
            }
        }
        return count;
    }

    barretenberg::fr* get_scalar_multiplication_data(const size_t work_item_number) const
    {
        size_t count = 0;
        for (const auto& item : work_item_queue) {
            if (item.work_type == WorkType::SCALAR_MULTIPLICATION) {
                if (count == work_item_number) {
                    return item.mul_scalars;
                }
                ++count;
            }
        }
        return nullptr;
    }

    void put_scalar_multiplication_data(const barretenberg::g1::affine_element result, const size_t work_item_number)
    {
        size_t count = 0;
        for (const auto& item : work_item_queue) {
            if (item.work_type == WorkType::SCALAR_MULTIPLICATION) {
                if (count == work_item_number) {
                    transcript->add_element(item.tag, result.to_buffer());
                    return;
                }
                ++count;
            }
        }
    }

    void flush_queue() { process_queue(true); }

    void add_to_queue(const work_item& item) { work_item_queue.push_back(item); }

    void process_queue(const size_t skip_multiplications = false)
    {
        for (const auto& item : work_item_queue) {
            switch (item.work_type) {
            // most expensive op
            case WorkType::SCALAR_MULTIPLICATION: {
                if (!skip_multiplications) {
                    barretenberg::g1::affine_element result(
                        barretenberg::scalar_multiplication::pippenger_unsafe(item.mul_scalars,
                                                                              key->reference_string->get_monomials(),
                                                                              key->small_domain.size,
                                                                              key->pippenger_runtime_state));

                    transcript->add_element(item.tag, result.to_buffer());
                }
                break;
            }
            // About 20% of the cost of a scalar multiplication. For WASM, might be a bit more expensive
            // due to the need to copy memory between web workers
            case WorkType::FFT: {
                barretenberg::polynomial& wire = witness->wires.at(item.tag);
                barretenberg::polynomial& wire_fft = key->wire_ffts.at(item.tag + "_fft");
                barretenberg::polynomial_arithmetic::copy_polynomial(&wire[0], &wire_fft[0], key->n, 4 * key->n + 4);
                wire_fft.coset_fft(key->large_domain);
                wire_fft.add_lagrange_base_coefficient(wire_fft[0]);
                wire_fft.add_lagrange_base_coefficient(wire_fft[1]);
                wire_fft.add_lagrange_base_coefficient(wire_fft[2]);
                wire_fft.add_lagrange_base_coefficient(wire_fft[3]);
                break;
            }
            // 1/4 the cost of an fft (each fft has 1/4 the number of elements)
            case WorkType::IFFT: {
                barretenberg::polynomial& wire = witness->wires.at(item.tag);
                wire.ifft(key->small_domain);
                break;
            }
            default: {
            }
            }
        }
        work_item_queue = std::vector<work_item>();
    }

    std::vector<work_item> get_queue() const { return work_item_queue; }

  private:
    proving_key* key;
    program_witness* witness;
    transcript::StandardTranscript* transcript;
    std::vector<work_item> work_item_queue;
};
} // namespace waffle