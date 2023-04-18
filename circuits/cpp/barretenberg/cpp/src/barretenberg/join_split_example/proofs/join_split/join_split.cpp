#include "join_split.hpp"
#include "join_split_circuit.hpp"
#include "compute_circuit_data.hpp"
#include "barretenberg/plonk/proof_system/commitment_scheme/kate_commitment_scheme.hpp"
#include "barretenberg/join_split_example/types.hpp"

namespace join_split_example {
namespace proofs {
namespace join_split {

using namespace proof_system::plonk;
using namespace proof_system::plonk::stdlib::merkle_tree;

static std::shared_ptr<plonk::proving_key> proving_key;
static std::shared_ptr<plonk::verification_key> verification_key;

void init_proving_key(std::shared_ptr<proof_system::ReferenceStringFactory> const& crs_factory, bool mock)
{
    if (proving_key) {
        return;
    }

    // Junk data required just to create proving key.
    join_split_tx tx = noop_tx();

    if (!mock) {
        Composer composer(crs_factory);
        join_split_circuit(composer, tx);
        proving_key = composer.compute_proving_key();
    } else {
        Composer composer;
        join_split_circuit(composer, tx);
        Composer mock_proof_composer(crs_factory);
        join_split_example::proofs::mock::mock_circuit(mock_proof_composer, composer.get_public_inputs());
        proving_key = mock_proof_composer.compute_proving_key();
    }
}

void init_proving_key(std::shared_ptr<proof_system::ProverReferenceString> const& crs,
                      plonk::proving_key_data&& pk_data)
{
    release_key();
    proving_key = std::make_shared<plonk::proving_key>(std::move(pk_data), crs);
}

void release_key()
{
    proving_key.reset();
}

void init_verification_key(std::unique_ptr<proof_system::ReferenceStringFactory>&& crs_factory)
{
    if (!proving_key) {
        std::abort();
    }
    // Patch the 'nothing' reference string fed to init_proving_key.
    proving_key->reference_string = crs_factory->get_prover_crs(proving_key->circuit_size + 1);

    verification_key = Composer::compute_verification_key_base(proving_key, crs_factory->get_verifier_crs());
}

void init_verification_key(std::shared_ptr<proof_system::VerifierMemReferenceString> const& crs,
                           plonk::verification_key_data&& vk_data)
{
    verification_key = std::make_shared<plonk::verification_key>(std::move(vk_data), crs);
}

Prover new_join_split_prover(join_split_tx const& tx, bool mock)
{
    Composer composer(proving_key, nullptr);
    join_split_circuit(composer, tx);

    if (composer.failed()) {
        std::string error = format("composer logic failed: ", composer.err());
        throw_or_abort(error);
    }

    info("public inputs: ", composer.public_inputs.size());

    if (!mock) {
        info("composer gates: ", composer.get_num_gates());
        return composer.create_prover();
    } else {
        Composer mock_proof_composer(proving_key, nullptr);
        join_split_example::proofs::mock::mock_circuit(mock_proof_composer, composer.get_public_inputs());
        info("mock composer gates: ", mock_proof_composer.get_num_gates());
        return mock_proof_composer.create_prover();
    }
}

bool verify_proof(plonk::proof const& proof)
{
    Verifier verifier(verification_key, Composer::create_manifest(verification_key->num_public_inputs));

    std::unique_ptr<plonk::KateCommitmentScheme<plonk::ultra_settings>> kate_commitment_scheme =
        std::make_unique<plonk::KateCommitmentScheme<plonk::ultra_settings>>();
    verifier.commitment_scheme = std::move(kate_commitment_scheme);

    return verifier.verify_proof(proof);
}

std::shared_ptr<plonk::proving_key> get_proving_key()
{
    return proving_key;
}

std::shared_ptr<plonk::verification_key> get_verification_key()
{
    return verification_key;
}

} // namespace join_split
} // namespace proofs
} // namespace join_split_example
