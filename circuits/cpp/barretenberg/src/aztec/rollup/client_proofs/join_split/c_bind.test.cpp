#include "../../tx/user_context.hpp"
#include "c_bind.h"
#include <ecc/curves/bn254/scalar_multiplication/c_bind.hpp>
#include "join_split.hpp"
#include <common/streams.hpp>
#include <crypto/schnorr/schnorr.hpp>
#include <fstream>
#include <gtest/gtest.h>
#include <srs/io.hpp>
#include <plonk/reference_string/pippenger_reference_string.hpp>

using namespace barretenberg;
using namespace rollup::client_proofs::join_split;
using namespace rollup::tx;

TEST(client_proofs_join_split_c_bind, test_create_c_bindings)
{
}