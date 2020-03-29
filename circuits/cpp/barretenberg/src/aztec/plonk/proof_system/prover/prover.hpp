#pragma once
#include "../../transcript/transcript_wrappers.hpp"
#include "../proving_key/proving_key.hpp"
#include "../types/plonk_proof.hpp"
#include "../types/program_settings.hpp"
#include "../types/program_witness.hpp"
#include "../widgets/base_widget.hpp"

namespace waffle {

template <typename settings> class ProverBase {

  public:
    ProverBase(std::shared_ptr<proving_key> input_key = nullptr,
               std::shared_ptr<program_witness> input_witness = nullptr,
               const transcript::Manifest& manifest = transcript::Manifest({}));
    ProverBase(ProverBase&& other);
    ProverBase(const ProverBase& other) = delete;
    ProverBase& operator=(const ProverBase& other) = delete;
    ProverBase& operator=(ProverBase&& other);

    void compute_round_commitments(const size_t round_index);
    void receive_round_commitments(const std::vector<std::string>& tags,
                                   const std::vector<barretenberg::g1::affine_element>& commitments);

    void execute_preamble_round();
    void execute_first_round();
    void execute_second_round();
    void execute_third_round();
    void execute_fourth_round();
    void execute_fifth_round();

    void compute_wire_coefficients();
    void compute_wire_pre_commitments();
    void compute_quotient_pre_commitment();
    void init_quotient_polynomials();
    void compute_opening_elements();

    barretenberg::fr compute_linearisation_coefficients();
    waffle::plonk_proof construct_proof();
    void reset();

    size_t n;

    std::vector<uint32_t> sigma_1_mapping;
    std::vector<uint32_t> sigma_2_mapping;
    std::vector<uint32_t> sigma_3_mapping;

    std::vector<std::unique_ptr<ProverBaseWidget>> widgets;
    transcript::StandardTranscript transcript;

    std::shared_ptr<proving_key> key;
    std::shared_ptr<program_witness> witness;

    bool uses_quotient_mid;
};
extern template class ProverBase<unrolled_standard_settings>;
extern template class ProverBase<unrolled_turbo_settings>;
extern template class ProverBase<standard_settings>;
extern template class ProverBase<turbo_settings>;

typedef ProverBase<unrolled_standard_settings> UnrolledProver;
typedef ProverBase<unrolled_turbo_settings> UnrolledTurboProver;
typedef ProverBase<standard_settings> Prover;
typedef ProverBase<turbo_settings> TurboProver;

} // namespace waffle
