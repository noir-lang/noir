#pragma once

#include "barretenberg/honk/transcript/transcript.hpp"
#include "barretenberg/srs/global_crs.hpp"
#include <cstddef>
#include <memory>

namespace proof_system::honk {

// Currently only one type of work queue operation but there will likely be others related to Sumcheck
enum WorkType { SCALAR_MULTIPLICATION };

template <typename Curve> class work_queue {

    using CommitmentKey = pcs::CommitmentKey<Curve>;
    using FF = typename Curve::ScalarField;
    using Commitment = typename Curve::AffineElement;

    struct work_item_info {
        uint32_t num_scalar_multiplications;
    };

    struct work_item {
        WorkType work_type = SCALAR_MULTIPLICATION;
        std::span<FF> mul_scalars;
        std::string label;
    };

  private:
    // TODO(luke): Consider handling all transcript interactions in the prover rather than embedding them in the queue.
    proof_system::honk::ProverTranscript<FF>& transcript;
    std::shared_ptr<CommitmentKey> commitment_key;
    std::vector<work_item> work_item_queue;

  public:
    explicit work_queue(auto commitment_key, proof_system::honk::ProverTranscript<FF>& prover_transcript)
        : transcript(prover_transcript)
        , commitment_key(commitment_key){};

    work_queue(const work_queue& other) = default;
    work_queue(work_queue&& other) noexcept = default;
    ~work_queue() = default;

    [[nodiscard]] work_item_info get_queued_work_item_info() const
    {
        uint32_t scalar_mul_count = 0;
        for (const auto& item : work_item_queue) {
            if (item.work_type == WorkType::SCALAR_MULTIPLICATION) {
                ++scalar_mul_count;
            }
        }
        return work_item_info{ scalar_mul_count };
    };

    [[nodiscard]] FF* get_scalar_multiplication_data(size_t work_item_number) const
    {
        size_t count = 0;
        for (const auto& item : work_item_queue) {
            if (item.work_type == WorkType::SCALAR_MULTIPLICATION) {
                if (count == work_item_number) {
                    return const_cast<FF*>(item.mul_scalars.data());
                }
                ++count;
            }
        }
        return nullptr;
    };

    [[nodiscard]] size_t get_scalar_multiplication_size(size_t work_item_number) const
    {
        size_t count = 0;
        for (const auto& item : work_item_queue) {
            if (item.work_type == WorkType::SCALAR_MULTIPLICATION) {
                if (count == work_item_number) {
                    return item.mul_scalars.size();
                }
                ++count;
            }
        }
        return 0;
    };

    void put_scalar_multiplication_data(const Commitment& result, size_t work_item_number)
    {
        size_t count = 0;
        for (const auto& item : work_item_queue) {
            if (item.work_type == WorkType::SCALAR_MULTIPLICATION) {
                if (count == work_item_number) {
                    transcript.send_to_verifier(item.label, result);
                    return;
                }
                ++count;
            }
        }
    };

    void flush_queue() { work_item_queue = std::vector<work_item>(); };

    void add_commitment(std::span<FF> polynomial, std::string label)
    {
        add_to_queue({ SCALAR_MULTIPLICATION, polynomial, label });
    }

    void process_queue()
    {
        for (const auto& item : work_item_queue) {
            switch (item.work_type) {

            case WorkType::SCALAR_MULTIPLICATION: {

                // Run pippenger multi-scalar multiplication.
                auto commitment = commitment_key->commit(item.mul_scalars);

                transcript.send_to_verifier(item.label, commitment);

                break;
            }
            default: {
            }
            }
        }
        work_item_queue = std::vector<work_item>();
    };

    [[nodiscard]] std::vector<work_item> get_queue() const { return work_item_queue; };

  private:
    void add_to_queue(const work_item& item)
    {
        // Note: currently no difference between wasm and native but may be in the future
#if defined(__wasm__)
        work_item_queue.push_back(item);
#else
        work_item_queue.push_back(item);
#endif
    };
};
} // namespace proof_system::honk