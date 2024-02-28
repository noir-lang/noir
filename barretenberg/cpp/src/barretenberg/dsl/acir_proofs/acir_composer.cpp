#include "acir_composer.hpp"
#include "barretenberg/common/serialize.hpp"
#include "barretenberg/common/throw_or_abort.hpp"
#include "barretenberg/dsl/acir_format/acir_format.hpp"
#include "barretenberg/dsl/types.hpp"
#include "barretenberg/plonk/proof_system/proving_key/serialize.hpp"
#include "barretenberg/plonk/proof_system/verification_key/sol_gen.hpp"
#include "barretenberg/plonk/proof_system/verification_key/verification_key.hpp"
#include "barretenberg/stdlib/primitives/circuit_builders/circuit_builders_fwd.hpp"
#include "contract.hpp"
#include <memory>

namespace acir_proofs {

AcirComposer::AcirComposer(size_t size_hint, bool verbose)
    : size_hint_(size_hint)
    , verbose_(verbose)
{}

/**
 * @brief Populate acir_composer-owned builder with circuit generated from constraint system and an optional witness
 *
 * @tparam Builder
 * @param constraint_system
 * @param witness
 */
template <typename Builder>
void AcirComposer::create_circuit(acir_format::AcirFormat& constraint_system, WitnessVector const& witness)
{
    vinfo("building circuit...");
    builder_ = acir_format::create_circuit<Builder>(constraint_system, size_hint_, witness);
    vinfo("gates: ", builder_.get_total_circuit_size());
    vinfo("circuit is recursive friendly: ", builder_.is_recursive_circuit);
}

std::shared_ptr<bb::plonk::proving_key> AcirComposer::init_proving_key()
{
    acir_format::Composer composer;
    vinfo("computing proving key...");
    proving_key_ = composer.compute_proving_key(builder_);
    return proving_key_;
}

std::vector<uint8_t> AcirComposer::create_proof()
{
    if (!proving_key_) {
        throw_or_abort("Must compute proving key before constructing proof.");
    }

    acir_format::Composer composer(proving_key_, nullptr);

    vinfo("creating proof...");
    std::vector<uint8_t> proof;
    if (builder_.is_recursive_circuit) {
        auto prover = composer.create_prover(builder_);
        proof = prover.construct_proof().proof_data;
    } else {
        auto prover = composer.create_ultra_with_keccak_prover(builder_);
        proof = prover.construct_proof().proof_data;
    }
    vinfo("done.");
    return proof;
}

std::shared_ptr<bb::plonk::verification_key> AcirComposer::init_verification_key()
{
    if (!proving_key_) {
        throw_or_abort("Compute proving key first.");
    }
    vinfo("computing verification key...");
    acir_format::Composer composer(proving_key_, nullptr);
    verification_key_ = composer.compute_verification_key(builder_);

    vinfo("done.");
    return verification_key_;
}

void AcirComposer::load_verification_key(bb::plonk::verification_key_data&& data)
{
    verification_key_ = std::make_shared<bb::plonk::verification_key>(std::move(data),
                                                                      srs::get_bn254_crs_factory()->get_verifier_crs());
}

bool AcirComposer::verify_proof(std::vector<uint8_t> const& proof)
{
    acir_format::Composer composer(proving_key_, verification_key_);

    if (!verification_key_) {
        vinfo("computing verification key...");
        verification_key_ = composer.compute_verification_key(builder_);
        vinfo("done.");
    }

    // Hack. Shouldn't need to do this. 2144 is size with no public inputs.
    builder_.public_inputs.resize((proof.size() - 2144) / 32);

    if (verification_key_->is_recursive_circuit) {
        auto verifier = composer.create_verifier(builder_);
        return verifier.verify_proof({ proof });
    } else {
        auto verifier = composer.create_ultra_with_keccak_verifier(builder_);
        return verifier.verify_proof({ proof });
    }
}

std::string AcirComposer::get_solidity_verifier()
{
    std::ostringstream stream;
    output_vk_sol(stream, verification_key_, "UltraVerificationKey");
    return stream.str() + CONTRACT_SOURCE;
}

/**
 * TODO: We should change this to return a proof without public inputs, since that is what std::verify_proof
 * TODO: takes.
 * @brief Takes in a proof buffer and converts into a vector of field elements.
 *        The Recursion opcode requires the proof serialized as a vector of witnesses.
 *        Use this method to get the witness values!
 *
 * @param proof
 * @param num_inner_public_inputs - number of public inputs on the proof being serialized
 */
std::vector<bb::fr> AcirComposer::serialize_proof_into_fields(std::vector<uint8_t> const& proof,
                                                              size_t num_inner_public_inputs)
{
    plonk::transcript::StandardTranscript transcript(proof,
                                                     acir_format::Composer::create_manifest(num_inner_public_inputs),
                                                     plonk::transcript::HashType::PedersenBlake3s,
                                                     16);

    return acir_format::export_transcript_in_recursion_format(transcript);
}

/**
 * @brief Takes in a verification key buffer and converts into a vector of field elements.
 *        The Recursion opcode requires the vk serialized as a vector of witnesses.
 *        Use this method to get the witness values!
 *        The composer should already have a verification key initialized.
 */
std::vector<bb::fr> AcirComposer::serialize_verification_key_into_fields()
{
    return acir_format::export_key_in_recursion_format(verification_key_);
}

template void AcirComposer::create_circuit<UltraCircuitBuilder>(acir_format::AcirFormat& constraint_system,
                                                                WitnessVector const& witness);

} // namespace acir_proofs
