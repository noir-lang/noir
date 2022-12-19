#pragma once

#include "../../plonk/transcript/transcript_wrappers.hpp"
#include "../proving_key/proving_key.hpp"

namespace waffle {
class work_queue {

  public:
    enum WorkType { FFT, SMALL_FFT, IFFT, SCALAR_MULTIPLICATION };
    enum MSMSize { N, N_PLUS_ONE };

    struct work_item_info {
        uint32_t num_scalar_multiplications;
        uint32_t num_ffts;
        uint32_t num_iffts;
    };

    struct work_item {
        WorkType work_type;
        barretenberg::fr* mul_scalars;
        std::string tag;
        barretenberg::fr constant;
        const size_t index;
    };

    struct queued_fft_inputs {
        barretenberg::fr* data;
        barretenberg::fr shift_factor;
    };

    work_queue(proving_key* prover_key = nullptr, transcript::StandardTranscript* prover_transcript = nullptr);

    work_queue(const work_queue& other) = default;
    work_queue(work_queue&& other) = default;
    work_queue& operator=(const work_queue& other) = default;
    work_queue& operator=(work_queue&& other) = default;

    work_item_info get_queued_work_item_info() const;

    barretenberg::fr* get_scalar_multiplication_data(const size_t work_item_number) const;

    size_t get_scalar_multiplication_size(const size_t work_item_number) const;

    barretenberg::fr* get_ifft_data(const size_t work_item_number) const;

    void put_ifft_data(barretenberg::fr* result, const size_t work_item_number);

    queued_fft_inputs get_fft_data(const size_t work_item_number) const;

    void put_fft_data(barretenberg::fr* result, const size_t work_item_number);

    void put_scalar_multiplication_data(const barretenberg::g1::affine_element result, const size_t work_item_number);

    void flush_queue();

    void add_to_queue(const work_item& item);

    void process_queue();

    std::vector<work_item> get_queue() const;

  private:
    proving_key* key;
    transcript::StandardTranscript* transcript;
    std::vector<work_item> work_item_queue;
};
} // namespace waffle