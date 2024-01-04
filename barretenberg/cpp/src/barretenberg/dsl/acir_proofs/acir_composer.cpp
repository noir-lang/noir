#include "acir_composer.hpp"
#include "barretenberg/common/serialize.hpp"
#include "barretenberg/common/throw_or_abort.hpp"
#include "barretenberg/dsl/acir_format/acir_format.hpp"
#include "barretenberg/dsl/acir_format/recursion_constraint.hpp"
#include "barretenberg/dsl/types.hpp"
#include "barretenberg/goblin/mock_circuits.hpp"
#include "barretenberg/plonk/proof_system/proving_key/proving_key.hpp"
#include "barretenberg/plonk/proof_system/proving_key/serialize.hpp"
#include "barretenberg/plonk/proof_system/verification_key/sol_gen.hpp"
#include "barretenberg/plonk/proof_system/verification_key/verification_key.hpp"
#include "barretenberg/srs/factories/crs_factory.hpp"
#include "contract.hpp"

namespace acir_proofs {

AcirComposer::AcirComposer(size_t size_hint, bool verbose)
    : size_hint_(size_hint)
    , verbose_(verbose)
{}

template <typename Builder> void AcirComposer::create_circuit(acir_format::acir_format& constraint_system)
{
    // this seems to have made sense for plonk but no longer makes sense for Honk? if we return early then the
    // sizes below never get set and that eventually causes too few srs points to be extracted
    if (builder_.get_num_gates() > 1) {
        return;
    }
    vinfo("building circuit...");
    builder_ = acir_format::create_circuit<Builder>(constraint_system, size_hint_);
    exact_circuit_size_ = builder_.get_num_gates();
    total_circuit_size_ = builder_.get_total_circuit_size();
    circuit_subgroup_size_ = builder_.get_circuit_subgroup_size(total_circuit_size_);
    size_hint_ = circuit_subgroup_size_;
    vinfo("gates: ", builder_.get_total_circuit_size());
}

std::shared_ptr<proof_system::plonk::proving_key> AcirComposer::init_proving_key(
    acir_format::acir_format& constraint_system)
{
    create_circuit(constraint_system);
    acir_format::Composer composer;
    vinfo("computing proving key...");
    proving_key_ = composer.compute_proving_key(builder_);
    return proving_key_;
}

std::vector<uint8_t> AcirComposer::create_proof(acir_format::acir_format& constraint_system,
                                                acir_format::WitnessVector& witness,
                                                bool is_recursive)
{
    vinfo("building circuit with witness...");
    builder_ = acir_format::Builder(size_hint_);
    create_circuit_with_witness(builder_, constraint_system, witness);
    vinfo("gates: ", builder_.get_total_circuit_size());

    auto composer = [&]() {
        if (proving_key_) {
            return acir_format::Composer(proving_key_, nullptr);
        }

        acir_format::Composer composer;
        vinfo("computing proving key...");
        proving_key_ = composer.compute_proving_key(builder_);
        vinfo("done.");
        return composer;
    }();

    vinfo("creating proof...");
    std::vector<uint8_t> proof;
    if (is_recursive) {
        auto prover = composer.create_prover(builder_);
        proof = prover.construct_proof().proof_data;
    } else {
        auto prover = composer.create_ultra_with_keccak_prover(builder_);
        proof = prover.construct_proof().proof_data;
    }
    vinfo("done.");
    return proof;
}

void AcirComposer::create_goblin_circuit(acir_format::acir_format& constraint_system,
                                         acir_format::WitnessVector& witness)
{
    // The public inputs in constraint_system do not index into "witness" but rather into the future "variables" which
    // it assumes will be equal to witness but with a prepended zero. We want to remove this +1 so that public_inputs
    // properly indexes into witness because we're about to make calls like add_variable(witness[public_inputs[idx]]).
    // Once the +1 is removed from noir, this correction can be removed entirely and we can use
    // constraint_system.public_inputs directly.
    const uint32_t pre_applied_noir_offset = 1;
    std::vector<uint32_t> corrected_public_inputs;
    for (const auto& index : constraint_system.public_inputs) {
        corrected_public_inputs.emplace_back(index - pre_applied_noir_offset);
    }

    // Construct a builder using the witness and public input data from acir
    goblin_builder_ =
        acir_format::GoblinBuilder{ goblin.op_queue, witness, corrected_public_inputs, constraint_system.varnum };

    // Populate constraints in the builder via the data in constraint_system
    acir_format::build_constraints(goblin_builder_, constraint_system, true);

    // TODO(https://github.com/AztecProtocol/barretenberg/issues/817): Add some arbitrary op gates to ensure the
    // associated polynomials are non-zero and to give ECCVM and Translator some ECC ops to process.
    GoblinMockCircuits::construct_goblin_ecc_op_circuit(goblin_builder_);
}

std::vector<uint8_t> AcirComposer::create_goblin_proof()
{
    return goblin.construct_proof(goblin_builder_);
}

std::shared_ptr<proof_system::plonk::verification_key> AcirComposer::init_verification_key()
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

void AcirComposer::load_verification_key(proof_system::plonk::verification_key_data&& data)
{
    verification_key_ = std::make_shared<proof_system::plonk::verification_key>(
        std::move(data), srs::get_crs_factory()->get_verifier_crs());
}

bool AcirComposer::verify_proof(std::vector<uint8_t> const& proof, bool is_recursive)
{
    acir_format::Composer composer(proving_key_, verification_key_);

    if (!verification_key_) {
        vinfo("computing verification key...");
        verification_key_ = composer.compute_verification_key(builder_);
        vinfo("done.");
    }

    // Hack. Shouldn't need to do this. 2144 is size with no public inputs.
    builder_.public_inputs.resize((proof.size() - 2144) / 32);

    // TODO: We could get rid of this, if we made the Noir program specify whether something should be
    // TODO: created with the recursive setting or not. ie:
    //
    // #[recursive_friendly]
    // fn main() {}
    // would put in the ACIR that we want this to be recursion friendly with a flag maybe and the backend
    // would set the is_recursive flag to be true.
    // This would eliminate the need for nargo to have a --recursive flag
    //
    // End result is that we may just be able to get it off of builder_, like builder_.is_recursive_friendly
    if (is_recursive) {
        auto verifier = composer.create_verifier(builder_);
        return verifier.verify_proof({ proof });
    } else {
        auto verifier = composer.create_ultra_with_keccak_verifier(builder_);
        return verifier.verify_proof({ proof });
    }
}

bool AcirComposer::verify_goblin_proof(std::vector<uint8_t> const& proof)
{
    return goblin.verify_proof({ proof });
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
std::vector<barretenberg::fr> AcirComposer::serialize_proof_into_fields(std::vector<uint8_t> const& proof,
                                                                        size_t num_inner_public_inputs)
{
    transcript::StandardTranscript transcript(proof,
                                              acir_format::Composer::create_manifest(num_inner_public_inputs),
                                              transcript::HashType::PedersenBlake3s,
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
