#include "join_split.hpp"
#include "join_split_circuit.hpp"
#include "compute_circuit_data.hpp"
#include <plonk/composer/turbo/compute_verification_key.hpp>
#include <plonk/proof_system/commitment_scheme/kate_commitment_scheme.hpp>

namespace rollup {
namespace proofs {
namespace join_split {

using namespace plonk;
using namespace plonk::stdlib::merkle_tree;

static std::shared_ptr<waffle::proving_key> proving_key;
static std::shared_ptr<waffle::verification_key> verification_key;

void init_proving_key(std::shared_ptr<waffle::ReferenceStringFactory> const& crs_factory, bool mock)
{
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
        rollup::proofs::mock::mock_circuit(mock_proof_composer, composer.get_public_inputs());
        proving_key = mock_proof_composer.compute_proving_key();
    }
}

void init_proving_key(std::shared_ptr<waffle::ProverReferenceString> const& crs, waffle::proving_key_data&& pk_data)
{
    proving_key = std::make_shared<waffle::proving_key>(std::move(pk_data), crs);
}

void init_verification_key(std::unique_ptr<waffle::ReferenceStringFactory>&& crs_factory)
{
    if (!proving_key) {
        std::abort();
    }
    // Patch the 'nothing' reference string fed to init_proving_key.
    proving_key->reference_string = crs_factory->get_prover_crs(proving_key->n);
    verification_key = waffle::turbo_composer::compute_verification_key(proving_key, crs_factory->get_verifier_crs());
}

void init_verification_key(std::shared_ptr<waffle::VerifierMemReferenceString> const& crs,
                           waffle::verification_key_data&& vk_data)
{
    verification_key = std::make_shared<waffle::verification_key>(std::move(vk_data), crs);
}

UnrolledProver new_join_split_prover(join_split_tx const& tx, bool mock)
{
    Composer composer(proving_key, nullptr);
    join_split_circuit(composer, tx);

    if (composer.failed) {
        info("composer logic failed: ", composer.err);
    }

    info("public inputs: ", composer.public_inputs.size());

    if (!mock) {
        info("composer gates: ", composer.get_num_gates());
        return composer.create_unrolled_prover();
    } else {
        Composer mock_proof_composer(proving_key, nullptr);
        rollup::proofs::mock::mock_circuit(mock_proof_composer, composer.get_public_inputs());
        info("mock composer gates: ", mock_proof_composer.get_num_gates());
        return mock_proof_composer.create_unrolled_prover();
    }
}

bool verify_proof(waffle::plonk_proof const& proof)
{
    UnrolledVerifier verifier(verification_key,
                              Composer::create_unrolled_manifest(verification_key->num_public_inputs));

    std::unique_ptr<waffle::KateCommitmentScheme<waffle::unrolled_turbo_settings>> kate_commitment_scheme =
        std::make_unique<waffle::KateCommitmentScheme<waffle::unrolled_turbo_settings>>();
    verifier.commitment_scheme = std::move(kate_commitment_scheme);

    return verifier.verify_proof(proof);
}

std::shared_ptr<waffle::proving_key> get_proving_key()
{
    return proving_key;
}

std::shared_ptr<waffle::verification_key> get_verification_key()
{
    return verification_key;
}

} // namespace join_split
} // namespace proofs
} // namespace rollup
