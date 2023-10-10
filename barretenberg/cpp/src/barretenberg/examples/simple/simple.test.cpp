#include "simple.hpp"
#include <barretenberg/common/test.hpp>
#include <barretenberg/srs/factories/file_crs_factory.hpp>
#include <filesystem>

namespace examples::simple {

TEST(examples_simple, create_proof)
{
    auto srs_path = std::filesystem::absolute("../srs_db/ignition");
    srs::init_crs_factory(srs_path);
    auto ptrs = create_builder_and_composer();
    auto proof = create_proof(ptrs);
    bool valid = verify_proof(ptrs, proof);
    delete_builder_and_composer(ptrs);
    EXPECT_TRUE(valid);
}

} // namespace examples::simple
