#pragma once
#include <barretenberg/dsl/acir_format/acir_format.hpp>

namespace acir_proofs {

/**
 * @brief A class responsible for marshalling construction of keys and prover and verifier instances used to prove
 * satisfiability of circuits written in ACIR.
 * @todo: This reflects the design of Plonk. Perhaps we should author new classes to better reflect the
 * structure of the newer code since there's much more of that code now?
 */
class AcirComposer {

    using WitnessVector = std::vector<fr, ContainerSlabAllocator<fr>>;

  public:
    AcirComposer(size_t size_hint = 0, bool verbose = true);

    template <typename Builder = UltraCircuitBuilder>
    void create_circuit(acir_format::AcirFormat& constraint_system, WitnessVector const& witness = {});

    std::shared_ptr<bb::plonk::proving_key> init_proving_key();

    std::vector<uint8_t> create_proof(bool is_recursive);

    void load_verification_key(bb::plonk::verification_key_data&& data);

    std::shared_ptr<bb::plonk::verification_key> init_verification_key();

    bool verify_proof(std::vector<uint8_t> const& proof, bool is_recursive);

    std::string get_solidity_verifier();
    size_t get_total_circuit_size() { return builder_.get_total_circuit_size(); };
    size_t get_dyadic_circuit_size() { return builder_.get_circuit_subgroup_size(builder_.get_total_circuit_size()); };

    std::vector<bb::fr> serialize_proof_into_fields(std::vector<uint8_t> const& proof, size_t num_inner_public_inputs);

    std::vector<bb::fr> serialize_verification_key_into_fields();

  private:
    acir_format::Builder builder_;
    size_t size_hint_;
    std::shared_ptr<bb::plonk::proving_key> proving_key_;
    std::shared_ptr<bb::plonk::verification_key> verification_key_;
    bool verbose_ = true;

    template <typename... Args> inline void vinfo(Args... args)
    {
        if (verbose_) {
            info(args...);
        }
    }
};

} // namespace acir_proofs
