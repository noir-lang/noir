#include "standard_example.hpp"
#include <common/log.hpp>
#include <plonk/composer/standard/compute_verification_key.hpp>
#include <plonk/proof_system/commitment_scheme/kate_commitment_scheme.hpp>

namespace rollup {
namespace proofs {
namespace standard_example {

using namespace plonk;

static std::shared_ptr<waffle::proving_key> proving_key;
static std::shared_ptr<waffle::verification_key> verification_key;

void build_circuit(Composer& composer)
{
    uint32_ct a(witness_ct(&composer, 123));
    uint32_ct b(public_witness_ct(&composer, 456));
    bool_ct r = (a + b) == 579;
    composer.assert_equal_constant(r.witness_index, 1);
}

void init_proving_key(std::unique_ptr<waffle::ReferenceStringFactory>&& crs_factory)
{
    Composer composer(std::move(crs_factory));
    build_circuit(composer);
    proving_key = composer.compute_proving_key();
}

void init_verification_key(std::unique_ptr<waffle::ReferenceStringFactory>&& crs_factory)
{
    if (!proving_key) {
        std::abort();
    }
    // Patch the 'nothing' reference string fed to init_proving_key.
    proving_key->reference_string = crs_factory->get_prover_crs(proving_key->n);
    verification_key =
        waffle::standard_composer::compute_verification_key(proving_key, crs_factory->get_verifier_crs());
}

Prover new_prover()
{
    Composer composer(proving_key, nullptr);
    build_circuit(composer);

    info("composer gates: ", composer.get_num_gates());

    Prover prover = composer.create_prover();

    return prover;
}

bool verify_proof(waffle::plonk_proof const& proof)
{
    Verifier verifier(verification_key, Composer::create_manifest(1));

    std::unique_ptr<waffle::KateCommitmentScheme<waffle::standard_settings>> kate_commitment_scheme = 
        std::make_unique<waffle::KateCommitmentScheme<waffle::standard_settings>>();
    verifier.commitment_scheme = std::move(kate_commitment_scheme);

    return verifier.verify_proof(proof);
}

} // namespace standard_example
} // namespace proofs
} // namespace rollup