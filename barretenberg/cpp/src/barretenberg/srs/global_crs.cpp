#include "./global_crs.hpp"
#include "./factories/file_crs_factory.hpp"
#include "./factories/mem_bn254_crs_factory.hpp"
#include "barretenberg/common/throw_or_abort.hpp"
#include "barretenberg/srs/factories/mem_grumpkin_crs_factory.hpp"

namespace {
// TODO(#637): As a PoC we have two global variables for the two CRS but this could be improved to avoid duplication.
std::shared_ptr<bb::srs::factories::CrsFactory<bb::curve::BN254>> crs_factory;
std::shared_ptr<bb::srs::factories::CrsFactory<bb::curve::Grumpkin>> grumpkin_crs_factory;
} // namespace

namespace bb::srs {

// Initializes the crs using the memory buffers
void init_crs_factory(std::vector<g1::affine_element> const& points, g2::affine_element const g2_point)
{
    crs_factory = std::make_shared<factories::MemBn254CrsFactory>(points, g2_point);
}

// Initializes crs from a file path this we use in the entire codebase
void init_crs_factory(std::string crs_path)
{
    crs_factory = std::make_shared<factories::FileCrsFactory<bb::curve::BN254>>(crs_path);
}

// Initializes the crs using the memory buffers
void init_grumpkin_crs_factory(std::vector<bb::curve::Grumpkin::AffineElement> const& points)
{
    grumpkin_crs_factory = std::make_shared<factories::MemGrumpkinCrsFactory>(points);
}

void init_grumpkin_crs_factory(std::string crs_path)
{
    grumpkin_crs_factory = std::make_shared<factories::FileCrsFactory<curve::Grumpkin>>(crs_path);
}

std::shared_ptr<factories::CrsFactory<curve::BN254>> get_crs_factory()
{
    if (!crs_factory) {
        throw_or_abort("You need to initalize the global CRS with a call to init_crs_factory(...)!");
    }
    return crs_factory;
}

std::shared_ptr<factories::CrsFactory<curve::Grumpkin>> get_grumpkin_crs_factory()
{
    if (!grumpkin_crs_factory) {
        throw_or_abort("You need to initalize the global CRS with a call to init_grumpkin_crs_factory(...)!");
    }
    return grumpkin_crs_factory;
}
} // namespace bb::srs