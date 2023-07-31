#include "acir_composer.hpp"
#include "barretenberg/common/serialize.hpp"
#include "barretenberg/common/throw_or_abort.hpp"
#include "barretenberg/dsl/acir_format/acir_format.hpp"
#include "barretenberg/dsl/acir_format/recursion_constraint.hpp"
#include "barretenberg/dsl/types.hpp"
#include "barretenberg/plonk/proof_system/proving_key/serialize.hpp"
#include "barretenberg/plonk/proof_system/verification_key/sol_gen.hpp"
#include "barretenberg/plonk/proof_system/verification_key/verification_key.hpp"
#include "barretenberg/srs/factories/crs_factory.hpp"

namespace acir_proofs {

AcirComposer::AcirComposer(size_t size_hint, bool verbose)
    : composer_(/*p_key=*/0, /*v_key=*/0)
    , size_hint_(size_hint)
    , verbose_(verbose)
{}

void AcirComposer::create_circuit(acir_format::acir_format& constraint_system)
{
    builder_ = acir_format::create_circuit(constraint_system, size_hint_);

    // We are done with the constraint system at this point, and we need the memory slab back.
    constraint_system.constraints.clear();
    constraint_system.constraints.shrink_to_fit();

    exact_circuit_size_ = builder_.get_num_gates();
    total_circuit_size_ = builder_.get_total_circuit_size();
    circuit_subgroup_size_ = builder_.get_circuit_subgroup_size(total_circuit_size_);
    size_hint_ = circuit_subgroup_size_;
}

void AcirComposer::init_proving_key(
    std::shared_ptr<barretenberg::srs::factories::CrsFactory<curve::BN254>> const& crs_factory,
    acir_format::acir_format& constraint_system)
{
    vinfo("building circuit... ", size_hint_);
    builder_ = acir_format::Builder(size_hint_);
    acir_format::create_circuit(builder_, constraint_system);

    // We are done with the constraint system at this point, and we need the memory slab back.
    constraint_system.constraints.clear();
    constraint_system.constraints.shrink_to_fit();

    exact_circuit_size_ = builder_.get_num_gates();
    total_circuit_size_ = builder_.get_total_circuit_size();
    circuit_subgroup_size_ = builder_.get_circuit_subgroup_size(total_circuit_size_);

    composer_ = acir_format::Composer(crs_factory);
    vinfo("computing proving key...");
    proving_key_ = composer_.compute_proving_key(builder_);
}

std::vector<uint8_t> AcirComposer::create_proof(
    std::shared_ptr<barretenberg::srs::factories::CrsFactory<curve::BN254>> const& crs_factory,
    acir_format::acir_format& constraint_system,
    acir_format::WitnessVector& witness,
    bool is_recursive)
{
    // Release prior memory first.
    composer_ = acir_format::Composer(/*p_key=*/0, /*v_key=*/0);

    vinfo("building circuit...");
    create_circuit_with_witness(builder_, constraint_system, witness);
    vinfo("gates: ", builder_.get_total_circuit_size());

    composer_ = [&]() {
        if (proving_key_) {
            auto composer = acir_format::Composer(proving_key_, verification_key_);
            // You can't produce the verification key unless you manually set the crs. Which seems like a bug.
            composer_.crs_factory_ = crs_factory;
            return composer;
        } else {
            return acir_format::Composer(crs_factory);
        }
    }();
    if (!proving_key_) {
        vinfo("computing proving key...");
        proving_key_ = composer_.compute_proving_key(builder_);
        vinfo("done.");
    }

    // We are done with the constraint system at this point, and we need the memory slab back.
    constraint_system.constraints.clear();
    constraint_system.constraints.shrink_to_fit();
    witness.clear();
    witness.shrink_to_fit();

    vinfo("creating proof...");
    std::vector<uint8_t> proof;
    if (is_recursive) {
        auto prover = composer_.create_prover(builder_);
        proof = prover.construct_proof().proof_data;
    } else {
        auto prover = composer_.create_ultra_with_keccak_prover(builder_);
        proof = prover.construct_proof().proof_data;
    }
    vinfo("done.");
    return proof;
}

std::shared_ptr<proof_system::plonk::verification_key> AcirComposer::init_verification_key()
{
    vinfo("computing verification key...");
    verification_key_ = composer_.compute_verification_key(builder_);
    vinfo("done.");
    return verification_key_;
}

void AcirComposer::load_verification_key(
    std::shared_ptr<barretenberg::srs::factories::CrsFactory<curve::BN254>> const& crs_factory,
    proof_system::plonk::verification_key_data&& data)
{
    verification_key_ =
        std::make_shared<proof_system::plonk::verification_key>(std::move(data), crs_factory->get_verifier_crs());
    composer_ = acir_format::Composer(proving_key_, verification_key_);
}

bool AcirComposer::verify_proof(std::vector<uint8_t> const& proof, bool is_recursive)
{
    if (!verification_key_) {
        vinfo("computing verification key...");
        verification_key_ = composer_.compute_verification_key(builder_);
        vinfo("done.");
    }

    // Hack. Shouldn't need to do this. 2144 is size with no public inputs.
    builder_.public_inputs.resize((proof.size() - 2144) / 32);

    if (is_recursive) {
        auto verifier = composer_.create_verifier(builder_);
        return verifier.verify_proof({ proof });
    } else {
        auto verifier = composer_.create_ultra_with_keccak_verifier(builder_);
        return verifier.verify_proof({ proof });
    }
}

std::string AcirComposer::get_solidity_verifier()
{
    std::ostringstream stream;
    output_vk_sol(stream, verification_key_, "UltraVerificationKey");
    return stream.str();
}

/**
 * @brief Takes in a proof buffer and converts into a vector of field elements.
 *        The Recursion opcode requires the proof serialized as a vector of witnesses.
 *        Use this method to get the witness values!
 *
 * @param proof
 * @param num_inner_public_inputs - number of public inputs on the proof being serialized
 */
std::vector<barretenberg::fr> AcirComposer::serialize_proof_into_fields(std::vector<uint8_t> const& proof,
                                                                        size_t num_inner_public_inputs)
{
    transcript::StandardTranscript transcript(proof,
                                              acir_format::Composer::create_manifest(num_inner_public_inputs),
                                              transcript::HashType::PlookupPedersenBlake3s,
                                              16);

    return acir_format::export_transcript_in_recursion_format(transcript);
}

/**
 * @brief Takes in a verification key buffer and converts into a vector of field elements.
 *        The Recursion opcode requires the vk serialized as a vector of witnesses.
 *        Use this method to get the witness values!
 *        The composer should already have a verification key initialized.
 */
std::vector<barretenberg::fr> AcirComposer::serialize_verification_key_into_fields()
{
    return acir_format::export_key_in_recursion_format(verification_key_);
}

} // namespace acir_proofs
