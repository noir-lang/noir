#pragma once
#include <barretenberg/dsl/acir_format/acir_format.hpp>
#include <barretenberg/goblin/goblin.hpp>
#include <barretenberg/proof_system/op_queue/ecc_op_queue.hpp>

namespace acir_proofs {

/**
 * @brief A class responsible for marshalling construction of keys and prover and verifier instances used to prove
 * satisfiability of circuits written in ACIR.
 * @todo: This reflects the design of Plonk. Perhaps we should author new classes to better reflect the
 * structure of the newer code since there's much more of that code now?
 */
class AcirComposer {
  public:
    AcirComposer(size_t size_hint = 0, bool verbose = true);

    template <typename Builder = UltraCircuitBuilder> void create_circuit(acir_format::acir_format& constraint_system);

    std::shared_ptr<proof_system::plonk::proving_key> init_proving_key(acir_format::acir_format& constraint_system);

    std::vector<uint8_t> create_proof(acir_format::acir_format& constraint_system,
                                      acir_format::WitnessVector& witness,
                                      bool is_recursive);

    void load_verification_key(proof_system::plonk::verification_key_data&& data);

    std::shared_ptr<proof_system::plonk::verification_key> init_verification_key();

    bool verify_proof(std::vector<uint8_t> const& proof, bool is_recursive);

    std::string get_solidity_verifier();
    size_t get_exact_circuit_size() { return exact_circuit_size_; };
    size_t get_total_circuit_size() { return total_circuit_size_; };
    size_t get_circuit_subgroup_size() { return circuit_subgroup_size_; };

    std::vector<barretenberg::fr> serialize_proof_into_fields(std::vector<uint8_t> const& proof,
                                                              size_t num_inner_public_inputs);

    std::vector<barretenberg::fr> serialize_verification_key_into_fields();

    // Goblin specific methods
    void create_goblin_circuit(acir_format::acir_format& constraint_system, acir_format::WitnessVector& witness);
    std::vector<uint8_t> create_goblin_proof();
    bool verify_goblin_proof(std::vector<uint8_t> const& proof);

  private:
    acir_format::Builder builder_;
    acir_format::GoblinBuilder goblin_builder_;
    Goblin goblin;
    size_t size_hint_;
    size_t exact_circuit_size_;
    size_t total_circuit_size_;
    size_t circuit_subgroup_size_;
    std::shared_ptr<proof_system::plonk::proving_key> proving_key_;
    std::shared_ptr<proof_system::plonk::verification_key> verification_key_;
    bool verbose_ = true;

    template <typename... Args> inline void vinfo(Args... args)
    {
        if (verbose_) {
            info(args...);
        }
    }
};

} // namespace acir_proofs
