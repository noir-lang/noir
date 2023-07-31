#include <barretenberg/dsl/acir_format/acir_format.hpp>
#include <barretenberg/plonk/proof_system/proving_key/proving_key.hpp>
#include <barretenberg/plonk/proof_system/verification_key/verification_key.hpp>
#include <cstddef>
#include <cstdint>
#include <memory>

namespace acir_proofs {

class AcirComposer {
  public:
    AcirComposer(size_t size_hint = 0, bool verbose = true);

    void create_circuit(acir_format::acir_format& constraint_system);

    void init_proving_key(std::shared_ptr<barretenberg::srs::factories::CrsFactory<curve::BN254>> const& crs_factory,
                          acir_format::acir_format& constraint_system);

    std::vector<uint8_t> create_proof(
        std::shared_ptr<barretenberg::srs::factories::CrsFactory<curve::BN254>> const& crs_factory,
        acir_format::acir_format& constraint_system,
        acir_format::WitnessVector& witness,
        bool is_recursive);

    void load_verification_key(
        std::shared_ptr<barretenberg::srs::factories::CrsFactory<curve::BN254>> const& crs_factory,
        proof_system::plonk::verification_key_data&& data);

    std::shared_ptr<proof_system::plonk::verification_key> init_verification_key();

    bool verify_proof(std::vector<uint8_t> const& proof, bool is_recursive);

    std::string get_solidity_verifier();
    size_t get_exact_circuit_size() { return exact_circuit_size_; };
    size_t get_total_circuit_size() { return total_circuit_size_; };

    std::vector<barretenberg::fr> serialize_proof_into_fields(std::vector<uint8_t> const& proof,
                                                              size_t num_inner_public_inputs);

    std::vector<barretenberg::fr> serialize_verification_key_into_fields();

  private:
    acir_format::Builder builder_;
    acir_format::Composer composer_;
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
